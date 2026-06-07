import { invoke } from "@tauri-apps/api/core";
import type { TestArtifactView } from "./testTypes";

export interface TestRunRequestView {
  runId: string;
  approvalId: string;
  program: string;
  args: string[];
  workingDirectory: string;
  approvedRoots: string[];
  timeoutMs: number;
  startedAt: string;
  completedAt?: string;
  createdAtMs: number;
}

export async function runApprovedTestOverBridge(
  request: TestRunRequestView,
): Promise<TestArtifactView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<TestArtifactView>("test_run_approved", { request });
  } catch {
    return undefined;
  }
}

export async function loadTestSnapshot(runId: string): Promise<TestArtifactView[] | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<TestArtifactView[]>("test_snapshot", { runId });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
