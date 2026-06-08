import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { firstRunnableTestCommand, type RunnableTestCommand } from "./testCommand";

export interface TestApprovalDecisionState {
  actionProposals: ActionProposalView[];
  activePlan: PlanView | undefined;
  activeRun: AgentRunView | undefined;
}

export function activeTestApprovalId(state: TestApprovalDecisionState) {
  const command = firstRunnableTestCommand(state.activePlan?.testsToRun);
  return command ? activeTestApproval(state, command)?.id : undefined;
}

export function activeTestApproval(
  state: TestApprovalDecisionState,
  command: RunnableTestCommand,
  preferredApprovalId?: string,
) {
  const now = Date.now();
  return state.actionProposals.find((proposal) => (
    proposal.runId === state.activeRun?.id
    && proposal.actionType === "run_terminal"
    && (!preferredApprovalId || proposal.id === preferredApprovalId)
    && proposal.scope.commands?.includes(command.label)
    && proposal.status === "approved"
    && Date.parse(proposal.expiresAt) > now
  ));
}
