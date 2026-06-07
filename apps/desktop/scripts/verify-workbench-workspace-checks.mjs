export const workspaceChecks = [
  ["src-tauri/src/workspace_bridge.rs", "workspace_snapshot"],
  ["src-tauri/src/workspace_bridge.rs", "WorkspaceProjectView"],
  ["src-tauri/src/workspace_bridge.rs", "index_files"],
  ["src-tauri/src/workspace_bridge_tests.rs", "workspace_snapshot_exposes_real_rules_and_indexed_files"],
  ["src-tauri/src/main.rs", "workspace_bridge::workspace_snapshot"],
  ["src/app/workspaceBridge.ts", "@tauri-apps/api/core"],
  ["src/app/workspaceBridge.ts", "workspace_snapshot"],
  ["src/app/AppShell.tsx", "loadWorkspaceProject"],
  ["src/features/workspace/workspaceData.ts", "branch: \"branch not loaded\""],
  ["src/features/workspace/workspaceData.ts", "indexedFiles: []"],
  ["src/features/workspace/WorkspaceOverlay.tsx", "No indexed files are loaded for this query"],
];
