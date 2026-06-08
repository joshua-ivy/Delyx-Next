import { invoke } from "@tauri-apps/api/core";
import { currentWorkspaceProject } from "../features/workspace/workspaceData";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

const defaultFileLimit = 250;

export async function loadWorkspaceProject(path?: string): Promise<WorkspaceProject> {
  if (!path) {
    const recent = await loadRecentWorkspaceProject();
    if (recent) {
      return recent;
    }
  }
  const requestedPath = path ?? currentWorkspaceProject.path;
  try {
    return await invoke<WorkspaceProject>("workspace_snapshot", { fileLimit: defaultFileLimit, path: requestedPath });
  } catch (error) {
    if (samePath(requestedPath, currentWorkspaceProject.path)) {
      return currentWorkspaceProject;
    }
    throw error;
  }
}

async function loadRecentWorkspaceProject(): Promise<WorkspaceProject | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    const project = await invoke<WorkspaceProject | null>("workspace_recent_project");
    return project ?? undefined;
  } catch {
    return undefined;
  }
}

function samePath(left: string, right: string) {
  return normalizePath(left) === normalizePath(right);
}

function normalizePath(path: string) {
  return path.replaceAll("\\", "/").replace(/\/+$/, "").toLowerCase();
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
