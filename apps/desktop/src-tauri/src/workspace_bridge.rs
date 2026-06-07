use crate::workspace::{Project, RulesFileKind, WorkspaceError, WorkspaceManager};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct WorkspaceBridgeState {
    database_path: PathBuf,
}

impl WorkspaceBridgeState {
    pub fn persistent(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceProjectView {
    pub id: String,
    pub name: String,
    pub path: String,
    pub approved_roots: Vec<String>,
    pub approval_policy: String,
    pub git: WorkspaceGitView,
    pub isolation: WorkspaceIsolationView,
    pub last_opened_label: String,
    pub pinned: bool,
    pub rules_files: Vec<WorkspaceRulesFileView>,
    pub indexed_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceGitView {
    pub is_repo: bool,
    pub branch: String,
    pub uncommitted_changes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkspaceIsolationView {
    pub detail: String,
    pub label: String,
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkspaceRulesFileView {
    pub path: String,
    pub kind: String,
}

#[tauri::command]
pub fn workspace_snapshot(
    state: tauri::State<WorkspaceBridgeState>,
    path: String,
    file_limit: usize,
) -> Result<WorkspaceProjectView, String> {
    let snapshot = workspace_snapshot_from_path(Path::new(&path), file_limit).map_err(|error| format!("{error:?}"))?;
    crate::workspace_persistence::save_recent_project(&state.database_path, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn workspace_recent_project(
    state: tauri::State<WorkspaceBridgeState>,
) -> Result<Option<WorkspaceProjectView>, String> {
    crate::workspace_persistence::load_recent_project(&state.database_path)
}

pub fn workspace_snapshot_from_path(path: &Path, file_limit: usize) -> Result<WorkspaceProjectView, WorkspaceError> {
    let mut manager = WorkspaceManager::new();
    let project = manager.add_project(path)?;
    let indexed_files = manager
        .index_files(&project.id, file_limit)
        .map(|entries| entries.into_iter().map(|entry| entry.relative_path).collect())?;
    Ok(project_view(project, indexed_files))
}

fn project_view(project: Project, indexed_files: Vec<String>) -> WorkspaceProjectView {
    WorkspaceProjectView {
        approved_roots: project.approved_roots.iter().map(|root| display_path(root)).collect(),
        approval_policy: "Approval required for file writes, terminal commands, memory saves, connector writes, and external agents.".to_string(),
        git: WorkspaceGitView {
            branch: project.git.branch.unwrap_or_else(|| "detached or unknown".to_string()),
            is_repo: project.git.is_repo,
            uncommitted_changes: project.git.uncommitted_changes,
        },
        id: project.id,
        indexed_files,
        isolation: WorkspaceIsolationView {
            detail: "Checkpoint or worktree appears after an approved build action.".to_string(),
            label: "No active isolation".to_string(),
            mode: "none".to_string(),
        },
        last_opened_label: "Current local session".to_string(),
        name: project.name,
        path: display_path(&project.path),
        pinned: false,
        rules_files: project.rules_files.iter().map(rules_file_view).collect(),
    }
}

fn rules_file_view(file: &crate::workspace::ProjectRulesFile) -> WorkspaceRulesFileView {
    WorkspaceRulesFileView {
        kind: rules_kind(file.kind.clone()).to_string(),
        path: display_path(&file.path),
    }
}

fn rules_kind(kind: RulesFileKind) -> &'static str {
    match kind {
        RulesFileKind::Agents => "AGENTS.md",
        RulesFileKind::Claude => "CLAUDE.md",
        RulesFileKind::Delyx => "DELYX.md",
        RulesFileKind::DelyxRule => ".delyx/rules",
    }
}

fn display_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    path.strip_prefix("//?/").unwrap_or(&path).to_string()
}
