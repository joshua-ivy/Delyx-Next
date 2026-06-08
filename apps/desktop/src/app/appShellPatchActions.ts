import type { Dispatch, SetStateAction } from "react";

import { loadPatchSnapshot } from "../features/patches/patchClient";
import type { PatchProposalView } from "../features/patches/patchTypes";
import { executePatchApplyNodeOverBridge } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadThreadRunSnapshot, updateThreadStatusOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { notifyLocalAction } from "./ShellPreferenceController";

export interface PatchApplyState {
  activeProject: WorkspaceProject;
  activeThread: TaskThread | undefined;
  patch: PatchProposalView | undefined;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setPatches: Dispatch<SetStateAction<PatchProposalView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function applyApprovedPatchForActiveRun(state: PatchApplyState) {
  if (!state.patch || !state.activeThread) {
    notifyLocalAction("Select a proposed patch before applying", "warning");
    return;
  }
  if (state.patch.status !== "proposed") {
    notifyLocalAction("Only proposed patches can be applied", "warning");
    return;
  }
  if (state.activeProject.approvedRoots.length === 0) {
    notifyLocalAction("Patch apply requires an approved workspace root", "warning");
    return;
  }

  const result = await executePatchApplyNodeOverBridge({
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
