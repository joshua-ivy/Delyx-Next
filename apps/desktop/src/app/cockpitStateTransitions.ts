import type { Dispatch, SetStateAction } from "react";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus } from "../features/threads/threadTypes";
import { expirePendingProposalsForRun } from "./appShellApprovalActions";
import { updateRunsForThreadStatus } from "./appShellRunActions";
import { modeForThreadStatus } from "./appShellThreadActions";
import { updateThreadStatusOverBridge } from "../features/threads/threadClient";

export interface CockpitTransitionState {
  activeRun: AgentRunView | undefined;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
}

export function expireRunProposals(state: CockpitTransitionState, activeThread: TaskThread) {
  const runId = state.activeRun?.id ?? activeThread.activeRunId;
  if (runId) {
    state.setActionProposals((current) => expirePendingProposalsForRun(current, runId));
  }
}

export function updateThreadAndRunStatus(
  state: CockpitTransitionState,
  activeThread: TaskThread,
  status: ThreadStatus,
) {
  const now = new Date().toISOString();
  void updateThreadStatusOverBridge(activeThread.id, status, now);
  state.setAgentRuns((current) => updateRunsForThreadStatus(current, activeThread, status, now));
  state.setThreads((current) => current.map((thread) => (
    thread.id === activeThread.id ? { ...thread, mode: modeForThreadStatus(status), status, updatedAt: now } : thread
  )));
}
