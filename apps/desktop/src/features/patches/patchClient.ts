import { invoke } from "@tauri-apps/api/core";
import type { PatchProposalView } from "./patchTypes";

export interface PatchProposalRequestView {
  clientId: string;
  runId: string;
  approvalId: string;
  approvedRoots: string[];
  files: Array<{
    path: string;
    after: string;
  }>;
}

export async function proposePatchOverBridge(
  request: PatchProposalRequestView,
): Promise<PatchProposalView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<PatchProposalView>("patch_propose", { request });
  } catch {
    return undefined;
  }
}

export async function loadPatchSnapshot(runId: string): Promise<PatchProposalView[] | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<PatchProposalView[]>("patch_snapshot", { runId });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
