import { beforeEach, describe, expect, it, vi } from "vitest";

import { finalizeThreadRunOverBridge } from "../features/threads/threadClient";
import { notifyLocalAction } from "./ShellPreferenceController";
import { recordFinalSupportForActiveThread } from "./appShellFinalAnswerActions";

vi.mock("../features/threads/threadClient", () => ({ finalizeThreadRunOverBridge: vi.fn() }));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));

const finalize = vi.mocked(finalizeThreadRunOverBridge);
const notify = vi.mocked(notifyLocalAction);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("recordFinalSupportForActiveThread", () => {
  it("blocks final support while review findings are unresolved", async () => {
    await recordFinalSupportForActiveThread({
      activeRun: { id: "run-1" } as never,
      activeThread: thread() as never,
      reviews: [reviewWithFinding()],
      setAgentRuns: vi.fn(),
      setThreadState: vi.fn(),
      setThreads: vi.fn(),
    });

    expect(finalize).not.toHaveBeenCalled();
    expect(notify).toHaveBeenCalledWith(
      "Resolve review findings before recording final support",
      "warning",
    );
  });
});

function thread() {
  return {
    id: "thread-1",
    messages: [
      { body: "Need a fix", role: "user" },
      { body: "The patch is ready.", role: "assistant" },
    ],
  };
}

function reviewWithFinding() {
  return {
    decision: "pending",
    findings: [{ id: "finding-1" }],
    id: "review-1",
    runId: "run-1",
  } as never;
}
