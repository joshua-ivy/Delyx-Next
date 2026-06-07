import { invoke } from "@tauri-apps/api/core";
import type { PatchProposalView } from "../patches/patchTypes";
import type { TestArtifactView } from "../tests/testTypes";
import type { ReviewReportView } from "./reviewTypes";

export interface ReviewCreateRequestView {
  runId: string;
  patches: PatchProposalView[];
  tests: TestArtifactView[];
}

export async function createReviewOverBridge(
  request: ReviewCreateRequestView,
): Promise<ReviewReportView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<ReviewReportView>("review_create", { request });
  } catch {
    return undefined;
  }
}

export async function loadReviewSnapshot(runId: string): Promise<ReviewReportView[] | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<ReviewReportView[]>("review_snapshot", { runId });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
