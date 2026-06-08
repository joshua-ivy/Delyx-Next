import { invoke } from "@tauri-apps/api/core";
import type { ReleaseSmokeStatus, ReleaseStateView } from "./releaseTypes";

export interface ReleaseSmokeCaptureRequest {
  status: Exclude<ReleaseSmokeStatus, "not_loaded">;
  installerPath: string;
  command: string;
  capturedAt: string;
  detail: string;
}

export interface SupportBundleFileExportRequest {
  runId: string;
  approvalId: string;
  outputPath: string;
  approvedRoots: string[];
  exportedAt: string;
  createdAtMs: number;
}

export async function captureReleaseSmokeOverBridge(
  request: ReleaseSmokeCaptureRequest,
): Promise<ReleaseStateView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<ReleaseStateView>("release_smoke_capture", { request });
  } catch {
    return undefined;
  }
}

export async function exportSupportBundleFileOverBridge(
  request: SupportBundleFileExportRequest,
): Promise<ReleaseStateView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<ReleaseStateView>("release_support_bundle_file_export", { request });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
