import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { resumeSchedulerRun } from "./appShellSchedulerActions";
import { dispatchSchedulerDecision, type SchedulerDispatchState } from "./appShellSchedulerDispatch";
import type { OllamaPatchProposalState } from "./appShellOllamaPatchActions";
import { decideFocusApproval, shouldResumeAfterApprovalDecision } from "./focusApprovalDecision";

export async function decideApprovalAndMaybeResume(
  state: OllamaPatchProposalState,
  proposalId: string,
  status: "approved" | "denied",
) {
  const decided = await decideFocusApproval(state, proposalId, status);
  if (!decided || !shouldResumeAfterApprovalDecision(decided, state.actionProposals, proposalId)) {
    return;
  }
  const decidedState = withDecidedProposal(state, decided);
  const decision = await resumeSchedulerRun(decidedState);
  if (decision) {
    await dispatchSchedulerDecision(decidedState, decision);
  }
}

function withDecidedProposal(
  state: OllamaPatchProposalState,
  decided: ActionProposalView,
): OllamaPatchProposalState & SchedulerDispatchState {
  return {
    ...state,
    actionProposals: state.actionProposals.map((proposal) => (
      proposal.id === decided.id ? decided : proposal
    )),
  };
}
