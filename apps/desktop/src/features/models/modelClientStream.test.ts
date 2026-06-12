import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { sendModelChatStream, sendModelChatTools } from "./modelClient";
import type { ModelSettingsView } from "./modelTypes";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);

type StreamHandler = (event: { payload: { requestId: string; kind: string; text: string } }) => void;

beforeEach(() => {
  vi.clearAllMocks();
  (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
});

function settings(providerId: string, kind: "delyx_local" | "ollama"): ModelSettingsView {
  return {
    providers: [{
      detail: "ready",
      id: providerId,
      kind,
      label: providerId,
      models: ["model-1"],
      requiresSecret: false,
      status: "ready",
    }],
    routes: [{ modelId: "model-1", providerId, role: "coding", saved: true }],
    selectedProviderId: providerId,
  };
}

describe("sendModelChatStream", () => {
  it("streams deltas into onToken and resolves with the final text", async () => {
    let handler: StreamHandler | undefined;
    listenMock.mockImplementation(async (_event, callback) => {
      handler = callback as StreamHandler;
      return () => undefined;
    });
    invokeMock.mockImplementation(async (_command, args) => {
      const requestId = (args as { request: { requestId: string } }).request.requestId;
      handler?.({ payload: { requestId, kind: "token", text: "Hel" } });
      handler?.({ payload: { requestId, kind: "token", text: "lo" } });
      handler?.({ payload: { requestId: "someone-else", kind: "token", text: "IGNORED" } });
      handler?.({ payload: { requestId, kind: "done", text: "Hello" } });
      return { model: "model-1", providerId: "delyx-local", text: "Hello" };
    });

    const seen: string[] = [];
    const result = await sendModelChatStream(settings("delyx-local", "delyx_local"), [], (accumulated) => {
      seen.push(accumulated);
    });

    expect(seen).toEqual(["Hel", "Hello"]);
    expect(result.text).toBe("Hello");
    expect(result.cancelled).toBe(false);
    expect(invokeMock).toHaveBeenCalledWith("model_chat_stream", expect.anything());
  });

  it("marks the result cancelled when the stream is stopped", async () => {
    let handler: StreamHandler | undefined;
    listenMock.mockImplementation(async (_event, callback) => {
      handler = callback as StreamHandler;
      return () => undefined;
    });
    invokeMock.mockImplementation(async (_command, args) => {
      const requestId = (args as { request: { requestId: string } }).request.requestId;
      handler?.({ payload: { requestId, kind: "token", text: "partial" } });
      handler?.({ payload: { requestId, kind: "cancelled", text: "partial" } });
      return { model: "model-1", providerId: "delyx-local", text: "partial" };
    });

    const result = await sendModelChatStream(settings("delyx-local", "delyx_local"), [], () => undefined);
    expect(result.cancelled).toBe(true);
    expect(result.text).toBe("partial");
  });

  it("tool loop narrates tool calls and streams the final answer", async () => {
    const handlers = new Map<string, StreamHandler>();
    listenMock.mockImplementation(async (eventName, callback) => {
      handlers.set(eventName as string, callback as StreamHandler);
      return () => undefined;
    });
    invokeMock.mockImplementation(async (_command, args) => {
      const requestId = (args as { request: { requestId: string } }).request.requestId;
      const tool = handlers.get("tool-loop");
      const stream = handlers.get("model-stream");
      tool?.({ payload: { requestId, kind: "tool", summary: "read_file src/main.rs" } as never });
      tool?.({ payload: { requestId, kind: "tool_result", summary: "read_file src/main.rs" } as never });
      stream?.({ payload: { requestId, kind: "token", text: "Answer." } });
      stream?.({ payload: { requestId, kind: "done", text: "Answer." } });
      return { model: "model-1", providerId: "delyx-local", text: "Answer." };
    });

    const tools: string[] = [];
    const tokens: string[] = [];
    const result = await sendModelChatTools(
      settings("delyx-local", "delyx_local"),
      [],
      "C:/code/app",
      { onToken: (accumulated) => tokens.push(accumulated), onTool: (summary) => tools.push(summary) },
    );

    expect(tools).toEqual(["read_file src/main.rs"]);
    expect(tokens).toEqual(["Answer."]);
    expect(result.text).toBe("Answer.");
    expect(invokeMock).toHaveBeenCalledWith("model_chat_tools", expect.objectContaining({
      request: expect.objectContaining({ projectRoot: "C:/code/app" }),
    }));
  });

  it("routes tool warnings to onToolWarning without polluting onTool", async () => {
    const handlers = new Map<string, StreamHandler>();
    listenMock.mockImplementation(async (eventName, callback) => {
      handlers.set(eventName as string, callback as StreamHandler);
      return () => undefined;
    });
    invokeMock.mockImplementation(async (_command, args) => {
      const requestId = (args as { request: { requestId: string } }).request.requestId;
      const tool = handlers.get("tool-loop");
      const stream = handlers.get("model-stream");
      tool?.({ payload: { requestId, kind: "tool", summary: "read_file notes.md" } as never });
      tool?.({
        payload: {
          requestId,
          kind: "tool_warning",
          summary: "possible prompt injection in read_file notes.md: instruction_override",
        } as never,
      });
      tool?.({ payload: { requestId: "someone-else", kind: "tool_warning", summary: "IGNORED" } as never });
      stream?.({ payload: { requestId, kind: "done", text: "Answer." } });
      return { model: "model-1", providerId: "delyx-local", text: "Answer." };
    });

    const tools: string[] = [];
    const warnings: string[] = [];
    await sendModelChatTools(settings("delyx-local", "delyx_local"), [], "C:/code/app", {
      onToken: () => undefined,
      onTool: (summary) => tools.push(summary),
      onToolWarning: (summary) => warnings.push(summary),
    });

    expect(tools).toEqual(["read_file notes.md"]);
    expect(warnings).toEqual(["possible prompt injection in read_file notes.md: instruction_override"]);
  });

  it("falls back to the non-streaming call for other providers", async () => {
    invokeMock.mockResolvedValue({ model: "model-1", providerId: "ollama-local", text: "full reply" });

    const seen: string[] = [];
    const result = await sendModelChatStream(settings("ollama-local", "ollama"), [], (accumulated) => {
      seen.push(accumulated);
    });

    expect(listenMock).not.toHaveBeenCalled();
    expect(invokeMock).toHaveBeenCalledWith("model_chat", expect.anything());
    expect(seen).toEqual(["full reply"]);
    expect(result.cancelled).toBe(false);
  });
});
