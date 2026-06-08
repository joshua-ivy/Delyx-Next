import type { Dispatch, SetStateAction } from "react";
import { decideApprovalOverBridge } from "../features/approvals/approvalClient";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import { recordApprovalDecisionForRun } from "./appShellRunActions";
import { updateThreadAndRunStatus } from "./cockpitStateTransitions";
import { notifyLocalAction } from "./ShellPreferenceController";

interface ApprovalDecisionState {
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  actionProposals: ActionProposalView[];
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function decideFocusApproval(
  state: ApprovalDecisionState,
  proposalId: string,
  status: "approved" | "denied",
): Promise<ActionProposalView | undefined> {
  const proposal = state.actionProposals.find((item) => item.id === proposalId);
  if (!proposal) {
    notifyLocalAction("Approval proposal is no longer available", "warning");
    return;
  }
  if (proposal.status !== "pending") {
    notifyLocalAction("Approval decision was already recorded", "warning");
    return;
  }
  if (isExpired(proposal.expiresAt)) {
    state.setActionProposals((current) => current.map((item) => (
      item.id === proposalId ? { ...item, status: "expired" } : item
    )));
    notifyLocalAction("Approval proposal is expired; request a fresh approval", "warning");
    return;
  }
  const decidedAtMs = Date.now();
  const decided = await decideApprovalOverBridge(proposalId, status, decidedAtMs) ?? { ...proposal, status };
  state.setActionProposals((current) => current.map((item) => (
    item.id === proposalId ? decided : item
  )));
  if (state.activeThread && decided.status !== "expired") {
    state.setAgentRuns((current) => recordApprovalDecisionForRun(
      current,
      state.activeThread as TaskThread,
      decided,
      new Date(decidedAtMs).toISOString(),
    ));
    updateThreadAndRunStatus(
      state,
      state.activeThread,
      threadStatusAfterDecision(decided, state.actionProposals, proposalId),
    );
  }
  notifyLocalAction(
    decisionMessage(decided, state.actionProposals, proposalId),
    decided.status === "approved" ? "success" : "warning",
  );
  return decided;
}

function isExpired(expiresAt: string) {
  const deadline = Date.parse(expiresAt);
  return Number.isFinite(deadline) && deadline <= Date.now();
}

function threadStatusAfterDecision(
  decided: ActionProposalView,
  proposals: ActionProposalView[],
  decidedId: string,
): ThreadStatus {
  if (decided.status !== "approved") {
    return "blocked";
  }
  return hasOtherPendingApproval(decided, proposals, decidedId) ? "waiting_for_approval" : "planning";
}

function decisionMessage(
  decided: ActionProposalView,
  proposals: ActionProposalView[],
  decidedId: string,
) {
  if (decided.status !== "approved") {
    return "Approval denied; run blocked";
  }
  return hasOtherPendingApproval(decided, proposals, decidedId)
    ? "Approval recorded; more approvals are still pending"
    : "Approval granted; waiting for the next executable step";
}

export function shouldResumeAfterApprovalDecision(
  decided: ActionProposalView,
  proposals: ActionProposalView[],
  decidedId: string,
) {
  return decided.status === "approved" && !hasOtherPendingApproval(decided, proposals, decidedId);
}

function hasOtherPendingApproval(
  decided: ActionProposalView,
  proposals: ActionProposalView[],
  decidedId: string,
) {
  return proposals.some((proposal) => (
    proposal.id !== decidedId
    && proposal.runId === decided.runId
    && proposal.status === "pending"
    && !isExpired(proposal.expiresAt)
  ));
}
