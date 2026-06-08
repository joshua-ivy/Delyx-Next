import { invoke } from "@tauri-apps/api/core";

export interface AgentDriveRequestView {
  finalSummary?: string;
  nowMs: number;
  runId: string;
  timeoutMs?: number;
  updatedAt: string;
}

export interface AgentDriveOutcomeView {
  runId: string;
  steps: AgentDriveStepView[];
  stoppedBecause: AgentDriveStopView;
}

export interface AgentDriveStepView {
  decision: string;
  status: string;
  message: string;
}

export interface AgentDriveStopView {
  kind: string;
  message: string;
  approvalIds: string[];
  proposalId?: string;
  reviewReportId?: string;
  findingId?: string;
  status?: string;
}

export async function driveRunOverBridge(
  request: AgentDriveRequestView,
): Promise<AgentDriveOutcomeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentDriveOutcomeView>("agent_drive_run", { request });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
