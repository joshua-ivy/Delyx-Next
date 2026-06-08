import { invoke } from "@tauri-apps/api/core";
import type { AgentTestExecutionBridgeView } from "./agentExecutorClient";

export interface AgentTestStepRequest {
  nowMs: number;
  runId: string;
  startedAt: string;
  timeoutMs?: number;
  updatedAt: string;
}

export async function runTestSchedulerStepOverBridge(
  request: AgentTestStepRequest,
): Promise<AgentTestExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentTestExecutionBridgeView>("agent_run_test_step", { request });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
