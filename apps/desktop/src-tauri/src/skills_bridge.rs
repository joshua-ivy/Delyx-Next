use crate::skills::{SkillManifest, SkillPermissions, SkillRegistry, SkillStatus, SkillTrust};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Default)]
pub struct SkillBridgeState {
    registry: Mutex<SkillRegistry>,
    database_path: Option<PathBuf>,
}

impl SkillBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        let registry = crate::skills_persistence::load_from_path(&database_path)?;
        Ok(Self {
            registry: Mutex::new(registry),
            database_path: Some(database_path),
        })
    }

    fn save_if_persistent(&self, registry: &SkillRegistry) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::skills_persistence::save_to_path(registry, path),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillStateView {
    pub skills: Vec<SkillManifestView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillManifestView {
    pub id: String,
    pub name: String,
    pub source: String,
    pub source_hash: String,
    pub trust: String,
    pub status: String,
    pub permissions: SkillPermissionsView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPermissionsView {
    pub can_run_scripts: bool,
    pub can_edit_files: bool,
    pub can_use_network: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillImportRequest {
    pub source: String,
    pub contents: String,
    pub trust: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillActivateRequest {
    pub skill_id: String,
    pub permissions: SkillPermissionsRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPermissionsRequest {
    pub can_run_scripts: bool,
    pub can_edit_files: bool,
    pub can_use_network: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillActionRequest {
    pub skill_id: String,
}

#[tauri::command]
pub fn skill_snapshot(state: tauri::State<SkillBridgeState>) -> Result<SkillStateView, String> {
    let registry = state
        .registry
        .lock()
        .map_err(|_| "Skill bridge lock failed.".to_string())?;
    Ok(skill_snapshot_from_registry(&registry))
}

#[tauri::command]
pub fn skill_import(
    state: tauri::State<SkillBridgeState>,
    request: SkillImportRequest,
) -> Result<SkillStateView, String> {
    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "Skill bridge lock failed.".to_string())?;
    let view = import_skill_record(&mut registry, request)?;
    state.save_if_persistent(&registry)?;
    Ok(view)
}

#[tauri::command]
pub fn skill_activate(
    state: tauri::State<SkillBridgeState>,
    request: SkillActivateRequest,
) -> Result<SkillStateView, String> {
    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "Skill bridge lock failed.".to_string())?;
    let view = activate_skill_record(&mut registry, request)?;
    state.save_if_persistent(&registry)?;
    Ok(view)
}

#[tauri::command]
pub fn skill_disable(
    state: tauri::State<SkillBridgeState>,
    request: SkillActionRequest,
) -> Result<SkillStateView, String> {
    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "Skill bridge lock failed.".to_string())?;
    let view = disable_skill_record(&mut registry, request)?;
    state.save_if_persistent(&registry)?;
    Ok(view)
}

#[tauri::command]
pub fn skill_suppress(
    state: tauri::State<SkillBridgeState>,
    request: SkillActionRequest,
) -> Result<SkillStateView, String> {
    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "Skill bridge lock failed.".to_string())?;
    let view = suppress_skill_record(&mut registry, request)?;
    state.save_if_persistent(&registry)?;
    Ok(view)
}

pub fn skill_snapshot_from_path(path: &Path) -> Result<SkillStateView, String> {
    let registry = crate::skills_persistence::load_from_path(path)?;
    Ok(skill_snapshot_from_registry(&registry))
}

pub fn skill_snapshot_from_registry(registry: &SkillRegistry) -> SkillStateView {
    SkillStateView {
        skills: registry.skills().iter().map(skill_view).collect(),
    }
}

pub fn import_skill_record(
    registry: &mut SkillRegistry,
    request: SkillImportRequest,
) -> Result<SkillStateView, String> {
    validate_import_request(&request)?;
    registry.import_skill_file(
        &request.source,
        &request.contents,
        parse_trust(&request.trust)?,
    );
    Ok(skill_snapshot_from_registry(registry))
}

pub fn activate_skill_record(
    registry: &mut SkillRegistry,
    request: SkillActivateRequest,
) -> Result<SkillStateView, String> {
    validate_skill_id(&request.skill_id)?;
    registry
        .activate(&request.skill_id, permissions_request(request.permissions))
        .map_err(|error| format!("{error:?}"))?;
    Ok(skill_snapshot_from_registry(registry))
}

pub fn disable_skill_record(
    registry: &mut SkillRegistry,
    request: SkillActionRequest,
) -> Result<SkillStateView, String> {
    validate_skill_id(&request.skill_id)?;
    registry
        .disable(&request.skill_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(skill_snapshot_from_registry(registry))
}

pub fn suppress_skill_record(
    registry: &mut SkillRegistry,
    request: SkillActionRequest,
) -> Result<SkillStateView, String> {
    validate_skill_id(&request.skill_id)?;
    registry
        .suppress(&request.skill_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(skill_snapshot_from_registry(registry))
}

fn skill_view(skill: &SkillManifest) -> SkillManifestView {
    SkillManifestView {
        id: skill.id.clone(),
        name: skill.name.clone(),
        source: skill.source.clone(),
        source_hash: skill.source_hash.clone(),
        trust: trust_key(skill.trust).to_string(),
        status: status_key(skill.status).to_string(),
        permissions: permissions_view(skill.permissions),
    }
}

fn permissions_view(permissions: SkillPermissions) -> SkillPermissionsView {
    SkillPermissionsView {
        can_run_scripts: permissions.can_run_scripts,
        can_edit_files: permissions.can_edit_files,
        can_use_network: permissions.can_use_network,
    }
}

fn permissions_request(request: SkillPermissionsRequest) -> SkillPermissions {
    SkillPermissions {
        can_run_scripts: request.can_run_scripts,
        can_edit_files: request.can_edit_files,
        can_use_network: request.can_use_network,
    }
}

fn trust_key(trust: SkillTrust) -> &'static str {
    match trust {
        SkillTrust::Local => "local",
        SkillTrust::ThirdParty => "third_party",
    }
}

fn parse_trust(value: &str) -> Result<SkillTrust, String> {
    match value {
        "local" => Ok(SkillTrust::Local),
        "third_party" => Ok(SkillTrust::ThirdParty),
        _ => Err("Unsupported skill trust.".to_string()),
    }
}

fn status_key(status: SkillStatus) -> &'static str {
    match status {
        SkillStatus::Active => "active",
        SkillStatus::Disabled => "disabled",
        SkillStatus::Inactive => "inactive",
        SkillStatus::Suppressed => "suppressed",
    }
}

fn validate_import_request(request: &SkillImportRequest) -> Result<(), String> {
    if request.source.trim().is_empty() || request.contents.trim().is_empty() {
        return Err("Skill import requires source and contents.".to_string());
    }
    Ok(())
}

fn validate_skill_id(skill_id: &str) -> Result<(), String> {
    if skill_id.trim().is_empty() {
        return Err("Skill mutation requires a skill ID.".to_string());
    }
    Ok(())
}
