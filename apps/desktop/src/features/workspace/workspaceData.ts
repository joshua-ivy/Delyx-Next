import type { WorkspaceProject } from "./workspaceTypes";

export const currentWorkspaceProject: WorkspaceProject = {
  id: "c-users-geaux-downloads-delyx-next",
  name: "delyx-next",
  path: "C:/Users/geaux/Downloads/Delyx Next",
  approvedRoots: ["C:/Users/geaux/Downloads/Delyx Next"],
  approvalPolicy: "Approval required for file writes, terminal commands, memory saves, connector writes, and external agents.",
  git: {
    isRepo: true,
    branch: "branch not loaded",
    uncommittedChanges: null,
  },
  isolation: {
    detail: "Checkpoint or worktree appears after an approved build action.",
    label: "No active isolation",
    mode: "none",
  },
  lastOpenedLabel: "Current local session",
  pinned: false,
  rulesFiles: [],
  indexedFiles: [],
};
