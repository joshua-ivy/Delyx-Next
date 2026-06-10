import { beforeEach, describe, expect, it, vi } from "vitest";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { runExternalAgent } from "../features/externalAgents/externalAgentClient";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { appendMessage, markThread, type ComposerBindingState } from "./cockpitComposerBindings";
import {
  launchQueuedWorker,
  parsePlannedFiles,
  queueWorkerRun,
  unplannedEdits,
  workerCards,
  workerModeFromCards,
  workerResultText,
  workerTaskFromCard,
} from "./appShellWorkerActions";

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

function project(): WorkspaceProject {
  return {
    approvalPolicy: "approval-gated",
    approvedRoots: ["C:/code/app"],
    git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
    id: "project-1",
    indexedFiles: [],
    isolation: { detail: "none", label: "none", mode: "none" },
    lastOpenedLabel: "now",
    name: "app",
    path: "C:/code/app",
    pinned: true,
    rulesFiles: [],
  };
}

function thread(): TaskThread {
  return {
    activeRunId: "run-1",
    archived: false,
    createdAt: "2026-06-09T00:00:00.000Z",
    createdLabel: "now",
    goal: "Refactor the parser",
    id: "thread-1",
    messages: [{ role: "user", body: "Refactor the parser" }],
    mode: "explore",
    projectId: "project-1",
    runIds: ["run-1"],
    status: "idle",
    title: "Refactor the parser",
    updatedAt: "2026-06-09T00:00:00.000Z",
  };
}

function state(): ComposerBindingState {
  return {
    activeProject: project(),
    activeRun: undefined,
    activeThread: thread(),
    modelSettings: { providers: [], routes: [], selectedProviderId: "" },
    setActionProposals: vi.fn(),
    setActiveThreadId: vi.fn(),
    setAgentRuns: vi.fn(),
    setThreads: vi.fn(),
    setThreadState: vi.fn(),
    threads: [thread()],
  };
}

function card(over: Partial<ActionProposalView>): ActionProposalView {
  return {
    id: "run-1-worker-external",
    runId: "run-1",
    nodeId: "run-1-worker-external",
    actionType: "external_agent",
    riskLabel: "high",
    requiredPermission: "Run Claude Code read-only inside the project root",
    rationale: "Task: Refactor the parser",
    expectedResult: "explores",
    scope: { kind: "external_agent", summary: "s", root: "C:/code/app" },
    expiresAt: "2999-01-01T00:00:00.000Z",
    status: "pending",
    ...over,
  };
}

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

describe("write mode", () => {
  it("parsePlannedFiles extracts and strips the files tag", () => {
    const parsed = parsePlannedFiles("fix the parser [files: src/parser.rs, src/parser_tests.rs] please");
    expect(parsed.files).toEqual(["src/parser.rs", "src/parser_tests.rs"]);
    expect(parsed.task).toBe("fix the parser please");
    expect(parsePlannedFiles("no tag here").files).toEqual([]);
  });

  it("refuses to queue a write run without planned files", async () => {
    const composer = state();
    await queueWorkerRun(composer, thread(), "claude-code", "fix the parser", "workspace_write");
    expect(propose).not.toHaveBeenCalled();
    expect(append).toHaveBeenCalledWith(
      composer,
      "thread-1",
      expect.objectContaining({ body: expect.stringContaining("[files:") }),
      "blocked",
    );
  });

  it("queues a write run with absolute planned paths on the approval scope", async () => {
    const composer = state();
    await queueWorkerRun(
      composer,
      thread(),
      "claude-code",
      "fix the parser [files: src/parser.rs]",
      "workspace_write",
    );
    const [externalCard] = propose.mock.calls[0];
    expect(externalCard.requiredPermission).toContain("write access");
    expect(externalCard.scope.paths).toEqual(["C:/code/app/src/parser.rs"]);
    expect(externalCard.rationale).toBe("Task: fix the parser");
    const [terminalCard] = propose.mock.calls[1];
    expect(terminalCard.scope.commands?.[0]).toContain("acceptEdits");
  });

  it("launches a write run with workspace_write, diff capture, and planned files", async () => {
    const writeExternal = card({
      status: "approved",
      requiredPermission: "Run Claude Code with write access to 1 planned file(s)",
      scope: { kind: "external_agent", summary: "s", root: "C:/code/app", paths: ["C:/code/app/src/parser.rs"] },
    });
    const writeTerminal = card({
      id: "run-1-worker-terminal", nodeId: "run-1-worker-terminal", actionType: "run_terminal",
      status: "approved", scope: { kind: "terminal", summary: "s", commands: ["claude -p … (acceptEdits)"] },
    });
    runAgent.mockResolvedValue({
      id: "external-agent-run-2", runId: "run-1", adapterId: "claude-code", status: "completed",
      scope: "root: C:/code/app; isolation: checkpoint",
      transcript: [
        { kind: "file_changed", message: "src/parser.rs", timestamp: "1" },
        { kind: "file_changed", message: "src/sneaky.rs", timestamp: "2" },
        { kind: "stdout", message: "result: Fixed.", timestamp: "3" },
      ],
      terminalOutput: "", diffSummary: "1 modified", testArtifactIds: [], reviewRequired: true,
    });
    const composer = state();

    await launchQueuedWorker(composer, thread(), [writeExternal, writeTerminal]);

    expect(runAgent).toHaveBeenCalledWith(
      "claude-code",
      expect.objectContaining({
        permissionMode: "workspace_write",
        captureDiff: true,
        changedFiles: ["C:/code/app/src/parser.rs"],
      }),
    );
    // The receipt flags the unplanned edit and blocks instead of absorbing it.
    expect(append).toHaveBeenCalledWith(
      composer,
      "thread-1",
      expect.objectContaining({ body: expect.stringContaining("sneaky.rs") }),
      "blocked",
    );
  });

  it("unplannedEdits flags only out-of-plan file_changed events", () => {
    const artifact = {
      id: "a", runId: "r", adapterId: "claude-code", status: "completed" as const, scope: "s",
      transcript: [
        { kind: "file_changed" as const, message: "src/parser.rs", timestamp: "1" },
        { kind: "file_changed" as const, message: "src/other.rs", timestamp: "2" },
        { kind: "stdout" as const, message: "result: x", timestamp: "3" },
      ],
      terminalOutput: "", testArtifactIds: [], reviewRequired: false,
    };
    expect(unplannedEdits(artifact, ["C:/code/app/src/parser.rs"])).toEqual(["src/other.rs"]);
    expect(unplannedEdits(artifact, [])).toEqual([]);
  });

  it("workerModeFromCards derives mode from the external card", () => {
    expect(workerModeFromCards({
      external: card({ requiredPermission: "Run Claude Code with write access to 2 planned file(s)" }),
      terminal: card({}),
    })).toBe("workspace_write");
    expect(workerModeFromCards({ external: card({}), terminal: card({}) })).toBe("read_only");
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
