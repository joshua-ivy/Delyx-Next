use crate::release::{
    check_signing, ReleaseProfile, ReleaseSmokeRecord, ReleaseSmokeStatus, SigningStatus,
    SupportBundle,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseStateView {
    pub platform: String,
    pub bundle_target: String,
    pub installer: String,
    pub smoke_status: String,
    pub smoke: ReleaseSmokeStateView,
    pub signing: SigningStateView,
    pub support_bundle: SupportBundleStateView,
    pub update_metadata: UpdateMetadataStateView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseSmokeStateView {
    pub status: String,
    pub detail: String,
    pub installer_path: Option<String>,
    pub command: Option<String>,
    pub captured_at: Option<String>,
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

pub fn release_snapshot_from_parts(
    profile: &ReleaseProfile,
    support_bundle: Option<&SupportBundle>,
    smoke: Option<&ReleaseSmokeRecord>,
) -> ReleaseStateView {
    let signing = check_signing(&profile.signing);
    ReleaseStateView {
        bundle_target: profile.bundle_target.clone(),
        installer: installer_label(signing.status).to_string(),
        platform: profile.target_platform.clone(),
        signing: SigningStateView {
            message: signing.message,
            status: signing_status_key(signing.status).to_string(),
        },
        smoke: smoke_view(smoke),
        smoke_status: smoke_status_key(smoke.map(|record| record.status)).to_string(),
        support_bundle: support_bundle_view(support_bundle),
        update_metadata: UpdateMetadataStateView {
            channel: profile.update_metadata.channel.clone(),
            status: if profile.update_metadata.published {
                "published"
            } else {
                "placeholder"
            }
            .to_string(),
        },
    }
}

fn smoke_view(smoke: Option<&ReleaseSmokeRecord>) -> ReleaseSmokeStateView {
    match smoke {
        Some(record) => ReleaseSmokeStateView {
            captured_at: Some(record.captured_at.clone()),
            command: Some(record.command.clone()),
            detail: record.detail.clone(),
            installer_path: Some(record.installer_path.clone()),
            status: smoke_status_key(Some(record.status)).to_string(),
        },
        None => ReleaseSmokeStateView {
            captured_at: None,
            command: None,
            detail: "No release smoke artifact loaded in this UI session.".to_string(),
            installer_path: None,
            status: "not_loaded".to_string(),
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

fn smoke_status_key(status: Option<ReleaseSmokeStatus>) -> &'static str {
    match status {
        Some(ReleaseSmokeStatus::Failed) => "failed",
        Some(ReleaseSmokeStatus::Passed) => "passed",
        None => "not_loaded",
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
        SigningStatus::Incomplete => "signing incomplete",
        SigningStatus::Ready => "signed installer",
        SigningStatus::UnsignedDev => "unsigned dev installer",
    }
}
