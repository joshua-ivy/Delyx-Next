import { beforeEach, describe, expect, it, vi } from "vitest";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import { runExternalAgent } from "../features/externalAgents/externalAgentClient";
import { appendMessage, markThread } from "./cockpitComposerBindings";
import {
  launchQueuedWorker,
  queueWorkerRun,
  workerCards,
  workerResultText,
  workerTaskFromCard,
} from "./appShellWorkerActions";
import { card, state, thread } from "./appShellWorkerTestFixtures";

vi.mock("../features/approvals/approvalClient", () => ({
  proposeApprovalOverBridge: vi.fn(),
}));
vi.mock("../features/externalAgents/externalAgentClient", () => ({
  runExternalAgent: vi.fn(),
}));
vi.mock("./ShellPreferenceController", () => ({
  notifyLocalAction: vi.fn(),
}));
vi.mock("./cockpitComposerBindings", () => ({
  appendMessage: vi.fn(),
  markThread: vi.fn(),
}));

const propose = vi.mocked(proposeApprovalOverBridge);
const runAgent = vi.mocked(runExternalAgent);
const append = vi.mocked(appendMessage);
const mark = vi.mocked(markThread);

beforeEach(() => {
  vi.clearAllMocks();
  propose.mockImplementation(async (proposal) => proposal);
});

describe("queueWorkerRun", () => {
  it("creates external + terminal cards carrying the task and posts guidance", async () => {
    const composer = state();
    await queueWorkerRun(composer, thread(), "claude-code", "Refactor the parser");

    expect(propose).toHaveBeenCalledTimes(2);
    const [externalCard] = propose.mock.calls[0];
    const [terminalCard] = propose.mock.calls[1];
    expect(externalCard.actionType).toBe("external_agent");
    expect(externalCard.nodeId).toBe("run-1-worker-external");
    expect(externalCard.rationale).toBe("Task: Refactor the parser");
    expect(terminalCard.actionType).toBe("run_terminal");
    expect(terminalCard.scope.commands?.[0]).toContain("claude");
    expect(composer.setActionProposals).toHaveBeenCalled();
    expect(append).toHaveBeenCalledWith(
      composer,
      "thread-1",
      expect.objectContaining({ role: "system", body: expect.stringContaining("Strong worker queued") }),
      "idle",
    );
  });

  it("refuses without an active run", async () => {
    const composer = state();
    await queueWorkerRun(composer, { ...thread(), activeRunId: undefined }, "claude-code", "x");
    expect(propose).not.toHaveBeenCalled();
    expect(append).toHaveBeenCalledWith(
      composer,
      "thread-1",
      expect.objectContaining({ body: expect.stringContaining("active run") }),
      "blocked",
    );
  });
});

describe("launchQueuedWorker", () => {
  it("refuses to launch while a card is still pending", async () => {
    const proposals = [
      card({ status: "approved" }),
      card({ id: "run-1-worker-terminal", nodeId: "run-1-worker-terminal", actionType: "run_terminal", status: "pending", scope: { kind: "terminal", summary: "s", commands: ["claude -p … (read-only)"] } }),
    ];
    await launchQueuedWorker(state(), thread(), proposals);
    expect(runAgent).not.toHaveBeenCalled();
  });

  it("launches with both approvals and posts the parsed result", async () => {
    const proposals = [
      card({ status: "approved" }),
      card({ id: "run-1-worker-terminal", nodeId: "run-1-worker-terminal", actionType: "run_terminal", status: "approved", scope: { kind: "terminal", summary: "s", commands: ["claude -p … (read-only)"] } }),
    ];
    runAgent.mockResolvedValue({
      id: "external-agent-run-1",
      runId: "run-1",
      adapterId: "claude-code",
      status: "completed",
      scope: "root: C:/code/app; isolation: no isolation",
      transcript: [
        { kind: "stdout", message: "Looking at the parser…", timestamp: "1" },
        { kind: "stdout", message: "result: The parser needs X and Y.", timestamp: "2" },
      ],
      terminalOutput: "raw",
      testArtifactIds: [],
      reviewRequired: false,
    });
    const composer = state();

    await launchQueuedWorker(composer, thread(), proposals);

    expect(mark).toHaveBeenCalledWith(composer, "thread-1", "exploring");
    expect(runAgent).toHaveBeenCalledWith(
      "claude-code",
      expect.objectContaining({
        runId: "run-1",
        externalApprovalId: "run-1-worker-external",
        terminalApprovalId: "run-1-worker-terminal",
        task: "Refactor the parser",
        permissionMode: "read_only",
        workingDirectory: "C:/code/app",
        approvedRoots: ["C:/code/app"],
      }),
    );
    expect(append).toHaveBeenCalledWith(
      composer,
      "thread-1",
      expect.objectContaining({ role: "assistant", body: expect.stringContaining("The parser needs X and Y.") }),
      "idle",
    );
  });
});

describe("worker helpers", () => {
  it("workerCards finds both cards by node id", () => {
    const both = workerCards("run-1", [
      card({}),
      card({ id: "run-1-worker-terminal", nodeId: "run-1-worker-terminal", actionType: "run_terminal" }),
    ]);
    expect(both?.external.nodeId).toBe("run-1-worker-external");
    expect(both?.terminal.nodeId).toBe("run-1-worker-terminal");
    expect(workerCards("run-1", [card({})])).toBeUndefined();
  });

  it("workerTaskFromCard strips the task prefix", () => {
    expect(workerTaskFromCard(card({}))).toBe("Refactor the parser");
  });

  it("workerResultText falls back from result event to stdout to terminal", () => {
    const base = {
      id: "a", runId: "r", adapterId: "claude-code", status: "completed" as const,
      scope: "s", testArtifactIds: [], reviewRequired: false,
    };
    expect(workerResultText({ ...base, transcript: [{ kind: "stdout", message: "result: done", timestamp: "1" }], terminalOutput: "" })).toBe("done");
    expect(workerResultText({ ...base, transcript: [{ kind: "stdout", message: "thinking", timestamp: "1" }], terminalOutput: "" })).toBe("thinking");
    expect(workerResultText({ ...base, transcript: [], terminalOutput: "raw tail" })).toBe("raw tail");
  });
});
