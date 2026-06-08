import { beforeEach, describe, expect, it, vi } from "vitest";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { loadPatchSnapshot } from "../features/patches/patchClient";
import type { PatchProposalView } from "../features/patches/patchTypes";
import { executePatchApplyNodeOverBridge, executePatchRestoreNodeOverBridge } from "../features/runs/agentExecutorClient";
import { loadThreadRunSnapshot, updateThreadStatusOverBridge } from "../features/threads/threadClient";
import { applyApprovedPatchForActiveRun } from "./appShellPatchActions";
import type { PatchApplyState } from "./appShellPatchActions";
import { patchApplyNodeId } from "./patchApplyApproval";
import { patchRestoreNodeId } from "./patchRestoreApproval";

vi.mock("../features/approvals/approvalClient", () => ({ proposeApprovalOverBridge: vi.fn() }));
vi.mock("../features/runs/agentExecutorClient", () => ({
  executePatchApplyNodeOverBridge: vi.fn(),
  executePatchRestoreNodeOverBridge: vi.fn(),
}));
vi.mock("../features/patches/patchClient", () => ({ loadPatchSnapshot: vi.fn() }));
vi.mock("../features/threads/threadClient", () => ({
  loadThreadRunSnapshot: vi.fn(),
  updateThreadStatusOverBridge: vi.fn(),
}));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));

const executeApply = vi.mocked(executePatchApplyNodeOverBridge);
const executeRestore = vi.mocked(executePatchRestoreNodeOverBridge);
const loadPatches = vi.mocked(loadPatchSnapshot);
const loadSnapshot = vi.mocked(loadThreadRunSnapshot);
const proposeApproval = vi.mocked(proposeApprovalOverBridge);
const updateThreadStatus = vi.mocked(updateThreadStatusOverBridge);

beforeEach(() => {
  vi.clearAllMocks();
  executeApply.mockResolvedValue({ message: "Patch applied.", patchId: "patch-1", runId: "run-1", status: "completed" });
  executeRestore.mockResolvedValue({ message: "Patch restored.", patchId: "patch-1", runId: "run-1", status: "completed" });
  loadPatches.mockResolvedValue([patch]);
  loadSnapshot.mockResolvedValue({ runs: [run], threads: [thread] });
  proposeApproval.mockImplementation(async (proposal) => ({ ...proposal, id: "prop-apply-1" }));
});

describe("applyApprovedPatchForActiveRun", () => {
  it("queues a separate apply approval before writing", async () => {
    const state = patchState();

    await applyApprovedPatchForActiveRun(state);

    expect(proposeApproval).toHaveBeenCalledWith(expect.objectContaining({
      id: "approval-patch-1-apply",
      nodeId: patchApplyNodeId(patch),
    }));
    expect(executeApply).not.toHaveBeenCalled();
    expect(state.setActionProposals).toHaveBeenCalled();
  });

  it("uses the approved apply approval id when applying", async () => {
    const approval = applyApproval("approved");
    const state = patchState({ actionProposals: [approval] });

    await applyApprovedPatchForActiveRun(state);

    expect(executeApply).toHaveBeenCalledWith(expect.objectContaining({
      approvalId: "prop-apply-1",
      proposalId: "patch-1",
    }));
    expect(updateThreadStatus).toHaveBeenCalledWith("thread-1", "testing", expect.any(String));
  });

  it("queues a separate restore approval before rollback", async () => {
    const state = patchState({ patch: appliedPatch });

    await applyApprovedPatchForActiveRun(state);

    expect(proposeApproval).toHaveBeenCalledWith(expect.objectContaining({
      id: "approval-patch-1-restore",
      nodeId: patchRestoreNodeId(appliedPatch),
    }));
    expect(executeRestore).not.toHaveBeenCalled();
    expect(state.setActionProposals).toHaveBeenCalled();
  });

  it("uses the approved restore approval id when restoring", async () => {
    const state = patchState({ actionProposals: [restoreApproval("approved")], patch: appliedPatch });

    await applyApprovedPatchForActiveRun(state);

    expect(executeRestore).toHaveBeenCalledWith(expect.objectContaining({
      approvalId: "prop-restore-1",
      proposalId: "patch-1",
    }));
    expect(updateThreadStatus).toHaveBeenCalledWith("thread-1", "reviewing", expect.any(String));
  });
});

function patchState(overrides: Partial<PatchApplyState> = {}): PatchApplyState {
  return {
    actionProposals: [],
    activeProject: project,
    activeRun: run,
    activeThread: thread,
    patch,
    setActionProposals: vi.fn(),
    setAgentRuns: vi.fn(),
    setPatches: vi.fn(),
    setThreads: vi.fn(),
    setThreadState: vi.fn(),
    ...overrides,
  };
}

function applyApproval(status: ActionProposalView["status"]): ActionProposalView {
  return {
    actionType: "edit_file",
    expectedResult: "Apply patch proposal patch-1 to disk and capture checkpoint receipts.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id: "prop-apply-1",
    nodeId: patchApplyNodeId(patch),
    rationale: "Apply one file change.",
    requiredPermission: "write_file",
    riskLabel: "high",
    rollbackPlan: "Restore checkpoint.",
    runId: "run-1",
    scope: { kind: "file", paths: ["C:/repo/src/main.ts"], projectId: "project-1", root: "C:/repo", summary: "Apply patch." },
    status,
  };
}

function restoreApproval(status: ActionProposalView["status"]): ActionProposalView {
  return {
    actionType: "edit_file",
    expectedResult: "Restore patch proposal patch-1 from checkpoint receipts.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id: "prop-restore-1",
    nodeId: patchRestoreNodeId(appliedPatch),
    rationale: "Rollback one file change.",
    requiredPermission: "write_file",
    riskLabel: "high",
    rollbackPlan: "Restore command is the rollback.",
    runId: "run-1",
    scope: { kind: "file", paths: ["C:/repo/src/main.ts"], projectId: "project-1", root: "C:/repo", summary: "Restore patch." },
    status,
  };
}

const project = {
  approvalPolicy: "manual",
  approvedRoots: ["C:/repo"],
  git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
  id: "project-1",
  indexedFiles: ["src/main.ts"],
  isolation: { detail: "none", label: "none", mode: "none" as const },
  lastOpenedLabel: "now",
  name: "Repo",
  path: "C:/repo",
  pinned: true,
  rulesFiles: [],
};

const thread = {
  activeRunId: "run-1",
  archived: false,
  createdAt: "2026-06-08T00:00:00.000Z",
  createdLabel: "now",
  goal: "Apply patch.",
  id: "thread-1",
  messages: [],
  mode: "build" as const,
  projectId: "project-1",
  runIds: ["run-1"],
  status: "building" as const,
  title: "Apply patch",
  updatedAt: "2026-06-08T00:00:00.000Z",
};

const run = {
  artifacts: [],
  createdAt: "2026-06-08T00:00:00.000Z",
  events: [],
  evidence: [],
  goal: "Apply patch.",
  id: "run-1",
  metrics: { approvalCount: 0, artifactCount: 0, commandCount: 0, eventCount: 0, evidenceCount: 0, nodeCount: 0 },
  mode: "build" as const,
  nodes: [],
  projectId: "project-1",
  status: "running" as const,
  threadId: "thread-1",
  updatedAt: "2026-06-08T00:00:00.000Z",
};

const patch: PatchProposalView = {
  approvalId: "prop-draft-1",
  checkpointFiles: [],
  files: [{
    after: "after\n",
    before: "before\n",
    diff: [{ kind: "added", text: "after" }],
    path: "C:/repo/src/main.ts",
  }],
  id: "patch-1",
  runId: "run-1",
  status: "proposed",
};

const appliedPatch: PatchProposalView = {
  ...patch,
  checkpointFiles: [{ contents: "before\n", path: "C:/repo/src/main.ts" }],
  checkpointId: "checkpoint-1",
  status: "applied",
};
