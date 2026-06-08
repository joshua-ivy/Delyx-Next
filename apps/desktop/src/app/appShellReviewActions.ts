import type { Dispatch, SetStateAction } from "react";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { executeReviewNodeOverBridge, requestReviewRevisionOverBridge } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadReviewSnapshot } from "../features/review/reviewClient";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import { loadThreadRunSnapshot, updateThreadStatusOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { upsertActionProposal } from "./appShellApprovalActions";
import { createRepairPatchDraftApprovalProposal } from "./appShellPatchDraftDecision";
import { recordApprovalProposalForRun } from "./appShellRunActions";
import { updateThreadAndRunStatus } from "./cockpitStateTransitions";
import { notifyLocalAction } from "./ShellPreferenceController";

export interface ReviewRunState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  patches: PatchProposalView[];
  reviews?: ReviewReportView[];
  schedulerConfirmedArtifacts?: boolean;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setReviews: Dispatch<SetStateAction<ReviewReportView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  tests: TestArtifactView[];
}

export interface ReviewRepairState {
  actionProposals: ActionProposalView[];
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  reviews?: ReviewReportView[];
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setReviews: Dispatch<SetStateAction<ReviewReportView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

interface ReviewReloadState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setReviews: Dispatch<SetStateAction<ReviewReportView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
}

export async function runReviewForActiveRun(state: ReviewRunState) {
  if (!state.activeRun || !state.activeThread) {
    notifyLocalAction("Create a thread with a run before review", "warning");
    return;
  }
  if (!state.schedulerConfirmedArtifacts && state.patches.length === 0 && state.tests.length === 0) {
    notifyLocalAction("Review needs a real patch or test artifact first", "warning");
    return;
  }

  const result = await executeReviewNodeOverBridge(state.activeRun.id);
  if (!result) {
    notifyLocalAction("Desktop bridge is required to run artifact review", "warning");
    return;
  }

  const now = new Date().toISOString();
  if (result.status === "completed") {
    await updateThreadStatusOverBridge(state.activeThread.id, "reviewing", now);
  }
  await reloadReviewState(state);
  state.setThreadState("ready");
  notifyLocalAction(result.message, result.status === "completed" ? "success" : "warning");
}

export async function requestRepairForReviewFinding(
  state: ReviewRepairState,
  reportId: string,
  findingId: string,
) {
  if (!state.activeRun || !state.activeThread) {
    notifyLocalAction("Create a thread with a run before requesting repair", "warning");
    return;
  }
  const report = state.reviews?.find((item) => item.id === reportId && item.runId === state.activeRun?.id);
  const finding = report?.findings.find((item) => item.id === findingId);
  if (!report || !finding) {
    notifyLocalAction("Select a real review finding before requesting repair", "warning");
    return;
  }
  const result = await requestReviewRevisionOverBridge(state.activeRun.id, report.id, finding.id);
  if (!result) {
    notifyLocalAction("Desktop bridge is required to request review repair", "warning");
    return;
  }
  await reloadReviewState(state);
  const queued = await queueRepairApproval(state, report, finding);
  state.setThreadState("ready");
  notifyLocalAction(
    queued ? "Repair requested; approve the scoped repair patch draft" : result.message,
    "success",
  );
}

async function queueRepairApproval(
  state: ReviewRepairState,
  report: ReviewReportView,
  finding: ReviewReportView["findings"][number],
) {
  if (!state.activeThread) {
    return false;
  }
  const fallback = createRepairPatchDraftApprovalProposal(state, report, finding);
  if (!fallback || state.actionProposals.some((proposal) => proposal.id === fallback.id && proposal.status !== "expired")) {
    return false;
  }
  const proposal = await proposeApprovalOverBridge(fallback) ?? fallback;
  const now = new Date().toISOString();
  state.setActionProposals((current) => upsertActionProposal(current, proposal));
  state.setAgentRuns((current) => recordApprovalProposalForRun(current, state.activeThread!, proposal, now));
  updateThreadAndRunStatus(state, state.activeThread, "waiting_for_approval");
  return true;
}

async function reloadReviewState(state: ReviewReloadState) {
  const [reports, snapshot] = await Promise.all([
    loadReviewSnapshot(state.activeRun?.id ?? ""),
    loadThreadRunSnapshot(state.activeProject.id),
  ]);
  if (reports) {
    state.setReviews((current) => [
      ...reports,
      ...current.filter((report) => report.runId !== state.activeRun?.id),
    ]);
  }
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
}
