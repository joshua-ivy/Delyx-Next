import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export function createPlanApprovalProposal(
  plan: PlanView,
  thread: TaskThread,
  run: AgentRunView | undefined,
  project: WorkspaceProject,
): ActionProposalView {
  const runId = run?.id ?? thread.activeRunId ?? thread.id;
  return {
    actionType: "edit_file",
    expectedResult: "Allow Delyx to propose a patch for the approved plan without applying it.",
    expiresAt: new Date(Date.now() + 30 * 60 * 1000).toISOString(),
    id: `approval-${plan.threadId}-build`,
    nodeId: `${runId}-plan-approval`,
    rationale: plan.goalUnderstanding,
    requiredPermission: plan.permissionsNeeded[0] ?? "edit_file",
    riskLabel: "high",
    rollbackPlan: plan.rollbackStrategy,
    runId,
    scope: {
      kind: "file",
      paths: plan.filesLikelyInvolved,
      projectId: project.id,
      root: project.approvedRoots[0],
      summary: "Files likely involved in the approved plan",
    },
    status: "pending",
  };
}

export function upsertActionProposal(proposals: ActionProposalView[], proposal: ActionProposalView) {
  return [proposal, ...proposals.filter((item) => item.id !== proposal.id)];
}
