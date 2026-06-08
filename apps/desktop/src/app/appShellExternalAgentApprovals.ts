import type { ActionProposalView } from "../features/approvals/approvalTypes";

export type CodexApprovalAction = Extract<ActionProposalView["actionType"], "external_agent" | "run_terminal">;
export type CodexPermissionMode = "read_only" | "workspace_write";

export function approvedCodexProposal(
  proposals: ActionProposalView[],
  runId: string | undefined,
  actionType: CodexApprovalAction,
  permissionMode: CodexPermissionMode,
) {
  const proposal = currentCodexProposal(proposals, runId, actionType, permissionMode);
  return proposal?.status === "approved" && !codexApprovalExpired(proposal) ? proposal : undefined;
}

export function currentCodexProposal(
  proposals: ActionProposalView[],
  runId: string | undefined,
  actionType: CodexApprovalAction,
  permissionMode: CodexPermissionMode,
) {
  if (!runId) {
    return undefined;
  }
  const nodeId = codexNodeId(runId, actionType, permissionMode);
  return proposals.find((proposal) => (
    proposal.runId === runId
    && proposal.actionType === actionType
    && proposal.nodeId === nodeId
  ));
}

export function codexApprovalExpired(proposal: ActionProposalView) {
  const expiresAt = Date.parse(proposal.expiresAt);
  return proposal.status === "expired" || !Number.isFinite(expiresAt) || expiresAt <= Date.now();
}

export function codexNodeId(
  runId: string,
  actionType: CodexApprovalAction,
  permissionMode: CodexPermissionMode,
) {
  return `${runId}-codex-${actionType}-${permissionMode}`;
}

export function codexSandbox(permissionMode: CodexPermissionMode) {
  return permissionMode === "read_only" ? "read-only" : "workspace-write";
}
