import { invoke } from "@tauri-apps/api/core";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

/**
 * Codebase awareness for the local model: a compact project context block —
 * identity, a capped repo map, and the project's rules files — prepended to the
 * system prompt so the model stops guessing at a codebase it has never seen.
 * Real file contents come through the existing scope-enforced
 * `workspace_read_files` bridge; everything is capped to keep token cost flat.
 */

const MAX_MAP_FILES = 80;
const MAX_RULES_FILES = 2;
const MAX_RULES_BYTES_PER_FILE = 6_000;

interface WorkspaceFileReadView {
  path: string;
  contents: string;
  truncated: boolean;
}

export interface RulesFileContent {
  path: string;
  contents: string;
  truncated: boolean;
}

/** Pure formatter, unit-testable without the bridge. */
export function buildProjectContextBlock(
  project: WorkspaceProject,
  rules: RulesFileContent[],
): string {
  const parts: string[] = [];
  const branch = project.git.isRepo ? ` (git branch: ${project.git.branch})` : "";
  parts.push(`Project: ${project.name} at ${project.path}${branch}.`);
  if (project.indexedFiles.length > 0) {
    const files = project.indexedFiles.slice(0, MAX_MAP_FILES);
    const more = project.indexedFiles.length - files.length;
    parts.push(
      `Repository files${more > 0 ? ` (first ${files.length} of ${project.indexedFiles.length})` : ""}:\n${files.join("\n")}`,
    );
  }
  for (const rule of rules) {
    const note = rule.truncated ? " (truncated)" : "";
    parts.push(`Project rules from ${rule.path}${note}:\n${rule.contents.trim()}`);
  }
  return parts.join("\n\n");
}

const contextCache = new Map<string, string>();

/**
 * Build (and cache per project) the context block. Returns an empty string in
 * the web preview or when reads fail — the chat still works, just blind.
 */
export async function projectContextBlock(project: WorkspaceProject): Promise<string> {
  const cacheKey = `${project.id}:${project.rulesFiles.length}:${project.indexedFiles.length}`;
  const cached = contextCache.get(cacheKey);
  if (cached !== undefined) {
    return cached;
  }
  const rules = await readRulesFiles(project);
  const block = buildProjectContextBlock(project, rules);
  contextCache.set(cacheKey, block);
  return block;
}

async function readRulesFiles(project: WorkspaceProject): Promise<RulesFileContent[]> {
  const paths = project.rulesFiles.slice(0, MAX_RULES_FILES).map((rule) => rule.path);
  if (paths.length === 0 || !hasTauriRuntime()) {
    return [];
  }
  try {
    const reads = await invoke<WorkspaceFileReadView[]>("workspace_read_files", {
      request: { projectPath: project.path, paths, maxBytesPerFile: MAX_RULES_BYTES_PER_FILE },
    });
    return reads
      .filter((read) => read.contents.trim().length > 0)
      .map((read) => ({ path: read.path, contents: read.contents, truncated: read.truncated }));
  } catch {
    return [];
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
