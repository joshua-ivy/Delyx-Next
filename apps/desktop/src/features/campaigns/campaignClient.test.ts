import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { parseQaqcReply, sendCampaignTurn } from "./campaignClient";
import type { ModelSettingsView } from "../models/modelTypes";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../models/modelClient", async (importOriginal) => ({
  ...(await importOriginal<object>()),
  sendModelChatStream: vi.fn(async () => streamResult),
}));

const invokeMock = vi.mocked(invoke);
let streamResult = { cancelled: false, model: "model-1", providerId: "delyx-local", text: "Scene." };

const settings = { providers: [], routes: [], selectedProviderId: "delyx-local" } as unknown as ModelSettingsView;

function mockBridge(repair: { rawText: string; repaired: boolean } | undefined) {
  invokeMock.mockImplementation(async (command) => {
    if (command === "campaign_turn_prompt") {
      return { messages: [{ content: "go", role: "user" }] };
    }
    if (command === "campaign_delta_repair") {
      return repair;
    }
    if (command === "campaign_turn_commit") {
      return { campaign: { id: "c1" }, turn: { turnIndex: 0 } };
    }
    throw new Error(`unexpected command ${command}`);
  });
}

describe("sendCampaignTurn delta repair", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    streamResult = { cancelled: false, model: "model-1", providerId: "delyx-local", text: "Scene." };
  });

  it("commits the repaired text when the schema-locked pass adds a delta", async () => {
    mockBridge({ rawText: "Scene.\n\n```delta\n{}\n```", repaired: true });
    await sendCampaignTurn(settings, "c1", "advance", { onToken: () => undefined });
    const commit = invokeMock.mock.calls.find(([command]) => command === "campaign_turn_commit");
    expect((commit?.[1] as { request: { modelText: string } }).request.modelText).toBe(
      "Scene.\n\n```delta\n{}\n```",
    );
  });

  it("skips the repair pass for cancelled partials and commits the partial as-is", async () => {
    streamResult = { ...streamResult, cancelled: true, text: "Half a scene" };
    mockBridge(undefined);
    const result = await sendCampaignTurn(settings, "c1", "advance", { onToken: () => undefined });
    expect(invokeMock.mock.calls.some(([command]) => command === "campaign_delta_repair")).toBe(false);
    expect(result.cancelled).toBe(true);
  });

  it("commits the original text when repair is a no-op or fails", async () => {
    invokeMock.mockImplementation(async (command) => {
      if (command === "campaign_turn_prompt") {
        return { messages: [{ content: "go", role: "user" }] };
      }
      if (command === "campaign_delta_repair") {
        throw new Error("repair backend unavailable");
      }
      return { campaign: { id: "c1" }, turn: { turnIndex: 0 } };
    });
    await sendCampaignTurn(settings, "c1", "advance", { onToken: () => undefined });
    const commit = invokeMock.mock.calls.find(([command]) => command === "campaign_turn_commit");
    expect((commit?.[1] as { request: { modelText: string } }).request.modelText).toBe("Scene.");
  });
});

describe("parseQaqcReply", () => {
  it("recognizes a clean verdict", () => {
    expect(parseQaqcReply("VERDICT: clean")).toEqual({ status: "clean" });
    expect(parseQaqcReply("Some preamble...\nverdict: CLEAN").status).toBe("clean");
  });

  it("captures issue notes after an issues verdict", () => {
    const reply = "VERDICT: issues\n- The M1 Garand was not issued until 1936.\n- Mills died on turn 2.";
    const parsed = parseQaqcReply(reply);
    expect(parsed.status).toBe("corrected");
    expect(parsed.notes).toContain("M1 Garand");
    expect(parsed.notes).toContain("Mills died");
  });

  it("falls back to skipped when the reviewer rambles", () => {
    const parsed = parseQaqcReply("I am not sure what you want from me.");
    expect(parsed.status).toBe("skipped");
    expect(parsed.notes).toContain("not sure");
  });

  it("handles an issues verdict with no detail", () => {
    const parsed = parseQaqcReply("VERDICT: issues");
    expect(parsed.status).toBe("corrected");
    expect(parsed.notes).toBe("Issues flagged without detail.");
  });
});
