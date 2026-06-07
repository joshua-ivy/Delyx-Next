export type WorkspaceUiState = "ready" | "loading" | "empty" | "error" | "denied";

export interface WorkspaceGitState {
  isRepo: boolean;
  branch: string;
  uncommittedChanges: number;
}

export interface WorkspaceRulesFile {
  path: string;
  kind: "AGENTS.md" | "DELYX.md" | "CLAUDE.md" | ".delyx/rules";
}

export interface WorkspaceProject {
  id: string;
  name: string;
  path: string;
  approvedRoots: string[];
  git: WorkspaceGitState;
  rulesFiles: WorkspaceRulesFile[];
  indexedFiles: string[];
}
