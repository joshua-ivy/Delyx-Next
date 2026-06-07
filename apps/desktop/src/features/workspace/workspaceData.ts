import type { WorkspaceProject } from "./workspaceTypes";

export const currentWorkspaceProject: WorkspaceProject = {
  id: "c-users-geaux-downloads-delyx-next",
  name: "delyx-next",
  path: "C:/Users/geaux/Downloads/Delyx Next",
  approvedRoots: ["C:/Users/geaux/Downloads/Delyx Next"],
  approvalPolicy: "Approval required for file writes, terminal commands, memory saves, connector writes, and external agents.",
  git: {
    isRepo: true,
    branch: "main",
    uncommittedChanges: null,
  },
  isolation: {
    detail: "Checkpoint or worktree appears after an approved build action.",
    label: "No active isolation",
    mode: "none",
  },
  lastOpenedLabel: "Current local session",
  pinned: false,
  rulesFiles: [
    {
      path: "AGENTS.md",
      kind: "AGENTS.md",
    },
  ],
  indexedFiles: [
    "AGENTS.md",
    "DELYX_NEXT_UI_FIRST_CODEX_BUILD_PLAN.md",
    "README.md",
    "Cargo.toml",
    "package.json",
    "apps/desktop/package.json",
    "apps/desktop/src/app/AppShell.tsx",
    "apps/desktop/src/app/cockpitMarkup.ts",
    "apps/desktop/src/design-system/tokens.css",
    "apps/desktop/src/styles/cockpit.css",
    "apps/desktop/src-tauri/src/workspace.rs",
    "apps/desktop/src-tauri/src/workspace_tests.rs",
    "docs/PRODUCT_DIRECTION.md",
    "docs/UI_ARCHITECTURE.md",
  ],
};
