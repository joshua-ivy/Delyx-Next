import { beforeEach, describe, expect, it, vi } from "vitest";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import { runCodexExternalAgent } from "../features/externalAgents/externalAgentClient";
import { notifyLocalAction } from "./ShellPreferenceController";
import { runCodexExternalAgentForRun, type ExternalAgentPreviewState } from "./appShellExternalAgentActions";
import { codexNodeId } from "./appShellExternalAgentApprovals";

vi.mock("../features/approvals/approvalClient", () => ({ proposeApprovalOverBridge: vi.fn() }));
vi.mock("../features/externalAgents/externalAgentClient", () => ({
  previewExternalAgentContract: vi.fn(),
  runCodexExternalAgent: vi.fn(),
}));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));

const notify = vi.mocked(notifyLocalAction);
const proposeApproval = vi.mocked(proposeApprovalOverBridge);
const runCodex = vi.mocked(runCodexExternalAgent);

beforeEach(() => {
  vi.clearAllMocks();
  proposeApproval.mockImplementation(async (proposal) => proposal);
  runCodex.mockResolvedValue({
    adapterId: "codex-cli",
    diffSummary: undefined,
    id: "external-run-1",
    reviewRequired: false,
    runId: "run-1",
    scope: "C:/repo",
    status: "completed",
    terminalOutput: "ok",
    testArtifactIds: [],
    transcript: [],
  });
});

describe("runCodexExternalAgentForRun", () => {
  it("runs Codex with unexpired approved approvals", async () => {
    await runCodexExternalAgentForRun(state({
      actionProposals: [approval("external_agent", "approved"), approval("run_terminal", "approved")],
    }));

    expect(runCodex).toHaveBeenCalledWith(expect.objectContaining({
      externalApprovalId: "approval-external_agent-approved",
      permissionMode: "read_only",
      terminalApprovalId: "approval-run_terminal-approved",
    }));
    expect(proposeApproval).not.toHaveBeenCalled();
  });

  it("queues fresh proposals instead of reusing expired approval records", async () => {
    await runCodexExternalAgentForRun(state({
      actionProposals: [approval("external_agent", "expired"), approval("run_terminal", "expired")],
    }));

    expect(runCodex).not.toHaveBeenCalled();
    expect(proposeApproval).toHaveBeenCalledTimes(2);
    expect(proposeApproval).toHaveBeenCalledWith(expect.objectContaining({
      actionType: "external_agent",
      id: expect.stringMatching(/^approval-run-1-codex-external-read_only-\d+$/),
    }));
    expect(proposeApproval).toHaveBeenCalledWith(expect.objectContaining({
      actionType: "run_terminal",
      id: expect.stringMatching(/^approval-run-1-codex-terminal-read_only-\d+$/),
    }));
  });

  it("does not requeue denied Codex approvals", async () => {
    await runCodexExternalAgentForRun(state({
      actionProposals: [approval("external_agent", "denied"), approval("run_terminal", "approved")],
    }));

    expect(runCodex).not.toHaveBeenCalled();
    expect(proposeApproval).not.toHaveBeenCalled();
    expect(notify).toHaveBeenCalledWith(
      "Codex approval was denied; Delyx will not launch Codex for this run",
      "warning",
    );
  });

  it("runs write-capable Codex only with planned files for checkpoint scope", async () => {
    await runCodexExternalAgentForRun(state({
      actionProposals: [
        approval("external_agent", "approved", "workspace_write"),
        approval("run_terminal", "approved", "workspace_write"),
      ],
      activePlan: approvedPlan(["src/main.ts"]),
    }));

    expect(runCodex).toHaveBeenCalledWith(expect.objectContaining({
      captureDiff: true,
      changedFiles: ["C:/repo/src/main.ts"],
      checkpointId: undefined,
      permissionMode: "workspace_write",
      worktreeId: undefined,
    }));
  });

  it("blocks write-capable Codex when no planned files can be checkpointed", async () => {
    await runCodexExternalAgentForRun(state({
      actionProposals: [
        approval("external_agent", "approved", "workspace_write"),
        approval("run_terminal", "approved", "workspace_write"),
      ],
      activePlan: approvedPlan([]),
    }));

    expect(runCodex).not.toHaveBeenCalled();
    expect(notify).toHaveBeenCalledWith(
      "Codex write mode needs planned files so Delyx can checkpoint before launch",
      "warning",
    );
  });
});

function state(overrides: Partial<ExternalAgentPreviewState> = {}): ExternalAgentPreviewState {
  return {
    actionProposals: [],
    activePlan: undefined,
    activeProject: {
      approvalPolicy: "manual",
      approvedRoots: ["C:/repo"],
      git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
      id: "project-1",
      indexedFiles: [],
      isolation: { detail: "none", label: "none", mode: "none" as const },
      lastOpenedLabel: "now",
      name: "Repo",
      path: "C:/repo",
      pinned: true,
      rulesFiles: [],
    },
    activeRun: { artifacts: [], id: "run-1" } as never,
    activeThread: { goal: "Inspect code", id: "thread-1", messages: [] } as never,
    setActionProposals: vi.fn(),
    setAgentRuns: vi.fn(),
    setExternalAgentState: vi.fn(),
    setThreadState: vi.fn(),
    ...overrides,
  };
}

function approval(
  actionType: "external_agent" | "run_terminal",
  status: "approved" | "denied" | "expired" | "pending",
  permissionMode: "read_only" | "workspace_write" = "read_only",
) {
  const runId = "run-1";
  return {
    actionType,
    expectedResult: "Run Codex with captured output.",
    expiresAt: status === "expired" ? "2020-01-01T00:00:00.000Z" : "2999-01-01T00:00:00.000Z",
    id: `approval-${actionType}-${status}`,
    nodeId: codexNodeId(runId, actionType, permissionMode),
    rationale: "External agent run.",
    requiredPermission: actionType === "external_agent" ? "external_agent" : "terminal_command",
    riskLabel: "high" as const,
    rollbackPlan: "Discard the captured artifact.",
    runId,
    scope: { kind: actionType === "external_agent" ? "external_agent" as const : "terminal" as const, root: "C:/repo", summary: "Codex run" },
    status,
  };
}

function approvedPlan(filesLikelyInvolved: string[]) {
  return {
    decision: "approved" as const,
    explore: {
      architectureSummary: "",
      projectCommands: [],
      relevantFiles: [],
      relevantSymbols: [],
      risks: [],
      suggestedNextSteps: [],
      unknowns: [],
    },
    filesLikelyInvolved,
    goalUnderstanding: "Edit planned files.",
    permissionsNeeded: ["external_agent"],
    risks: [],
    rollbackStrategy: "Checkpoint before edits.",
    steps: [],
    testsToRun: [],
    threadId: "thread-1",
  };
}
