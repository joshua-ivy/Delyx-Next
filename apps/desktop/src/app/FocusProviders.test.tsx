import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (cmd: string, args?: unknown) => invoke(cmd, args) }));

import { FocusProviders } from "./FocusProviders";

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function status(anthropic: boolean, openai: boolean) {
  return {
    providers: [
      { id: "anthropic", label: "Anthropic", hasKey: anthropic },
      { id: "openai", label: "OpenAI", hasKey: openai },
    ],
  };
}

describe("FocusProviders", () => {
  it("detects CLIs, shows key state, and saves a pasted key through the bridge", async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "secret_status") return Promise.resolve(status(false, false));
      if (cmd === "local_model_list") return Promise.resolve([]);
      if (cmd === "local_model_list_ollama") return Promise.resolve([]);
      if (cmd === "external_agent_status") {
        return Promise.resolve({
          adapters: [
            { id: "claude-code", kind: "claude_code", label: "Claude Code", status: "available", detail: "found" },
            { id: "codex-cli", kind: "codex_cli", label: "Codex CLI", status: "missing", detail: "missing" },
          ],
        });
      }
      if (cmd === "secret_set") return Promise.resolve(status(true, false));
      return Promise.resolve(status(false, false));
    });

    render(<FocusProviders />);

    // CLI detection: Claude detected, Codex shows the install command.
    expect(await screen.findByText("detected")).toBeTruthy();
    expect(screen.getByText(/npm i -g @openai\/codex/)).toBeTruthy();

    // Anthropic key starts unset, then saving flips it to "set".
    const input = await screen.findByLabelText("Anthropic API key");
    fireEvent.change(input, { target: { value: "sk-test-key" } });
    fireEvent.click(screen.getAllByText("Save")[0]);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("secret_set", { providerId: "anthropic", value: "sk-test-key" }));
    expect(await screen.findByText("set")).toBeTruthy();
  });

  it("falls back to a desktop-only notice when the bridge is unavailable", async () => {
    invoke.mockRejectedValue(new Error("no bridge"));

    render(<FocusProviders />);

    expect(await screen.findByText("web preview")).toBeTruthy();
  });

  it("shows local model sampling fields and saves numeric tuning values", async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "secret_status") return Promise.resolve({ providers: [] });
      if (cmd === "external_agent_status") return Promise.resolve({ adapters: [] });
      if (cmd === "local_model_list") {
        return Promise.resolve([{
          contextWindow: 8192,
          displayName: "Qwen local",
          format: "gguf",
          id: "local-1",
          loadStatus: "loaded",
          modelPath: "C:\\models\\qwen.gguf",
          repeatPenalty: 1.1,
          runtime: "llama.cpp",
          supportsTools: true,
          temperature: 0.7,
          topK: 40,
          topP: 0.9,
        }]);
      }
      if (cmd === "local_model_list_ollama") return Promise.resolve([]);
      if (cmd === "local_model_set_sampling") return Promise.resolve({ message: "sampling saved", status: "ok" });
      return Promise.resolve({ providers: [] });
    });

    render(<FocusProviders />);

    expect(await screen.findByText("Qwen local")).toBeTruthy();
    expect(screen.getByText("Temp")).toBeTruthy();
    expect(screen.getByText("top_p")).toBeTruthy();
    expect(screen.getByText("Repeat")).toBeTruthy();

    fireEvent.change(screen.getByLabelText("local-1 temperature"), { target: { value: "0.5" } });
    fireEvent.change(screen.getByLabelText("local-1 max_tokens"), { target: { value: "2048" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("local_model_set_sampling", {
      request: {
        id: "local-1",
        maxTokens: 2048,
        repeatPenalty: 1.1,
        temperature: 0.5,
        topK: 40,
        topP: 0.9,
      },
    }));
  });
});
