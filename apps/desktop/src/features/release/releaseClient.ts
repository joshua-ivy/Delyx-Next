import { invoke } from "@tauri-apps/api/core";
import type { ReleaseSmokeStatus, ReleaseStateView } from "./releaseTypes";

export interface ReleaseSmokeCaptureRequest {
  status: Exclude<ReleaseSmokeStatus, "not_loaded">;
  installerPath: string;
  command: string;
  capturedAt: string;
  detail: string;
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

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
