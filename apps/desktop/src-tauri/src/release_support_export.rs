use crate::approval::{ApprovalEngine, RiskyAction};
use crate::release::SupportBundleFileExport;
use crate::release_bridge::ReleaseBridgeStore;
use crate::release_bridge_views::ReleaseStateView;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportBundleFileExportRequest {
    pub run_id: String,
    pub approval_id: String,
    pub output_path: String,
    pub approved_roots: Vec<String>,
    pub exported_at: String,
    pub created_at_ms: u64,
}

pub fn export_support_bundle_file_record(
    store: &mut ReleaseBridgeStore,
    approvals: &ApprovalEngine,
    request: SupportBundleFileExportRequest,
) -> Result<ReleaseStateView, String> {
    validate_request(&request)?;
    approvals
        .assert_can_execute_action_for_run(
            &request.approval_id,
            request.created_at_ms,
            RiskyAction::FileWrite,
            &request.run_id,
        )
        .map_err(|error| format!("Support bundle file export approval blocked: {error:?}"))?;

    let bundle = store.support_bundle.as_ref().ok_or_else(|| {
        "Support bundle file export requires a generated support bundle.".to_string()
    })?;
    let output_path = checked_output_path(&request.output_path, &request.approved_roots)?;
    let payload = serde_json::to_vec_pretty(bundle).map_err(|error| error.to_string())?;
    fs::write(&output_path, &payload).map_err(|error| error.to_string())?;
    store.support_bundle_file_export = Some(SupportBundleFileExport {
        approval_id: request.approval_id,
        bytes_written: payload.len() as u64,
        exported_at: request.exported_at,
        path: display_path(&output_path),
        run_id: request.run_id,
    });
    Ok(crate::release_bridge::release_snapshot_from_store(store))
}

fn validate_request(request: &SupportBundleFileExportRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty()
        || request.approval_id.trim().is_empty()
        || request.output_path.trim().is_empty()
        || request.exported_at.trim().is_empty()
        || request.created_at_ms == 0
    {
        return Err(
            "Support bundle file export requires run, approval, path, timestamp, and clock."
                .to_string(),
        );
    }
    if request.approved_roots.is_empty() {
        return Err("Support bundle file export requires at least one approved root.".to_string());
    }
    Ok(())
}

fn checked_output_path(output_path: &str, approved_roots: &[String]) -> Result<PathBuf, String> {
    let output = normalized_target(Path::new(output_path))?;
    let roots = approved_roots
        .iter()
        .map(|root| canonical_root(root))
        .collect::<Result<Vec<_>, _>>()?;
    roots
        .iter()
        .any(|root| output.starts_with(root))
        .then_some(output)
        .ok_or_else(|| {
            "Support bundle file export path must stay inside an approved root.".to_string()
        })
}

fn normalized_target(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        if path.is_dir() {
            return Err("Support bundle file export path must be a file.".to_string());
        }
        return fs::canonicalize(path).map_err(|error| error.to_string());
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| "Support bundle file export parent folder must exist.".to_string())?;
    let name = path
        .file_name()
        .ok_or_else(|| "Support bundle file export path must be a file.".to_string())?;
    Ok(fs::canonicalize(parent)
        .map_err(|_| "Support bundle file export parent folder must exist.".to_string())?
        .join(name))
}

fn canonical_root(root: &str) -> Result<PathBuf, String> {
    if root.trim().is_empty() {
        return Err("Support bundle file export approved root must exist.".to_string());
    }
    fs::canonicalize(root)
        .map_err(|_| "Support bundle file export approved root must exist.".to_string())
}

fn display_path(path: &Path) -> String {
    let text = path.display().to_string();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
}
