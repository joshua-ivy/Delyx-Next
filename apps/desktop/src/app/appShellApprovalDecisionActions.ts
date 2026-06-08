import type { Dispatch, SetStateAction } from "react";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { resumeSchedulerRun } from "./appShellSchedulerActions";
import { decideFocusApproval, shouldResumeAfterApprovalDecision } from "./focusApprovalDecision";

interface ApprovalDecisionActionState {
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  actionProposals: ActionProposalView[];
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function decideApprovalAndMaybeResume(
  state: ApprovalDecisionActionState,
  proposalId: string,
  status: "approved" | "denied",
) {
  const decided = await decideFocusApproval(state, proposalId, status);
  if (!decided || !shouldResumeAfterApprovalDecision(decided, state.actionProposals, proposalId)) {
    return;
  }
  await resumeSchedulerRun(state);
}
