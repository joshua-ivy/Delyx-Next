import type { Dispatch, SetStateAction } from "react";

import { executeReviewNodeOverBridge } from "../features/runs/agentExecutorClient";
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
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setReviews: Dispatch<SetStateAction<ReviewReportView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  tests: TestArtifactView[];
}

export async function runReviewForActiveRun(state: ReviewRunState) {
  if (!state.activeRun || !state.activeThread) {
    notifyLocalAction("Create a thread with a run before review", "warning");
    return;
  }
  if (state.patches.length === 0 && state.tests.length === 0) {
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

async function reloadReviewState(state: ReviewRunState) {
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
