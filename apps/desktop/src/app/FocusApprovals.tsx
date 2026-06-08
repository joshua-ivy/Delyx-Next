import type { ActionProposalView, ProposalStatus } from "../features/approvals/approvalTypes";
import { FocusIcon } from "./focusAtoms";

export function visibleApprovalProposals(proposals: ActionProposalView[]) {
  return proposals.filter((proposal) => proposal.status !== "approved");
}

export function FocusApprovalBlock({
  onDecideProposal,
  proposals,
}: {
  onDecideProposal: (proposalId: string, status: "approved" | "denied") => void;
  proposals: ActionProposalView[];
}) {
  if (proposals.length === 0) {
    return null;
  }
  return <>{proposals.map((proposal) => (
    <div className="plan approval-focus" key={proposal.id}>
      <div className="plan-head">
        <span className="ey">Approval / {proposal.riskLabel} risk / {proposal.status}</span>
        <FocusIcon name="shield" />
      </div>
      <div className="approval-copy">
        <b>{proposal.actionType}</b>
        <span>{proposal.rationale}</span>
        <span>{proposal.expectedResult}</span>
        {proposal.status !== "pending" && <span>{approvalStateText(proposal.status)}</span>}
      </div>
      {proposal.status === "pending" && (
        <div className="plan-actions">
          <button className="select" onClick={() => onDecideProposal(proposal.id, "approved")} type="button">Approve once</button>
          <button className="select danger" onClick={() => onDecideProposal(proposal.id, "denied")} type="button">Deny</button>
        </div>
      )}
    </div>
  ))}</>;
}

function approvalStateText(status: ProposalStatus) {
  if (status === "denied") {
    return "Denied; Delyx will not execute this action.";
  }
  if (status === "expired") {
    return "Expired; request a fresh approval before this can run.";
  }
  return "Approval recorded.";
}
