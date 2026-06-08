import { beforeEach, describe, expect, it, vi } from "vitest";

import type { PatchProposalView } from "../features/patches/patchTypes";
import { scheduleNextRunActionOverBridge, type AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import { recordFinalSupportForActiveThread } from "./appShellFinalAnswerActions";
import { applyApprovedPatchForActiveRun } from "./appShellPatchActions";
import { runReviewForActiveRun } from "./appShellReviewActions";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";
import { runTestsForActiveRun } from "./appShellTestActions";

vi.mock("./appShellFinalAnswerActions", () => ({ recordFinalSupportForActiveThread: vi.fn() }));
vi.mock("./appShellPatchActions", () => ({ applyApprovedPatchForActiveRun: vi.fn() }));
vi.mock("./appShellReviewActions", () => ({ runReviewForActiveRun: vi.fn() }));
vi.mock("./appShellTestActions", () => ({ runTestsForActiveRun: vi.fn() }));
vi.mock("../features/runs/agentExecutorClient", () => ({ scheduleNextRunActionOverBridge: vi.fn() }));

const applyPatch = vi.mocked(applyApprovedPatchForActiveRun);
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

function state({ patches = [] }: { patches?: PatchProposalView[] } = {}) {
  return {
    actionProposals: [],
    activePlan: undefined,
    activeProject: {
      approvalPolicy: "manual",
      approvedRoots: ["C:\\repo"],
      git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
      id: "project-1",
      indexedFiles: [],
      isolation: { detail: "none", label: "none", mode: "none" as const },
      lastOpenedLabel: "now",
      name: "Project",
      path: "C:\\repo",
      pinned: true,
      rulesFiles: [],
    },
    activeRun: { id: "run-1" } as never,
    activeThread: undefined,
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
