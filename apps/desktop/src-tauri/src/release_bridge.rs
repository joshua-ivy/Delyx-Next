use crate::release::{
    default_release_profile, export_support_bundle, ReleaseProfile, ReleaseSmokeRecord,
    ReleaseSmokeStatus, SigningPolicy, SupportBundle, UpdateMetadata,
};
use crate::release_bridge_views::{release_snapshot_from_parts, ReleaseStateView};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug)]
pub struct ReleaseBridgeState {
    store: Mutex<ReleaseBridgeStore>,
    database_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ReleaseBridgeStore {
    pub profile: ReleaseProfile,
    pub smoke: Option<ReleaseSmokeRecord>,
    pub support_bundle: Option<SupportBundle>,
}

impl ReleaseBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        Ok(Self {
            store: Mutex::new(load_store_from_path(&database_path)?),
            database_path: Some(database_path),
        })
    }

    fn save_profile_if_persistent(&self, profile: &ReleaseProfile) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::release_persistence::save_profile_to_path(profile, path),
            None => Ok(()),
        }
    }

    fn save_bundle_if_persistent(&self, bundle: &SupportBundle) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::release_persistence::save_support_bundle_to_path(bundle, path),
            None => Ok(()),
        }
    }

    fn save_smoke_if_persistent(&self, smoke: &ReleaseSmokeRecord) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::release_persistence::save_smoke_to_path(smoke, path),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseProfileSaveRequest {
    pub product_name: String,
    pub version: String,
    pub target_platform: String,
    pub bundle_target: String,
    pub certificate_thumbprint: Option<String>,
    pub digest_algorithm: Option<String>,
    pub timestamp_url: Option<String>,
    pub sign_command: Option<String>,
    pub tsp: bool,
    pub update_channel: String,
    pub update_published: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportBundleExportRequest {
    pub created_at_ms: u64,
    pub config: Vec<SupportEntryRequest>,
    pub logs: Vec<SupportLogRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseSmokeCaptureRequest {
    pub status: String,
    pub installer_path: String,
    pub command: String,
    pub captured_at: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportEntryRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportLogRequest {
    pub source: String,
    pub line: String,
}

#[tauri::command]
pub fn release_snapshot(
    state: tauri::State<ReleaseBridgeState>,
) -> Result<ReleaseStateView, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "Release bridge lock failed.".to_string())?;
    Ok(release_snapshot_from_store(&store))
}

#[tauri::command]
pub fn release_profile_save(
    state: tauri::State<ReleaseBridgeState>,
    request: ReleaseProfileSaveRequest,
) -> Result<ReleaseStateView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Release bridge lock failed.".to_string())?;
    let view = save_release_profile_record(&mut store, request)?;
    state.save_profile_if_persistent(&store.profile)?;
    Ok(view)
}

#[tauri::command]
pub fn release_support_bundle_export(
    state: tauri::State<ReleaseBridgeState>,
    request: SupportBundleExportRequest,
) -> Result<ReleaseStateView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Release bridge lock failed.".to_string())?;
    let view = export_support_bundle_record(&mut store, request)?;
    let bundle = store
        .support_bundle
        .as_ref()
        .ok_or_else(|| "Support bundle export failed.".to_string())?;
    state.save_bundle_if_persistent(bundle)?;
    Ok(view)
}

#[tauri::command]
pub fn release_smoke_capture(
    state: tauri::State<ReleaseBridgeState>,
    request: ReleaseSmokeCaptureRequest,
) -> Result<ReleaseStateView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Release bridge lock failed.".to_string())?;
    let view = capture_release_smoke_record(&mut store, request)?;
    let smoke = store
        .smoke
        .as_ref()
        .ok_or_else(|| "Release smoke capture failed.".to_string())?;
    state.save_smoke_if_persistent(smoke)?;
    Ok(view)
}

pub fn release_snapshot_from_path(path: &Path) -> Result<ReleaseStateView, String> {
    Ok(release_snapshot_from_store(&load_store_from_path(path)?))
}

pub fn release_snapshot_from_store(store: &ReleaseBridgeStore) -> ReleaseStateView {
    release_snapshot_from_parts(
        &store.profile,
        store.support_bundle.as_ref(),
        store.smoke.as_ref(),
    )
}

pub fn save_release_profile_record(
    store: &mut ReleaseBridgeStore,
    request: ReleaseProfileSaveRequest,
) -> Result<ReleaseStateView, String> {
    validate_profile_request(&request)?;
    store.profile = ReleaseProfile {
        bundle_target: request.bundle_target,
        product_name: request.product_name,
        signing: SigningPolicy {
            certificate_thumbprint: request.certificate_thumbprint,
            digest_algorithm: request.digest_algorithm,
            sign_command: request.sign_command,
            timestamp_url: request.timestamp_url,
            tsp: request.tsp,
        },
        target_platform: request.target_platform,
        update_metadata: UpdateMetadata {
            channel: request.update_channel,
            published: request.update_published,
        },
        version: request.version,
    };
    Ok(release_snapshot_from_store(store))
}

pub fn export_support_bundle_record(
    store: &mut ReleaseBridgeStore,
    request: SupportBundleExportRequest,
) -> Result<ReleaseStateView, String> {
    if request.created_at_ms == 0 {
        return Err("Support bundle export requires a creation timestamp.".to_string());
    }
    let config = request
        .config
        .iter()
        .map(|entry| (entry.key.as_str(), entry.value.as_str()))
        .collect();
    let logs = request
        .logs
        .iter()
        .map(|entry| (entry.source.as_str(), entry.line.as_str()))
        .collect();
    store.support_bundle = Some(export_support_bundle(
        &store.profile,
        config,
        logs,
        request.created_at_ms,
    ));
    Ok(release_snapshot_from_store(store))
}

pub fn capture_release_smoke_record(
    store: &mut ReleaseBridgeStore,
    request: ReleaseSmokeCaptureRequest,
) -> Result<ReleaseStateView, String> {
    validate_smoke_request(&request)?;
    store.smoke = Some(ReleaseSmokeRecord {
        captured_at: request.captured_at,
        command: request.command,
        detail: request.detail,
        installer_path: request.installer_path,
        status: parse_smoke_status(&request.status)?,
    });
    Ok(release_snapshot_from_store(store))
}

fn load_store_from_path(path: &Path) -> Result<ReleaseBridgeStore, String> {
    Ok(ReleaseBridgeStore {
        profile: crate::release_persistence::load_profile_from_path(path)?
            .unwrap_or_else(default_release_profile),
        smoke: crate::release_persistence::load_smoke_from_path(path)?,
        support_bundle: crate::release_persistence::load_support_bundle_from_path(path)?,
    })
}

fn validate_profile_request(request: &ReleaseProfileSaveRequest) -> Result<(), String> {
    if request.product_name.trim().is_empty()
        || request.version.trim().is_empty()
        || request.target_platform.trim().is_empty()
        || request.bundle_target.trim().is_empty()
        || request.update_channel.trim().is_empty()
    {
        return Err(
            "Release profile requires product, version, target, bundle, and update channel."
                .to_string(),
        );
    }
    Ok(())
}

fn validate_smoke_request(request: &ReleaseSmokeCaptureRequest) -> Result<(), String> {
    if request.installer_path.trim().is_empty()
        || request.command.trim().is_empty()
        || request.captured_at.trim().is_empty()
        || request.detail.trim().is_empty()
    {
        return Err(
            "Release smoke capture requires installer path, command, timestamp, and detail."
                .to_string(),
        );
    }
    parse_smoke_status(&request.status).map(|_| ())
}

fn parse_smoke_status(value: &str) -> Result<ReleaseSmokeStatus, String> {
    match value {
        "failed" => Ok(ReleaseSmokeStatus::Failed),
        "passed" => Ok(ReleaseSmokeStatus::Passed),
        _ => Err("Release smoke status must be passed or failed.".to_string()),
    }
}
