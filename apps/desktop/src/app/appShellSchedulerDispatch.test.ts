import { beforeEach, describe, expect, it, vi } from "vitest";

import type { PatchProposalView } from "../features/patches/patchTypes";
import { scheduleNextRunActionOverBridge, type AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import { recordFinalSupportForActiveThread } from "./appShellFinalAnswerActions";
import { proposeApprovedPlanPatchWithOllama } from "./appShellOllamaPatchActions";
import { applyApprovedPatchForActiveRun } from "./appShellPatchActions";
import { runReviewForActiveRun } from "./appShellReviewActions";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";
import { runTestsForActiveRun } from "./appShellTestActions";

vi.mock("./appShellFinalAnswerActions", () => ({ recordFinalSupportForActiveThread: vi.fn() }));
vi.mock("./appShellOllamaPatchActions", () => ({ proposeApprovedPlanPatchWithOllama: vi.fn() }));
vi.mock("./appShellPatchActions", () => ({ applyApprovedPatchForActiveRun: vi.fn() }));
vi.mock("./appShellReviewActions", () => ({ runReviewForActiveRun: vi.fn() }));
vi.mock("./appShellTestActions", () => ({ runTestsForActiveRun: vi.fn() }));
vi.mock("../features/runs/agentExecutorClient", () => ({ scheduleNextRunActionOverBridge: vi.fn() }));

const applyPatch = vi.mocked(applyApprovedPatchForActiveRun);
const draftPatch = vi.mocked(proposeApprovedPlanPatchWithOllama);
const recordFinal = vi.mocked(recordFinalSupportForActiveThread);
const runReview = vi.mocked(runReviewForActiveRun);
const runTests = vi.mocked(runTestsForActiveRun);
const scheduleNext = vi.mocked(scheduleNextRunActionOverBridge);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("dispatchSchedulerDecision", () => {
  it("dispatches approved patch apply decisions with the matching patch", async () => {
    const patch = patchView();
    scheduleNext.mockResolvedValue(undefined);

    const handled = await dispatchSchedulerDecision(state({ patches: [patch] }), {
      ...decision("run_patch_apply"),
      proposalId: patch.id,
    });

    expect(handled).toBe(true);
    expect(applyPatch).toHaveBeenCalledWith(expect.objectContaining({ patch }));
  });

  it("continues from patch apply into the scheduler-selected test step", async () => {
    const patch = patchView();
    scheduleNext.mockResolvedValueOnce(decision("run_tests")).mockResolvedValueOnce(undefined);

    await dispatchSchedulerDecision(state({ patches: [patch] }), {
      ...decision("run_patch_apply"),
      proposalId: patch.id,
    });

    expect(applyPatch).toHaveBeenCalledTimes(1);
    expect(runTests).toHaveBeenCalledWith(expect.objectContaining({ schedulerConfirmedRunTests: true }));
  });

  it("dispatches test, review, and final-support decisions", async () => {
    const base = state({ patches: [patchView()] });
    scheduleNext.mockResolvedValue(undefined);

    await dispatchSchedulerDecision(base, decision("run_tests"));
    await dispatchSchedulerDecision(base, decision("run_review"));
    await dispatchSchedulerDecision(base, decision("ready_for_final_support"));

    expect(runTests).toHaveBeenCalledWith(expect.objectContaining({ schedulerConfirmedRunTests: true }));
    expect(runReview).toHaveBeenCalledWith(expect.objectContaining({ patches: [patchView()] }));
    expect(recordFinal).toHaveBeenCalledWith(base);
  });

  it("dispatches scheduler-verified test approvals by id", async () => {
    scheduleNext.mockResolvedValue(undefined);

    await dispatchSchedulerDecision(state({ testReady: true }), {
      ...decision("run_tests"),
      approvalIds: ["approval-test"],
    });

    expect(runTests).toHaveBeenCalledWith(expect.objectContaining({
      schedulerConfirmedRunTests: true,
      schedulerTestApprovalId: "approval-test",
    }));
  });

  it("dispatches patch draft decisions and continues with reloaded patches", async () => {
    const patch = patchView();
    draftPatch.mockResolvedValue({
      created: true,
      patches: [patch],
      snapshot: { runs: [{ id: "run-1" }] as never, threads: [] },
    });
    scheduleNext.mockResolvedValueOnce({ ...decision("run_patch_apply"), proposalId: patch.id }).mockResolvedValueOnce(undefined);

    await dispatchSchedulerDecision(state({ draftReady: true }), {
      ...decision("run_patch_draft"),
      approvalIds: ["approval-1"],
    });

    expect(draftPatch).toHaveBeenCalledWith(expect.objectContaining({
      actionProposals: [approval()],
    }), approval());
    expect(applyPatch).toHaveBeenCalledWith(expect.objectContaining({ patch }));
  });

  it("leaves passive scheduler decisions unhandled", async () => {
    scheduleNext.mockResolvedValue(undefined);
    const handled = await dispatchSchedulerDecision(state(), decision("wait_for_approval"));

    expect(handled).toBe(false);
    expect(applyPatch).not.toHaveBeenCalled();
    expect(runTests).not.toHaveBeenCalled();
    expect(runReview).not.toHaveBeenCalled();
    expect(recordFinal).not.toHaveBeenCalled();
  });
});

function state({ draftReady = false, patches = [], testReady = false }: { draftReady?: boolean; patches?: PatchProposalView[]; testReady?: boolean } = {}) {
  const activePlan = draftReady || testReady ? plan() : undefined;
  return {
    actionProposals: draftReady ? [approval()] : testReady ? [testApproval()] : [],
    activePlan,
    activeProject: {
      approvalPolicy: "manual",
      approvedRoots: ["C:\\repo"],
      git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
      id: "project-1",
      indexedFiles: draftReady ? ["src/main.ts"] : [],
      isolation: { detail: "none", label: "none", mode: "none" as const },
      lastOpenedLabel: "now",
      name: "Project",
      path: "C:\\repo",
      pinned: true,
      rulesFiles: [],
    },
    activeRun: { id: "run-1" } as never,
    activeThread: undefined,
    modelSettings: { providers: [], routes: [], selectedProviderId: "ollama-local" },
    patches,
    reviews: [],
    setActionProposals: vi.fn(),
    setAgentRuns: vi.fn(),
    setPatches: vi.fn(),
    setReviews: vi.fn(),
    setTests: vi.fn(),
    setThreadState: vi.fn(),
    setThreads: vi.fn(),
    tests: [],
  };
}

function decision(kind: AgentScheduleDecisionView["kind"]): AgentScheduleDecisionView {
  return {
    approvalIds: [],
    kind,
    message: "Scheduler decision.",
    patchCount: 0,
    runId: "run-1",
    testCount: 0,
  };
}

function patchView(): PatchProposalView {
  return {
    approvalId: "approval-1",
    checkpointFiles: [],
    files: [],
    id: "patch-1",
    runId: "run-1",
    status: "proposed",
  };
}

function approval() {
  return {
    actionType: "edit_file" as const,
    expectedResult: "Draft a patch.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id: "approval-1",
    nodeId: "node-1",
    rationale: "Build approved plan.",
    requiredPermission: "edit_file",
    riskLabel: "high" as const,
    runId: "run-1",
    scope: { kind: "file" as const, paths: ["src/main.ts"], root: "C:\\repo", summary: "Edit src/main.ts" },
    status: "approved" as const,
  };
}

function testApproval() {
  return {
    actionType: "run_terminal" as const,
    expectedResult: "Run tests.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id: "approval-test",
    nodeId: "node-test",
    rationale: "Validate patch.",
    requiredPermission: "terminal_command",
    riskLabel: "medium" as const,
    runId: "run-1",
    scope: { commands: ["npm test"], kind: "terminal" as const, root: "C:\\repo", summary: "Run tests" },
    status: "approved" as const,
  };
}

function plan() {
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
    filesLikelyInvolved: ["src/main.ts"],
    goalUnderstanding: "Update code.",
    permissionsNeeded: ["edit_file"],
    risks: [],
    rollbackStrategy: "Restore file.",
    steps: ["Update src/main.ts"],
    testsToRun: ["npm test"],
    threadId: "thread-1",
  };
}
