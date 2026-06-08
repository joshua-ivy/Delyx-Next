import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ReviewFindingView, ReviewReportView } from "../features/review/reviewTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export interface RepairPatchDraftApprovalState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread?: TaskThread | undefined;
}

export function createRepairPatchDraftApprovalProposal(
  state: RepairPatchDraftApprovalState,
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

function repairApprovalIdFor(runId: string, reportId: string, findingId: string) {
  return `approval-${runId}-${reportId}-${findingId}-repair-draft`;
}

function repairNodeIdFor(runId: string, reportId: string, findingId: string) {
  return `${runId}-repair-${reportId}-${findingId}`;
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
