import type { Dispatch, SetStateAction } from "react";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import { scheduleNextRunActionOverBridge, type AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { recordFinalSupportForActiveThread } from "./appShellFinalAnswerActions";
import { applyApprovedPatchForActiveRun } from "./appShellPatchActions";
import { runReviewForActiveRun } from "./appShellReviewActions";
import { runTestsForActiveRun } from "./appShellTestActions";
import { firstRunnableTestCommand } from "./testCommand";

const maxAutoSteps = 3;

export interface SchedulerDispatchState {
  actionProposals: ActionProposalView[];
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  patches: PatchProposalView[];
  reviews: ReviewReportView[];
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setPatches: Dispatch<SetStateAction<PatchProposalView[]>>;
  setReviews: Dispatch<SetStateAction<ReviewReportView[]>>;
  setTests: Dispatch<SetStateAction<TestArtifactView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  tests: TestArtifactView[];
}

export async function dispatchSchedulerDecision(
  state: SchedulerDispatchState,
  decision: AgentScheduleDecisionView,
  depth = 0,
) {
  const handled = await dispatchOneSchedulerDecision(state, decision);
  if (!handled || depth >= maxAutoSteps) {
    return handled;
  }
  const next = await nextSchedulerDecision(state);
  if (!next || !autoContinuable(next.kind) || repeatedDecision(decision, next)) {
    return handled;
  }
  await dispatchSchedulerDecision(state, next, depth + 1);
  return true;
}

async function dispatchOneSchedulerDecision(
  state: SchedulerDispatchState,
  decision: AgentScheduleDecisionView,
) {
  if (decision.kind === "run_patch_apply") {
    await applyApprovedPatchForActiveRun({
      activeProject: state.activeProject,
      activeThread: state.activeThread,
      patch: state.patches.find((patch) => patch.id === decision.proposalId),
      setAgentRuns: state.setAgentRuns,
      setPatches: state.setPatches,
      setThreads: state.setThreads,
      setThreadState: state.setThreadState,
    });
    return true;
  }
  if (decision.kind === "run_tests") {
    await runTestsForActiveRun({ ...state, schedulerConfirmedRunTests: true });
    return true;
  }
  if (decision.kind === "run_review") {
    await runReviewForActiveRun({
      ...state,
      patches: state.activeRun ? state.patches.filter((patch) => patch.runId === state.activeRun?.id) : [],
      schedulerConfirmedArtifacts: true,
    });
    return true;
  }
  if (decision.kind === "ready_for_final_support") {
    await recordFinalSupportForActiveThread(state);
    return true;
  }
  return false;
}

async function nextSchedulerDecision(state: SchedulerDispatchState) {
  if (!state.activeRun) {
    return undefined;
  }
  return scheduleNextRunActionOverBridge({
    hasSupportedTestCommand: Boolean(firstRunnableTestCommand(state.activePlan?.testsToRun)),
    nowMs: Date.now(),
    runId: state.activeRun.id,
  });
}

function autoContinuable(kind: AgentScheduleDecisionView["kind"]) {
  return kind === "ready_for_final_support" || kind === "run_patch_apply" || kind === "run_review" || kind === "run_tests";
}

function repeatedDecision(previous: AgentScheduleDecisionView, next: AgentScheduleDecisionView) {
  return previous.kind === next.kind && previous.proposalId === next.proposalId && previous.reviewReportId === next.reviewReportId;
}
