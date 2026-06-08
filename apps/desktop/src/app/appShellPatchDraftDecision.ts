import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewFindingView, ReviewReportView } from "../features/review/reviewTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export interface PatchDraftDecisionState {
  actionProposals: ActionProposalView[];
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread?: TaskThread | undefined;
  patches: PatchProposalView[];
  reviews?: ReviewReportView[];
}

export function patchDraftApprovalId(state: PatchDraftDecisionState) {
  return patchDraftApprovalForApprovedPlan(state)?.id ?? patchDraftApprovalForRepair(state)?.id;
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

export function approvedPatchDraftFiles(
  state: PatchDraftDecisionState,
  approval: ActionProposalView,
) {
  if (shouldDraftPatchAfterPlanApproval(state, approval)) {
    return approvedPatchDraftPlanFiles(state, approval);
  }
  const repair = activeRepairFinding(state);
  if (repair && approvalMatchesRepair(state, approval, repair.report, repair.finding)) {
    return [repair.path];
  }
  return [];
}

export function patchDraftPrompt(
  state: PatchDraftDecisionState,
  approval: ActionProposalView,
  threadGoal: string,
) {
  const repair = activeRepairFinding(state);
  if (repair && approvalMatchesRepair(state, approval, repair.report, repair.finding)) {
    return {
      goal: `Repair review finding "${repair.finding.title}" for: ${threadGoal}`,
      planSteps: [
        `Fix ${repair.path}: ${repair.finding.detail}`,
        `Suggested fix: ${repair.finding.suggestedFix}`,
      ],
    };
  }
  return { goal: threadGoal, planSteps: state.activePlan?.steps ?? [] };
}

export function patchDraftScopePaths(state: PatchDraftDecisionState, approval: ActionProposalView) {
  return approval.scope.paths ?? approvedPatchDraftFiles(state, approval);
}

export function patchDraftApprovalForRepair(state: PatchDraftDecisionState) {
  const repair = activeRepairFinding(state);
  if (!repair) {
    return undefined;
  }
  return state.actionProposals.find((approval) => (
    approvalMatchesRepair(state, approval, repair.report, repair.finding)
    && approval.status === "approved"
    && (approval.actionType === "edit_file" || approval.actionType === "write_file")
    && !state.patches.some((patch) => patch.approvalId === approval.id)
  ));
}

export function createRepairPatchDraftApprovalProposal(
  state: Pick<PatchDraftDecisionState, "activeProject" | "activeRun" | "activeThread">,
  report: ReviewReportView,
  finding: ReviewFindingView,
): ActionProposalView | undefined {
  const runId = state.activeRun?.id ?? state.activeThread?.activeRunId;
  const path = repairFindingRelativePath(finding.filePath, state.activeProject.path);
  const root = state.activeProject.approvedRoots[0];
  if (!runId || !root || !path) {
    return undefined;
  }
  return {
    actionType: "edit_file",
    expectedResult: `Allow Delyx to draft a repair patch proposal for ${path} without applying it.`,
    expiresAt: new Date(Date.now() + 30 * 60 * 1000).toISOString(),
    id: repairApprovalIdFor(runId, report.id, finding.id),
    nodeId: repairNodeIdFor(runId, report.id, finding.id),
    rationale: `Repair review finding: ${finding.title}. ${finding.suggestedFix}`,
    requiredPermission: "edit_file",
    riskLabel: "high",
    rollbackPlan: "PatchDraft only proposes a diff; applying it still requires a separate checkpointed approval.",
    runId,
    scope: {
      kind: "file",
      paths: [path],
      projectId: state.activeProject.id,
      root,
      summary: `Repair ${finding.title}`,
    },
    status: "pending",
  };
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

function activeRepairFinding(state: PatchDraftDecisionState) {
  const report = [...(state.reviews ?? [])]
    .reverse()
    .find((item) => item.runId === state.activeRun?.id && item.decision === "revise_requested");
  const finding = report?.findings[0];
  const path = finding ? repairFindingRelativePath(finding.filePath, state.activeProject.path) : undefined;
  return report && finding && path ? { finding, path, report } : undefined;
}

function repairApprovalId(
  state: PatchDraftDecisionState,
  report: ReviewReportView,
  finding: ReviewFindingView,
) {
  const runId = state.activeRun?.id ?? state.activeThread?.activeRunId ?? "run";
  return repairApprovalIdFor(runId, report.id, finding.id);
}

function repairNodeId(
  state: PatchDraftDecisionState,
  report: ReviewReportView,
  finding: ReviewFindingView,
) {
  const runId = state.activeRun?.id ?? state.activeThread?.activeRunId ?? "run";
  return repairNodeIdFor(runId, report.id, finding.id);
}

function repairApprovalIdFor(runId: string, reportId: string, findingId: string) {
  return `approval-${runId}-${reportId}-${findingId}-repair-draft`;
}

function repairNodeIdFor(runId: string, reportId: string, findingId: string) {
  return `${runId}-repair-${reportId}-${findingId}`;
}

function approvalMatchesRepair(
  state: PatchDraftDecisionState,
  approval: ActionProposalView,
  report: ReviewReportView,
  finding: ReviewFindingView,
) {
  return approval.id === repairApprovalId(state, report, finding)
    || approval.nodeId === repairNodeId(state, report, finding);
}

function repairFindingRelativePath(filePath: string, projectPath: string) {
  const normalized = filePath.replace(/\\/g, "/");
  const root = projectPath.replace(/\\/g, "/").replace(/\/+$/, "");
  const absolute = /^[a-zA-Z]:\//.test(normalized) || normalized.startsWith("/");
  const relative = absolute && normalized.toLowerCase().startsWith(`${root.toLowerCase()}/`)
    ? normalized.slice(root.length + 1)
    : absolute ? "" : normalized.replace(/^\.\//, "");
  const parts = relative.split("/").filter(Boolean);
  if (parts.length === 0 || parts.some((part) => part === "." || part === "..")) {
    return undefined;
  }
  return parts.join("/");
}

function normalizePath(path: string) {
  return path.replace(/\\/g, "/").replace(/^\.\//, "").toLowerCase();
}
