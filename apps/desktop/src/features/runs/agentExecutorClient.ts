import { invoke } from "@tauri-apps/api/core";
import type { PatchProposalRequestView } from "../patches/patchClient";

export interface AgentPatchProposalExecuteRequest extends PatchProposalRequestView {
  createdAtMs: number;
}

export interface AgentExecutionBridgeView {
  status: "completed" | "failed" | "waiting_for_approval";
  runId: string;
  patchId?: string;
  message: string;
}

export async function executePatchProposalNodeOverBridge(
  request: AgentPatchProposalExecuteRequest,
): Promise<AgentExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentExecutionBridgeView>("agent_execute_patch_proposal", { request });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
