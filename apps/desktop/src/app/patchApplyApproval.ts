import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export function patchApplyApprovalId(patchId: string) {
  return `approval-${patchId}-apply`;
}

export function patchApplyNodeId(patch: PatchProposalView) {
  return `${patch.runId}-patch-apply-${patch.id}`;
}

export function activePatchApplyApproval(
  proposals: ActionProposalView[],
  patch: PatchProposalView,
) {
  const nodeId = patchApplyNodeId(patch);
  const matches = proposals.filter((proposal) => proposal.id === patchApplyApprovalId(patch.id) || proposal.nodeId === nodeId);
  return matches.find((proposal) => proposal.status !== "expired") ?? matches[0];
}

export function createPatchApplyApprovalProposal(
  patch: PatchProposalView,
  run: AgentRunView | undefined,
  project: WorkspaceProject,
): ActionProposalView {
  const paths = patch.files.map((file) => file.path);
  return {
    actionType: "edit_file",
    expectedResult: `Apply patch proposal ${patch.id} to disk and capture checkpoint receipts.`,
    expiresAt: new Date(Date.now() + 30 * 60 * 1000).toISOString(),
    id: patchApplyApprovalId(patch.id),
    nodeId: patchApplyNodeId(patch),
    rationale: `Apply ${patch.files.length} file change(s) from proposed patch ${patch.id}.`,
    requiredPermission: "write_file",
    riskLabel: "high",
    rollbackPlan: "Use checkpoint receipts to restore changed files if review rejects the diff.",
    runId: run?.id ?? patch.runId,
    scope: {
      kind: "file",
      paths,
      projectId: project.id,
      root: project.approvedRoots[0],
      summary: "Apply one proposed patch inside the approved project root.",
    },
    status: "pending",
  };
}
