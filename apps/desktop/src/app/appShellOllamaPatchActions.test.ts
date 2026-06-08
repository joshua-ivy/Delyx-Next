import { beforeEach, describe, expect, it, vi } from "vitest";

import { selectedOllamaModel } from "../features/models/ollamaClient";
import { loadPatchSnapshot } from "../features/patches/patchClient";
import type { PatchProposalView } from "../features/patches/patchTypes";
import { executePatchDraftNodeOverBridge, scheduleNextRunActionOverBridge } from "../features/runs/agentExecutorClient";
import { appendThreadMessageOverBridge, loadThreadRunSnapshot } from "../features/threads/threadClient";
import { proposeApprovedPlanPatchWithOllama, type OllamaPatchProposalState } from "./appShellOllamaPatchActions";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";

vi.mock("../features/models/ollamaClient", () => ({
  selectedOllamaModel: vi.fn(),
}));

vi.mock("../features/runs/agentExecutorClient", () => ({
  executePatchDraftNodeOverBridge: vi.fn(),
  scheduleNextRunActionOverBridge: vi.fn(),
}));

vi.mock("../features/patches/patchClient", () => ({
  loadPatchSnapshot: vi.fn(),
}));

vi.mock("../features/threads/threadClient", () => ({
  appendThreadMessageOverBridge: vi.fn(),
  loadThreadRunSnapshot: vi.fn(),
}));

vi.mock("./ShellPreferenceController", () => ({
  notifyLocalAction: vi.fn(),
}));

vi.mock("./appShellSchedulerDispatch", () => ({
  dispatchSchedulerDecision: vi.fn(),
}));

const dispatchDecision = vi.mocked(dispatchSchedulerDecision);
const executePatchDraft = vi.mocked(executePatchDraftNodeOverBridge);
const loadPatches = vi.mocked(loadPatchSnapshot);
const loadSnapshot = vi.mocked(loadThreadRunSnapshot);
const model = vi.mocked(selectedOllamaModel);
const scheduleNext = vi.mocked(scheduleNextRunActionOverBridge);

beforeEach(() => {
  vi.clearAllMocks();
  model.mockReturnValue("qwen3-coder:30b");
  executePatchDraft.mockResolvedValue({
    message: "Patch proposal patch-1 captured.",
    model: "qwen3-coder:30b",
    patchId: "patch-1",
    providerId: "ollama-local",
    runId: "run-1",
    status: "completed",
  });
  loadPatches.mockResolvedValue([patch]);
  loadSnapshot.mockResolvedValue({ runs: [run], threads: [thread] });
  scheduleNext.mockResolvedValue({
    approvalIds: [],
    kind: "run_patch_apply",
    message: "Patch is ready to apply.",
    patchCount: 1,
    proposalId: "patch-1",
    runId: "run-1",
    testCount: 0,
  });
});

describe("proposeApprovedPlanPatchWithOllama", () => {
  it("turns an approved plan into a proposed patch artifact", async () => {
    const state = actionState();

    const created = await proposeApprovedPlanPatchWithOllama(state, approval);

    expect(created).toBe(true);
    expect(executePatchDraft).toHaveBeenCalledWith(expect.objectContaining({
      approvalId: "approval-1",
      approvedRoots: ["C:/repo"],
      clientId: "patch-run-1-approval-1",
      filesLikelyInvolved: ["src/main.ts"],
      goal: "Update value.",
      model: "qwen3-coder:30b",
      planSteps: ["Update value"],
      projectPath: "C:/repo",
      runId: "run-1",
      scopePaths: ["src/main.ts"],
    }));
    expect(state.setPatches).toHaveBeenCalledWith([patch]);
    expect(scheduleNext).toHaveBeenCalledWith(expect.objectContaining({
      hasSupportedTestCommand: true,
      runId: "run-1",
    }));
    expect(dispatchDecision).toHaveBeenCalledWith(expect.objectContaining({
      activeRun: run,
      patches: [patch],
    }), expect.objectContaining({ kind: "run_patch_apply", proposalId: "patch-1" }));
    expect(appendThreadMessageOverBridge).toHaveBeenCalled();
  });

  it("skips drafting when the run already has a patch", async () => {
    const state = actionState({ patches: [patch] });

    const created = await proposeApprovedPlanPatchWithOllama(state, approval);

    expect(created).toBe(false);
    expect(executePatchDraft).not.toHaveBeenCalled();
    expect(scheduleNext).not.toHaveBeenCalled();
  });
});

function actionState(overrides: Partial<OllamaPatchProposalState> = {}): OllamaPatchProposalState {
  return {
    actionProposals: [approval],
    activePlan: plan,
    activeProject: project,
    activeRun: run,
    activeThread: thread,
    modelSettings: { providers: [], routes: [], selectedProviderId: "ollama-local" },
    patches: [],
    reviews: [],
    setActionProposals: vi.fn(),
    setAgentRuns: vi.fn(),
    setPatches: vi.fn(),
    setReviews: vi.fn(),
    setTests: vi.fn(),
    setThreads: vi.fn(),
    setThreadState: vi.fn(),
    tests: [],
    ...overrides,
  } as OllamaPatchProposalState;
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

const plan = {
  decision: "approved" as const,
  explore: {
    architectureSummary: "TypeScript project.",
    projectCommands: ["npm test"],
    relevantFiles: ["src/main.ts"],
    relevantSymbols: [],
    risks: [],
    suggestedNextSteps: [],
    unknowns: [],
  },
  filesLikelyInvolved: ["src/main.ts"],
  goalUnderstanding: "Update value.",
  permissionsNeeded: ["edit_file"],
  risks: [],
  rollbackStrategy: "Restore the previous contents.",
  steps: ["Update value"],
  testsToRun: ["npm test"],
  threadId: "thread-1",
};

const thread = {
  activeRunId: "run-1",
  archived: false,
  createdAt: "2026-06-08T00:00:00.000Z",
  createdLabel: "now",
  goal: "Update value.",
  id: "thread-1",
  messages: [],
  mode: "build" as const,
  projectId: "project-1",
  runIds: ["run-1"],
  status: "building" as const,
  title: "Update value",
  updatedAt: "2026-06-08T00:00:00.000Z",
};

const run = {
  artifacts: [],
  createdAt: "2026-06-08T00:00:00.000Z",
  events: [],
  evidence: [],
  goal: "Update value.",
  id: "run-1",
  metrics: { approvalCount: 0, artifactCount: 0, commandCount: 0, eventCount: 0, evidenceCount: 0, nodeCount: 0 },
  mode: "build" as const,
  nodes: [],
  projectId: "project-1",
  status: "running" as const,
  threadId: "thread-1",
  updatedAt: "2026-06-08T00:00:00.000Z",
};

const approval = {
  actionType: "edit_file" as const,
  expectedResult: "Propose a patch.",
  expiresAt: "2999-01-01T00:00:00.000Z",
  id: "approval-1",
  nodeId: "node-1",
  rationale: "Update value.",
  requiredPermission: "edit_file",
  riskLabel: "high" as const,
  rollbackPlan: "Restore previous contents.",
  runId: "run-1",
  scope: { kind: "file" as const, paths: ["src/main.ts"], root: "C:/repo", summary: "Edit src/main.ts" },
  status: "approved" as const,
};

const patch: PatchProposalView = {
  approvalId: "approval-1",
  checkpointFiles: [],
  checkpointId: undefined,
  files: [],
  id: "patch-1",
  restoreApprovalId: undefined,
  runId: "run-1",
  status: "proposed" as const,
};
