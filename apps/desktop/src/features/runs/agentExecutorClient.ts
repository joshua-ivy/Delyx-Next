import { invoke } from "@tauri-apps/api/core";
import type {
  PatchApplyRequestView,
  PatchProposalRequestView,
  PatchRestoreRequestView,
} from "../patches/patchClient";
import type { TestRunRequestView } from "../tests/testClient";

export interface AgentPatchProposalExecuteRequest extends PatchProposalRequestView {
  createdAtMs: number;
}

export interface AgentExecutionBridgeView {
  status: "completed" | "failed" | "waiting_for_approval";
  runId: string;
  patchId?: string;
  message: string;
}

export interface AgentTestExecutionBridgeView {
  status: "completed" | "failed" | "waiting_for_approval";
  runId: string;
  testArtifactId?: string;
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

export async function executePatchApplyNodeOverBridge(
  request: PatchApplyRequestView,
): Promise<AgentExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentExecutionBridgeView>("agent_execute_patch_apply", { request });
  } catch {
    return undefined;
  }
}

export async function executePatchRestoreNodeOverBridge(
  request: PatchRestoreRequestView,
): Promise<AgentExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentExecutionBridgeView>("agent_execute_patch_restore", { request });
  } catch {
    return undefined;
  }
}

export async function executeTestRunNodeOverBridge(
  request: TestRunRequestView,
): Promise<AgentTestExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentTestExecutionBridgeView>("agent_execute_test_run", { request });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
