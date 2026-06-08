use crate::release::ReleaseSmokeStatus;
use serde::Deserialize;

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

pub(crate) fn validate_profile_request(request: &ReleaseProfileSaveRequest) -> Result<(), String> {
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

pub(crate) fn validate_smoke_request(request: &ReleaseSmokeCaptureRequest) -> Result<(), String> {
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

pub(crate) fn parse_smoke_status(value: &str) -> Result<ReleaseSmokeStatus, String> {
    match value {
        "failed" => Ok(ReleaseSmokeStatus::Failed),
        "passed" => Ok(ReleaseSmokeStatus::Passed),
        _ => Err("Release smoke status must be passed or failed.".to_string()),
    }
}
