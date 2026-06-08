import type { Dispatch, SetStateAction } from "react";

import { executeReviewNodeOverBridge, requestReviewRevisionOverBridge } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadReviewSnapshot } from "../features/review/reviewClient";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import { loadThreadRunSnapshot, updateThreadStatusOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
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
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  reviews?: ReviewReportView[];
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setReviews: Dispatch<SetStateAction<ReviewReportView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
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
  state.setThreadState("ready");
  notifyLocalAction(result.message, "success");
}

async function reloadReviewState(state: ReviewRepairState) {
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
