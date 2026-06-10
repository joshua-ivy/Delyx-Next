import { invoke } from "@tauri-apps/api/core";
import type { ProjectSaveRequest, ProjectView } from "./projectTypes";

/**
 * Native project bridge. Projects are durable Delyx trust state persisted in
 * SQLite — these wrappers call the Rust `project_*` commands. They require the
 * desktop runtime; callers should handle the rejection in the web preview.
 */
export async function saveProject(request: ProjectSaveRequest): Promise<ProjectView> {
  return invoke<ProjectView>("project_save", { request });
}

export async function loadProject(id: string): Promise<ProjectView | null> {
  return invoke<ProjectView | null>("project_snapshot", { id });
}

/** Load the native project for a workspace root, creating defaults if missing. */
export async function ensureProject(name: string, rootPath: string): Promise<ProjectView> {
  return invoke<ProjectView>("project_ensure", { name, rootPath });
}

export async function listProjects(): Promise<ProjectView[]> {
  return invoke<ProjectView[]>("project_list");
}

export async function removeProject(id: string): Promise<void> {
  await invoke("project_remove", { id });
}
