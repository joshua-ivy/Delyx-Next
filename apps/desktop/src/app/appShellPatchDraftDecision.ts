import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export interface PatchDraftDecisionState {
  actionProposals: ActionProposalView[];
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  patches: PatchProposalView[];
}

export function patchDraftApprovalId(state: PatchDraftDecisionState) {
  return patchDraftApprovalForApprovedPlan(state)?.id;
}

export function patchDraftApprovalForApprovedPlan(state: PatchDraftDecisionState) {
  return state.actionProposals.find((approval) => shouldDraftPatchAfterPlanApproval(state, approval));
}

export function approvedPatchDraftPlanFiles(
  state: PatchDraftDecisionState,
  approval: ActionProposalView,
) {
  const indexed = new Set(state.activeProject.indexedFiles.map(normalizePath));
  const scoped = new Set((approval.scope.paths ?? []).map(normalizePath));
  return (state.activePlan?.filesLikelyInvolved ?? [])
    .filter((path) => indexed.has(normalizePath(path)))
    .filter((path) => scoped.size === 0 || scoped.has(normalizePath(path)))
    .slice(0, 4);
}

function shouldDraftPatchAfterPlanApproval(
  state: PatchDraftDecisionState,
  approval: ActionProposalView,
) {
  return Boolean(
    approval.status === "approved"
      && (approval.actionType === "edit_file" || approval.actionType === "write_file")
      && state.activePlan?.decision === "approved"
      && state.activeRun
      && state.patches.every((patch) => patch.runId !== state.activeRun?.id)
      && approvedPatchDraftPlanFiles(state, approval).length > 0,
  );
}

function normalizePath(path: string) {
  return path.replace(/\\/g, "/").replace(/^\.\//, "").toLowerCase();
}
