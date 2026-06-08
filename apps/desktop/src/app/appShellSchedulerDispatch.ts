import type { Dispatch, SetStateAction } from "react";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import { loadPatchSnapshot } from "../features/patches/patchClient";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import { loadReviewSnapshot } from "../features/review/reviewClient";
import type { ReviewReportView } from "../features/review/reviewTypes";
import {
  runPatchApplySchedulerStepOverBridge,
  scheduleNextRunActionOverBridge,
  type AgentScheduleDecisionView,
} from "../features/runs/agentExecutorClient";
import {
  runReviewSchedulerStepOverBridge,
  runTestSchedulerStepOverBridge,
} from "../features/runs/agentSchedulerStepClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadTestSnapshot } from "../features/tests/testClient";
import type { TestArtifactView } from "../features/tests/testTypes";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { recordFinalSupportForActiveThread } from "./appShellFinalAnswerActions";
import { proposeApprovedPlanPatchWithOllama } from "./appShellOllamaPatchActions";
import { applyApprovedPatchForActiveRun } from "./appShellPatchActions";
import { runTestsForActiveRun } from "./appShellTestActions";
import { patchApplyApprovalIdForScheduler } from "./patchApplyApproval";
import { notifyLocalAction } from "./ShellPreferenceController";

const maxAutoSteps = 4;

export interface SchedulerDispatchState {
  actionProposals: ActionProposalView[];
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  modelSettings: ModelSettingsView;
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
  const result = await dispatchOneSchedulerDecision(state, decision);
  if (!result.handled || depth >= maxAutoSteps) {
    return result.handled;
  }
  const nextState = result.nextState ?? state;
  const next = await nextSchedulerDecision(nextState);
  if (!next || !autoContinuable(next.kind) || repeatedDecision(decision, next)) {
    return result.handled;
  }
  await dispatchSchedulerDecision(nextState, next, depth + 1);
  return true;
}

async function dispatchOneSchedulerDecision(
  state: SchedulerDispatchState,
  decision: AgentScheduleDecisionView,
) {
  if (decision.kind === "run_patch_apply") {
    const result = await runSchedulerPatchApplyStep(state);
    return { handled: true, nextState: applyReloadedState(state, result) };
  }
  if (decision.kind === "request_patch_apply_approval") {
    await applyApprovedPatchForActiveRun({
      actionProposals: state.actionProposals,
      activeProject: state.activeProject,
      activeRun: state.activeRun,
      activeThread: state.activeThread,
      patch: state.patches.find((patch) => patch.id === decision.proposalId),
      setActionProposals: state.setActionProposals,
      setAgentRuns: state.setAgentRuns,
      setPatches: state.setPatches,
      setThreads: state.setThreads,
      setThreadState: state.setThreadState,
    });
    return { handled: true };
  }
  if (decision.kind === "run_patch_draft") {
    const result = await proposeApprovedPlanPatchWithOllama(state);
    return {
      handled: true,
      nextState: {
        ...state,
        activeRun: result?.snapshot?.runs.find((run) => run.id === state.activeRun?.id) ?? state.activeRun,
        patches: result?.patches ?? state.patches,
      },
    };
  }
  if (decision.kind === "run_tests") {
    if (!decision.approvalIds[0]) {
      await runTestsForActiveRun({
        ...state,
        schedulerConfirmedRunTests: true,
      });
      return { handled: true };
    }
    const result = await runSchedulerTestStep(state);
    return { handled: true, nextState: testReloadedState(state, result) };
  }
  if (decision.kind === "run_review") {
    const result = await runSchedulerReviewStep(state);
    return { handled: true, nextState: reviewReloadedState(state, result) };
  }
  if (decision.kind === "ready_for_final_support") {
    await recordFinalSupportForActiveThread(state);
    return { handled: true };
  }
  return { handled: false };
}

async function runSchedulerPatchApplyStep(state: SchedulerDispatchState) {
  if (!state.activeRun) {
    notifyLocalAction("Create a run before applying a scheduler-selected patch", "warning");
    return undefined;
  }
  const now = new Date();
  const result = await runPatchApplySchedulerStepOverBridge({
    nowMs: now.getTime(),
    runId: state.activeRun.id,
    updatedAt: now.toISOString(),
  });
  if (!result) {
    notifyLocalAction("Desktop bridge is required to apply scheduler-selected patches", "warning");
    return undefined;
  }
  const [patches, snapshot] = await Promise.all([
    loadPatchSnapshot(state.activeRun.id),
    loadThreadRunSnapshot(state.activeProject.id),
  ]);
  if (patches) {
    state.setPatches(patches);
  }
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
  state.setThreadState("ready");
  notifyLocalAction(result.message, result.status === "completed" ? "success" : "warning");
  return { patches, snapshot };
}

function applyReloadedState(
  state: SchedulerDispatchState,
  result: Awaited<ReturnType<typeof runSchedulerPatchApplyStep>>,
) {
  return {
    ...state,
    activeRun: result?.snapshot?.runs.find((run) => run.id === state.activeRun?.id) ?? state.activeRun,
    patches: result?.patches ?? state.patches,
  };
}

async function runSchedulerTestStep(state: SchedulerDispatchState) {
  if (!state.activeRun) {
    notifyLocalAction("Create a run before running scheduler-selected tests", "warning");
    return undefined;
  }
  const started = new Date();
  const result = await runTestSchedulerStepOverBridge({
    nowMs: started.getTime(),
    runId: state.activeRun.id,
    startedAt: started.toISOString(),
    timeoutMs: 5 * 60 * 1000,
    updatedAt: new Date().toISOString(),
  });
  if (!result) {
    notifyLocalAction("Desktop bridge is required to run scheduler-selected tests", "warning");
    return undefined;
  }
  const [tests, snapshot] = await Promise.all([
    loadTestSnapshot(state.activeRun.id),
    loadThreadRunSnapshot(state.activeProject.id),
  ]);
  if (tests) {
    state.setTests(tests);
  }
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
  state.setThreadState("ready");
  notifyLocalAction(result.message, result.status === "completed" ? "success" : "warning");
  return { snapshot, tests };
}

function testReloadedState(
  state: SchedulerDispatchState,
  result: Awaited<ReturnType<typeof runSchedulerTestStep>>,
) {
  return {
    ...state,
    activeRun: result?.snapshot?.runs.find((run) => run.id === state.activeRun?.id) ?? state.activeRun,
    tests: result?.tests ?? state.tests,
  };
}

async function runSchedulerReviewStep(state: SchedulerDispatchState) {
  if (!state.activeRun) {
    notifyLocalAction("Create a run before reviewing scheduler-selected artifacts", "warning");
    return undefined;
  }
  const now = new Date();
  const result = await runReviewSchedulerStepOverBridge({
    nowMs: now.getTime(),
    runId: state.activeRun.id,
    updatedAt: now.toISOString(),
  });
  if (!result) {
    notifyLocalAction("Desktop bridge is required to review scheduler-selected artifacts", "warning");
    return undefined;
  }
  const [reviews, snapshot] = await Promise.all([
    loadReviewSnapshot(state.activeRun.id),
    loadThreadRunSnapshot(state.activeProject.id),
  ]);
  if (reviews) {
    state.setReviews((current) => [
      ...reviews,
      ...current.filter((report) => report.runId !== state.activeRun?.id),
    ]);
  }
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
  state.setThreadState("ready");
  notifyLocalAction(result.message, result.status === "completed" ? "success" : "warning");
  return { reviews, snapshot };
}

function reviewReloadedState(
  state: SchedulerDispatchState,
  result: Awaited<ReturnType<typeof runSchedulerReviewStep>>,
) {
  return {
    ...state,
    activeRun: result?.snapshot?.runs.find((run) => run.id === state.activeRun?.id) ?? state.activeRun,
    reviews: result?.reviews ?? state.reviews,
  };
}

async function nextSchedulerDecision(state: SchedulerDispatchState) {
  if (!state.activeRun) {
    return undefined;
  }
  return scheduleNextRunActionOverBridge({
    hasSupportedTestCommand: false,
    nowMs: Date.now(),
    patchApplyApprovalId: patchApplyApprovalIdForScheduler(state.actionProposals, state.patches),
    runId: state.activeRun.id,
  });
}

function autoContinuable(kind: AgentScheduleDecisionView["kind"]) {
  return kind === "ready_for_final_support" || kind === "run_patch_apply" || kind === "run_patch_draft" || kind === "run_review" || kind === "run_tests";
}

function repeatedDecision(previous: AgentScheduleDecisionView, next: AgentScheduleDecisionView) {
  return previous.kind === next.kind && previous.proposalId === next.proposalId && previous.reviewReportId === next.reviewReportId;
}
