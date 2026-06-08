use crate::release::{
    default_release_profile, export_support_bundle, ReleaseProfile, ReleaseSmokeRecord,
    SigningPolicy, SupportBundle, SupportBundleFileExport, UpdateMetadata,
};
use crate::release_bridge_requests::{
    parse_smoke_status, validate_profile_request, validate_smoke_request,
};
use crate::release_bridge_views::{release_snapshot_from_parts, ReleaseStateView};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub use crate::release_bridge_requests::{
    ReleaseProfileSaveRequest, ReleaseSmokeCaptureRequest, SupportBundleExportRequest,
    SupportEntryRequest, SupportLogRequest,
};

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
    pub support_bundle_file_export: Option<SupportBundleFileExport>,
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

    fn save_file_export_if_persistent(
        &self,
        export: &SupportBundleFileExport,
    ) -> Result<(), String> {
        match &self.database_path {
            Some(path) => {
                crate::release_file_export_persistence::save_file_export_to_path(export, path)
            }
            None => Ok(()),
        }
    }
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
pub fn release_support_bundle_file_export(
    state: tauri::State<ReleaseBridgeState>,
    approvals: tauri::State<crate::approval_bridge::ApprovalBridgeState>,
    request: crate::release_support_export::SupportBundleFileExportRequest,
) -> Result<ReleaseStateView, String> {
    approvals.with_engine(|engine| {
        let mut store = state
            .store
            .lock()
            .map_err(|_| "Release bridge lock failed.".to_string())?;
        let view = crate::release_support_export::export_support_bundle_file_record(
            &mut store, engine, request,
        )?;
        let export = store
            .support_bundle_file_export
            .as_ref()
            .ok_or_else(|| "Support bundle file export failed.".to_string())?;
        state.save_file_export_if_persistent(export)?;
        Ok(view)
    })?
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
        store.support_bundle_file_export.as_ref(),
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
        support_bundle_file_export:
            crate::release_file_export_persistence::load_file_export_from_path(path)?,
        support_bundle: crate::release_persistence::load_support_bundle_from_path(path)?,
    })
}
