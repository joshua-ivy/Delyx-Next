//! Tauri bridge for native projects: snapshot, list, save, and remove.
//!
//! Projects are durable trust state, so every command persists through
//! `project_persistence` to SQLite. The bridge holds only the database path.

use crate::project::{
    ApprovalPolicyRecord, FileScopeRecord, MemoryScopeRecord, ModelPermissionsRecord,
    ProjectRecord, ProjectTrustLevel, ToolPermissionsRecord,
};
use crate::project_persistence::{
    delete_project_from_path, ensure_project_to_path, list_projects_from_path,
    load_project_from_path, save_project_to_path,
};
use serde::Deserialize;
use std::path::PathBuf;

pub struct ProjectBridgeState {
    database_path: PathBuf,
}

impl ProjectBridgeState {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

/// Save request from the UI. `id` is optional — a fresh project derives a stable
/// id from its root path. Timestamps are owned by the database, not the client.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSaveRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub root_path: String,
    #[serde(default = "default_trust")]
    pub trust_level: ProjectTrustLevel,
    #[serde(default)]
    pub allowed_file_scopes: Option<Vec<FileScopeRecord>>,
    #[serde(default)]
    pub approval_policy: Option<ApprovalPolicyRecord>,
    #[serde(default)]
    pub model_permissions: Option<ModelPermissionsRecord>,
    #[serde(default)]
    pub tool_permissions: Option<ToolPermissionsRecord>,
    #[serde(default)]
    pub memory_scope: Option<MemoryScopeRecord>,
}

fn default_trust() -> ProjectTrustLevel {
    ProjectTrustLevel::Local
}

impl ProjectSaveRequest {
    /// Fold the request onto a defaulted record so optional fields keep their
    /// safe defaults and a missing id resolves from the root path.
    pub fn into_record(self) -> ProjectRecord {
        let mut record = ProjectRecord::new(&self.name, &self.root_path);
        if let Some(id) = self.id.filter(|value| !value.trim().is_empty()) {
            record.id = id;
        }
        record.trust_level = self.trust_level;
        if let Some(scopes) = self.allowed_file_scopes {
            if !scopes.is_empty() {
                record.allowed_file_scopes = scopes;
            }
        }
        if let Some(policy) = self.approval_policy {
            record.approval_policy = policy;
        }
        if let Some(models) = self.model_permissions {
            record.model_permissions = models;
        }
        if let Some(tools) = self.tool_permissions {
            record.tool_permissions = tools;
        }
        if let Some(memory) = self.memory_scope {
            record.memory_scope = memory;
        }
        record
    }
}

#[tauri::command]
pub fn project_save(
    state: tauri::State<ProjectBridgeState>,
    request: ProjectSaveRequest,
) -> Result<ProjectRecord, String> {
    save_project_to_path(&state.database_path, &request.into_record())
}

/// Load-or-create the native project for a workspace root. Used when opening a
/// project so the UI always has trust/scope state to show without clobbering an
/// existing record.
#[tauri::command]
pub fn project_ensure(
    state: tauri::State<ProjectBridgeState>,
    name: String,
    root_path: String,
) -> Result<ProjectRecord, String> {
    ensure_project_to_path(&state.database_path, &name, &root_path)
}

#[tauri::command]
pub fn project_snapshot(
    state: tauri::State<ProjectBridgeState>,
    id: String,
) -> Result<Option<ProjectRecord>, String> {
    load_project_from_path(&state.database_path, &id)
}

#[tauri::command]
pub fn project_list(state: tauri::State<ProjectBridgeState>) -> Result<Vec<ProjectRecord>, String> {
    list_projects_from_path(&state.database_path)
}

#[tauri::command]
pub fn project_remove(state: tauri::State<ProjectBridgeState>, id: String) -> Result<(), String> {
    delete_project_from_path(&state.database_path, &id)
}
