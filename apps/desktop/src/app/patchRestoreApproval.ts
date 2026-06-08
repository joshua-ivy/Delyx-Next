import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export function patchRestoreApprovalId(patchId: string) {
  return `approval-${patchId}-restore`;
}

export function patchRestoreNodeId(patch: PatchProposalView) {
  return `${patch.runId}-patch-restore-${patch.id}`;
}

export function activePatchRestoreApproval(
  proposals: ActionProposalView[],
  patch: PatchProposalView,
) {
  const nodeId = patchRestoreNodeId(patch);
  const matches = proposals.filter((proposal) => proposal.id === patchRestoreApprovalId(patch.id) || proposal.nodeId === nodeId);
  return matches.find((proposal) => proposal.status !== "expired") ?? matches[0];
}

export function createPatchRestoreApprovalProposal(
  patch: PatchProposalView,
  run: AgentRunView | undefined,
  project: WorkspaceProject,
): ActionProposalView {
  return {
    actionType: "edit_file",
    expectedResult: `Restore patch proposal ${patch.id} from checkpoint receipts.`,
    expiresAt: new Date(Date.now() + 30 * 60 * 1000).toISOString(),
    id: patchRestoreApprovalId(patch.id),
    nodeId: patchRestoreNodeId(patch),
    rationale: `Rollback ${patch.files.length} file change(s) from applied patch ${patch.id}.`,
    requiredPermission: "write_file",
    riskLabel: "high",
    rollbackPlan: "The restore action is the rollback; inspect the restored diff before continuing.",
    runId: run?.id ?? patch.runId,
    scope: {
      kind: "file",
      paths: patch.files.map((file) => file.path),
      projectId: project.id,
      root: project.approvedRoots[0],
      summary: "Restore one applied patch checkpoint inside the approved project root.",
    },
    status: "pending",
  };
}
