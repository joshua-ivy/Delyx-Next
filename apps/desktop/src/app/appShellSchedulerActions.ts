import type { Dispatch, SetStateAction } from "react";

import { resumeWaitingRunOverBridge } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { notifyLocalAction } from "./ShellPreferenceController";

interface ResumeRunState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function resumeSchedulerRun(state: ResumeRunState) {
  if (!state.activeRun) {
    notifyLocalAction("Create a run before resuming scheduler state", "warning");
    return;
  }
  const decision = await resumeWaitingRunOverBridge({
    hasSupportedTestCommand: false,
    nowMs: Date.now(),
    runId: state.activeRun.id,
  });
  if (!decision) {
    notifyLocalAction("Desktop bridge is required to resume the run", "warning");
    return;
  }
  const snapshot = await loadThreadRunSnapshot(state.activeProject.id);
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
  state.setThreadState("ready");
  notifyLocalAction(decision.message, decision.kind === "resume_after_approval" ? "success" : "warning");
}
