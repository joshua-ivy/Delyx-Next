import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { resumeSchedulerRun } from "./appShellSchedulerActions";
import { dispatchSchedulerDecision, type SchedulerDispatchState } from "./appShellSchedulerDispatch";
import { decideFocusApproval, shouldResumeAfterApprovalDecision } from "./focusApprovalDecision";

export async function decideApprovalAndMaybeResume(
  state: SchedulerDispatchState,
  proposalId: string,
  status: "approved" | "denied",
) {
  const decided = await decideFocusApproval(state, proposalId, status);
  if (!decided || !shouldResumeAfterApprovalDecision(decided, state.actionProposals, proposalId)) {
    return;
  }
  const decision = await resumeSchedulerRun(state);
  if (decision) {
    await dispatchSchedulerDecision(withDecidedProposal(state, decided), decision);
  }
}

function withDecidedProposal(state: SchedulerDispatchState, decided: ActionProposalView): SchedulerDispatchState {
  return {
    ...state,
    actionProposals: state.actionProposals.map((proposal) => (
      proposal.id === decided.id ? decided : proposal
    )),
  };
}
