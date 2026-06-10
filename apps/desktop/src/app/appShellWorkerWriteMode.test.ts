import { beforeEach, describe, expect, it, vi } from "vitest";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import { runExternalAgent } from "../features/externalAgents/externalAgentClient";
import { appendMessage } from "./cockpitComposerBindings";
import {
  launchQueuedWorker,
  parsePlannedFiles,
  queueWorkerRun,
  unplannedEdits,
  workerModeFromCards,
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

beforeEach(() => {
  vi.clearAllMocks();
  propose.mockImplementation(async (proposal) => proposal);
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
