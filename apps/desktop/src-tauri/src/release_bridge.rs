use crate::local_store_bridge::LocalStoreBridgeState;
use crate::release::{
    check_signing, default_release_profile, ReleaseProfile, SigningStatus, SupportBundle,
};
use serde::Serialize;
use std::path::Path;

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

#[tauri::command]
pub fn release_snapshot(
    state: tauri::State<LocalStoreBridgeState>,
) -> Result<ReleaseStateView, String> {
    release_snapshot_from_path(state.database_path())
}

pub fn release_snapshot_from_path(path: &Path) -> Result<ReleaseStateView, String> {
    let profile = crate::release_persistence::load_profile_from_path(path)?
        .unwrap_or_else(default_release_profile);
    let support_bundle = crate::release_persistence::load_support_bundle_from_path(path)?;
    Ok(release_snapshot_from_parts(
        &profile,
        support_bundle.as_ref(),
    ))
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
