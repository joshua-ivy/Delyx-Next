import { invoke } from "@tauri-apps/api/core";
import type { PlanView } from "./planTypes";

export async function savePlanOverBridge(
  projectId: string,
  plan: PlanView,
): Promise<PlanView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<PlanView>("plan_save", { request: { plan, projectId } });
  } catch {
    return undefined;
  }
}

export async function loadPlanSnapshot(projectId: string): Promise<PlanView[] | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<PlanView[]>("plan_snapshot", { projectId });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
