import type { Dispatch, SetStateAction } from "react";
import {
  previewExternalAgentContract,
  type ExternalAgentContractKind,
} from "../features/externalAgents/externalAgentClient";
import type { ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { notifyLocalAction } from "./ShellPreferenceController";

export interface ExternalAgentPreviewState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  setExternalAgentState: Dispatch<SetStateAction<ExternalAgentStateView>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function previewExternalAgentContractForRun(
  state: ExternalAgentPreviewState,
  kind: ExternalAgentContractKind,
) {
  if (!state.activeThread || !state.activeRun) {
    state.setThreadState(state.activeThread ? "ready" : "empty");
    notifyLocalAction("Create a thread and AgentRun before previewing an external-agent contract", "warning");
    return;
  }
  try {
    const contract = await previewExternalAgentContract({
      kind,
      permissionMode: "read_only",
      runId: state.activeRun.id,
      task: contractTask(state.activeThread),
      workingDirectory: state.activeProject.path,
    });
    state.setExternalAgentState((current) => ({
      ...current,
      contracts: [contract, ...current.contracts.filter((item) => item.id !== contract.id)],
    }));
    notifyLocalAction(`External-agent contract ready: ${contract.adapterId}`, "success");
  } catch (error) {
    notifyLocalAction(contractPreviewError(error), "warning");
  }
}

function contractTask(thread: TaskThread) {
  const userMessage = [...thread.messages].reverse().find((message) => message.role === "user");
  return userMessage?.body.trim() || thread.goal;
}

function contractPreviewError(error: unknown) {
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  return "Desktop bridge is required to preview external-agent contracts.";
}
