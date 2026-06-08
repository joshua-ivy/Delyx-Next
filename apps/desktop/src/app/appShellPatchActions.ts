import type { Dispatch, SetStateAction } from "react";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { loadPatchSnapshot } from "../features/patches/patchClient";
import type { PatchProposalView } from "../features/patches/patchTypes";
import { executePatchApplyNodeOverBridge, executePatchRestoreNodeOverBridge } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadThreadRunSnapshot, updateThreadStatusOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { upsertActionProposal } from "./appShellApprovalActions";
import { recordApprovalProposalForRun } from "./appShellRunActions";
import { updateThreadAndRunStatus } from "./cockpitStateTransitions";
import { activePatchApplyApproval, createPatchApplyApprovalProposal } from "./patchApplyApproval";
import { activePatchRestoreApproval, createPatchRestoreApprovalProposal } from "./patchRestoreApproval";
import { notifyLocalAction } from "./ShellPreferenceController";

export interface PatchApplyState {
  actionProposals: ActionProposalView[];
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  patch: PatchProposalView | undefined;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setPatches: Dispatch<SetStateAction<PatchProposalView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function applyApprovedPatchForActiveRun(state: PatchApplyState) {
  if (!state.patch || !state.activeThread) {
    notifyLocalAction("Select a patch before changing files", "warning");
    return;
  }
  if (state.patch.status === "applied") {
    await restoreAppliedPatchForActiveRun(state);
    return;
  }
  if (state.patch.status !== "proposed") {
    notifyLocalAction("Only proposed or applied patches can be changed", "warning");
    return;
  }
  if (state.activeProject.approvedRoots.length === 0) {
    notifyLocalAction("Patch apply requires an approved workspace root", "warning");
    return;
  }
  const approval = activePatchApplyApproval(state.actionProposals, state.patch);
  if (!approval || approval.status !== "approved") {
    await queuePatchApplyApproval(state, approval);
    return;
  }

  const result = await executePatchApplyNodeOverBridge({
    approvalId: approval.id,
    approvedRoots: state.activeProject.approvedRoots,
    createdAtMs: Date.now(),
    proposalId: state.patch.id,
  });
  if (!result) {
    notifyLocalAction("Desktop bridge is required to apply patches", "warning");
    return;
  }

  if (result.status === "completed") {
    await updateThreadStatusOverBridge(state.activeThread.id, "testing", new Date().toISOString());
  }
  await reloadPatchState(state);
  state.setThreadState("ready");
  notifyLocalAction(result.message, result.status === "completed" ? "success" : "warning");
}

async function restoreAppliedPatchForActiveRun(state: PatchApplyState) {
  if (!state.patch || !state.activeThread) {
    return;
  }
  if (state.activeProject.approvedRoots.length === 0) {
    notifyLocalAction("Patch restore requires an approved workspace root", "warning");
    return;
  }
  const approval = activePatchRestoreApproval(state.actionProposals, state.patch);
  if (!approval || approval.status !== "approved") {
    await queuePatchRestoreApproval(state, approval);
    return;
  }

  const result = await executePatchRestoreNodeOverBridge({
    approvalId: approval.id,
    approvedRoots: state.activeProject.approvedRoots,
    createdAtMs: Date.now(),
    proposalId: state.patch.id,
  });
  if (!result) {
    notifyLocalAction("Desktop bridge is required to restore patches", "warning");
    return;
  }
  if (result.status === "completed") {
    await updateThreadStatusOverBridge(state.activeThread.id, "reviewing", new Date().toISOString());
  }
  await reloadPatchState(state);
  state.setThreadState("ready");
  notifyLocalAction(result.message, result.status === "completed" ? "success" : "warning");
}

async function queuePatchApplyApproval(
  state: PatchApplyState,
  existing: ActionProposalView | undefined,
) {
  if (!state.patch || !state.activeThread) {
    return;
  }
  if (existing?.status === "pending") {
    notifyLocalAction("Approve the patch apply request, then apply again", "warning");
    return;
  }
  if (existing && existing.status !== "expired") {
    notifyLocalAction(`Patch apply approval is ${existing.status}`, "warning");
    return;
  }
  const fallback = createPatchApplyApprovalProposal(state.patch, state.activeRun, state.activeProject);
  const proposal = existing?.status === "expired" ? { ...fallback, id: `${fallback.id}-${Date.now()}` } : fallback;
  const recorded = await proposeApprovalOverBridge(proposal) ?? proposal;
  const now = new Date().toISOString();
  state.setActionProposals((current) => upsertActionProposal(current, recorded));
  state.setAgentRuns((current) => recordApprovalProposalForRun(current, state.activeThread!, recorded, now));
  updateThreadAndRunStatus(state, state.activeThread, "waiting_for_approval");
  notifyLocalAction("Approve the patch apply request before writing files", "warning");
}

async function queuePatchRestoreApproval(
  state: PatchApplyState,
  existing: ActionProposalView | undefined,
) {
  if (!state.patch || !state.activeThread) {
    return;
  }
  if (existing?.status === "pending") {
    notifyLocalAction("Approve the patch restore request, then restore again", "warning");
    return;
  }
  if (existing && existing.status !== "expired") {
    notifyLocalAction(`Patch restore approval is ${existing.status}`, "warning");
    return;
  }
  const fallback = createPatchRestoreApprovalProposal(state.patch, state.activeRun, state.activeProject);
  const proposal = existing?.status === "expired" ? { ...fallback, id: `${fallback.id}-${Date.now()}` } : fallback;
  const recorded = await proposeApprovalOverBridge(proposal) ?? proposal;
  const now = new Date().toISOString();
  state.setActionProposals((current) => upsertActionProposal(current, recorded));
  state.setAgentRuns((current) => recordApprovalProposalForRun(current, state.activeThread!, recorded, now));
  updateThreadAndRunStatus(state, state.activeThread, "waiting_for_approval");
  notifyLocalAction("Approve the patch restore request before writing files", "warning");
}

async function reloadPatchState(state: PatchApplyState) {
  const [patches, snapshot] = await Promise.all([
    loadPatchSnapshot(state.patch?.runId ?? ""),
    loadThreadRunSnapshot(state.activeProject.id),
  ]);
  if (patches) {
    state.setPatches(patches);
  }
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
}
