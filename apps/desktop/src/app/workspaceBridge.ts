import { invoke } from "@tauri-apps/api/core";
import { currentWorkspaceProject } from "../features/workspace/workspaceData";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

const defaultFileLimit = 250;

export async function loadWorkspaceProject(path = currentWorkspaceProject.path): Promise<WorkspaceProject> {
  try {
    return await invoke<WorkspaceProject>("workspace_snapshot", { fileLimit: defaultFileLimit, path });
  } catch (error) {
    if (samePath(path, currentWorkspaceProject.path)) {
      return currentWorkspaceProject;
    }
    throw error;
  }
}

function samePath(left: string, right: string) {
  return normalizePath(left) === normalizePath(right);
}

function normalizePath(path: string) {
  return path.replaceAll("\\", "/").replace(/\/+$/, "").toLowerCase();
}
