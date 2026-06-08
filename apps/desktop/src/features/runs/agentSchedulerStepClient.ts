import { invoke } from "@tauri-apps/api/core";
import type {
  AgentReviewExecutionBridgeView,
  AgentTestExecutionBridgeView,
} from "./agentExecutorClient";

export interface AgentTestStepRequest {
  nowMs: number;
  runId: string;
  startedAt: string;
  timeoutMs?: number;
  updatedAt: string;
}

export interface AgentReviewStepRequest {
  nowMs: number;
  runId: string;
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

export async function runReviewSchedulerStepOverBridge(
  request: AgentReviewStepRequest,
): Promise<AgentReviewExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentReviewExecutionBridgeView>("agent_run_review_step", { request });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
