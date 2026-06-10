import { decideApprovalAndMaybeResume, resumeAndDispatchSchedulerRun } from "./appShellApprovalDecisionActions";
import { recordFinalSupportForActiveThread } from "./appShellFinalAnswerActions";
import { applyApprovedPatchForActiveRun } from "./appShellPatchActions";
import { requestRepairForReviewFinding, runReviewForActiveRun } from "./appShellReviewActions";
import type { SchedulerDispatchState } from "./appShellSchedulerDispatch";
import { runTestsForActiveRun } from "./appShellTestActions";

export function buildFocusRunHandlers(state: SchedulerDispatchState) {
  const { activeRun, patches } = state;
  const runReview = () => {
    void runReviewForActiveRun({
      ...state,
      patches: activeRun ? patches.filter((patch) => patch.runId === activeRun.id) : [],
    });
  };
  const runTests = () => {
    void runTestsForActiveRun(state);
  };
  const recordFinal = () => {
    void recordFinalSupportForActiveThread(state);
  };
  const requestRepair = (reportId: string, findingId: string) => {
    void requestRepairForReviewFinding(state, reportId, findingId);
  };
  const applyPatch = (patchId: string) => {
    void applyApprovedPatchForActiveRun({
      ...state,
      patch: patches.find((patch) => patch.id === patchId),
    });
  };
  const decideProposal = (proposalId: string, status: "approved" | "denied") => {
    void decideApprovalAndMaybeResume(state, proposalId, status);
  };
  const resumeRun = () => {
    void resumeAndDispatchSchedulerRun(state);
  };
  return {
    applyPatch,
    decideProposal,
    recordFinal,
    requestRepair,
    resumeRun,
    runReview,
    runTests,
  };
}
