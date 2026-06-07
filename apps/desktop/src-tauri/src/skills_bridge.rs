use crate::local_store_bridge::LocalStoreBridgeState;
use crate::skills::{SkillManifest, SkillPermissions, SkillRegistry, SkillStatus, SkillTrust};
use serde::Serialize;
use std::path::Path;

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

#[tauri::command]
pub fn skill_snapshot(state: tauri::State<LocalStoreBridgeState>) -> Result<SkillStateView, String> {
    skill_snapshot_from_path(state.database_path())
}

pub fn skill_snapshot_from_path(path: &Path) -> Result<SkillStateView, String> {
    let registry = crate::skills_persistence::load_from_path(path)?;
    Ok(skill_snapshot_from_registry(&registry))
}

pub fn skill_snapshot_from_registry(registry: &SkillRegistry) -> SkillStateView {
    SkillStateView { skills: registry.skills().iter().map(skill_view).collect() }
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

fn trust_key(trust: SkillTrust) -> &'static str {
    match trust {
        SkillTrust::Local => "local",
        SkillTrust::ThirdParty => "third_party",
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
