use crate::release::{
    check_signing, default_release_profile, export_support_bundle, ReleaseProfile, SigningPolicy,
    SigningStatus, SupportBundle, UpdateMetadata,
};
use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseStateView {
    pub platform: String,
    pub bundle_target: String,
    pub installer: String,
    pub smoke_status: String,
    pub signing: SigningStateView,
    pub support_bundle: SupportBundleStateView,
    pub update_metadata: UpdateMetadataStateView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SigningStateView {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportBundleStateView {
    pub export_status: String,
    pub secret_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadataStateView {
    pub status: String,
    pub channel: String,
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

pub fn release_snapshot_from_path(path: &Path) -> Result<ReleaseStateView, String> {
    Ok(release_snapshot_from_store(&load_store_from_path(path)?))
}

pub fn release_snapshot_from_store(store: &ReleaseBridgeStore) -> ReleaseStateView {
    release_snapshot_from_parts(&store.profile, store.support_bundle.as_ref())
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

pub fn release_snapshot_from_parts(
    profile: &ReleaseProfile,
    support_bundle: Option<&SupportBundle>,
) -> ReleaseStateView {
    let signing = check_signing(&profile.signing);
    ReleaseStateView {
        platform: profile.target_platform.clone(),
        bundle_target: profile.bundle_target.clone(),
        installer: installer_label(signing.status).to_string(),
        smoke_status: "not_loaded".to_string(),
        signing: SigningStateView {
            status: signing_status_key(signing.status).to_string(),
            message: signing.message,
        },
        support_bundle: support_bundle_view(support_bundle),
        update_metadata: UpdateMetadataStateView {
            status: if profile.update_metadata.published {
                "published"
            } else {
                "placeholder"
            }
            .to_string(),
            channel: profile.update_metadata.channel.clone(),
        },
    }
}

fn support_bundle_view(bundle: Option<&SupportBundle>) -> SupportBundleStateView {
    match bundle {
        Some(bundle) => SupportBundleStateView {
            export_status: "available".to_string(),
            secret_policy: bundle.secret_policy.clone(),
        },
        None => SupportBundleStateView {
            export_status: "not_exported".to_string(),
            secret_policy: "No support bundle export is loaded in this UI session.".to_string(),
        },
    }
}

fn signing_status_key(status: SigningStatus) -> &'static str {
    match status {
        SigningStatus::Incomplete => "missing_certificate",
        SigningStatus::Ready => "signed",
        SigningStatus::UnsignedDev => "unsigned_dev",
    }
}

fn installer_label(status: SigningStatus) -> &'static str {
    match status {
        SigningStatus::Ready => "signed installer",
        SigningStatus::Incomplete => "signing incomplete",
        SigningStatus::UnsignedDev => "unsigned dev installer",
    }
}

fn load_store_from_path(path: &Path) -> Result<ReleaseBridgeStore, String> {
    Ok(ReleaseBridgeStore {
        profile: crate::release_persistence::load_profile_from_path(path)?
            .unwrap_or_else(default_release_profile),
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
