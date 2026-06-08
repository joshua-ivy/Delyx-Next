use crate::workspace::{Project, RulesFileKind, WorkspaceError, WorkspaceManager};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

const MAX_READ_FILE_COUNT: usize = 4;
const DEFAULT_MAX_READ_BYTES: usize = 20_000;

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceReadFilesRequest {
    pub project_path: String,
    pub paths: Vec<String>,
    pub max_bytes_per_file: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFileReadView {
    pub path: String,
    pub contents: String,
    pub truncated: bool,
}

#[tauri::command]
pub fn workspace_snapshot(
    state: tauri::State<WorkspaceBridgeState>,
    path: String,
    file_limit: usize,
) -> Result<WorkspaceProjectView, String> {
    let snapshot = workspace_snapshot_from_path(Path::new(&path), file_limit)
        .map_err(|error| format!("{error:?}"))?;
    crate::workspace_persistence::save_recent_project(&state.database_path, &snapshot)?;
    Ok(snapshot)
}

#[tauri::command]
pub fn workspace_recent_project(
    state: tauri::State<WorkspaceBridgeState>,
) -> Result<Option<WorkspaceProjectView>, String> {
    crate::workspace_persistence::load_recent_project(&state.database_path)
}

#[tauri::command]
pub fn workspace_read_files(
    request: WorkspaceReadFilesRequest,
) -> Result<Vec<WorkspaceFileReadView>, String> {
    workspace_read_files_from_path(
        Path::new(&request.project_path),
        &request.paths,
        request.max_bytes_per_file.unwrap_or(DEFAULT_MAX_READ_BYTES),
    )
}

pub fn workspace_snapshot_from_path(
    path: &Path,
    file_limit: usize,
) -> Result<WorkspaceProjectView, WorkspaceError> {
    let mut manager = WorkspaceManager::new();
    let project = manager.add_project(path)?;
    let indexed_files = manager
        .index_files(&project.id, file_limit)
        .map(|entries| {
            entries
                .into_iter()
                .map(|entry| entry.relative_path)
                .collect()
        })?;
    Ok(project_view(project, indexed_files))
}

pub fn workspace_read_files_from_path(
    project_path: &Path,
    paths: &[String],
    max_bytes_per_file: usize,
) -> Result<Vec<WorkspaceFileReadView>, String> {
    if paths.is_empty() || paths.len() > MAX_READ_FILE_COUNT || max_bytes_per_file == 0 {
        return Err("Workspace file read requires 1-4 paths and a byte limit.".to_string());
    }
    let mut manager = WorkspaceManager::new();
    let project = manager
        .add_project(project_path)
        .map_err(|error| format!("{error:?}"))?;
    paths
        .iter()
        .map(|path| read_project_file(&manager, &project, path, max_bytes_per_file))
        .collect()
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

fn read_project_file(
    manager: &WorkspaceManager,
    project: &Project,
    path: &str,
    max_bytes_per_file: usize,
) -> Result<WorkspaceFileReadView, String> {
    let relative = safe_relative_path(path)?;
    let relative_label = display_path(&relative);
    let contents = manager
        .read_file(&project.id, project.path.join(&relative))
        .map_err(|error| format!("{error:?}"))?;
    let (contents, truncated) = truncate_text(contents, max_bytes_per_file);
    Ok(WorkspaceFileReadView {
        contents,
        path: relative_label,
        truncated,
    })
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

fn safe_relative_path(path: &str) -> Result<PathBuf, String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("Workspace file path is empty.".to_string());
    }
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return Err("Workspace file reads require relative project paths.".to_string());
    }
    let mut normalized = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            _ => return Err("Workspace file path must stay inside the project.".to_string()),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("Workspace file path is empty.".to_string());
    }
    Ok(normalized)
}

fn truncate_text(contents: String, max_bytes: usize) -> (String, bool) {
    if contents.len() <= max_bytes {
        return (contents, false);
    }
    let mut end = max_bytes;
    while !contents.is_char_boundary(end) {
        end -= 1;
    }
    (contents[..end].to_string(), true)
}

fn display_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    path.strip_prefix("//?/").unwrap_or(&path).to_string()
}
