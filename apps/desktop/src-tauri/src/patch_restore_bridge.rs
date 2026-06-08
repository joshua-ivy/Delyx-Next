use crate::approval::{ApprovalEngine, RiskyAction};
use crate::patch_bridge::{PatchBridgeStore, PatchProposalView};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchRestoreRequest {
    pub proposal_id: String,
    pub approval_id: String,
    pub approved_roots: Vec<String>,
    pub created_at_ms: u64,
}

pub fn restore_patch_record(
    store: &mut PatchBridgeStore,
    approvals: &ApprovalEngine,
    request: PatchRestoreRequest,
) -> Result<PatchProposalView, String> {
    validate_request(&request)?;
    let index = store
        .records
        .iter()
        .position(|proposal| proposal.id == request.proposal_id)
        .ok_or_else(|| "Patch proposal not found.".to_string())?;
    let proposal = store.records[index].clone();
    if proposal.status != "applied" {
        return Err("Patch proposal must be applied before restore.".to_string());
    }
    approvals
        .assert_can_execute_action_for_run(
            &request.approval_id,
            request.created_at_ms,
            RiskyAction::FileWrite,
            &proposal.run_id,
        )
        .map_err(|error| format!("Patch restore approval blocked: {error:?}"))?;
    ensure_current_matches_after(&proposal)?;
    restore_checkpoint_files(&proposal, &request.approved_roots)?;

    let mut restored = proposal;
    restored.status = "restored".to_string();
    restored.restore_approval_id = Some(request.approval_id);
    store.records[index] = restored.clone();
    Ok(restored)
}

fn validate_request(request: &PatchRestoreRequest) -> Result<(), String> {
    if request.proposal_id.trim().is_empty()
        || request.approval_id.trim().is_empty()
        || request.created_at_ms == 0
    {
        return Err("Patch restore requires proposal ID, approval ID, and clock.".to_string());
    }
    if request.approved_roots.is_empty() {
        return Err("Patch restore requires at least one approved root.".to_string());
    }
    Ok(())
}

fn ensure_current_matches_after(proposal: &PatchProposalView) -> Result<(), String> {
    for file in &proposal.files {
        let current = fs::read_to_string(&file.path).unwrap_or_default();
        if current != file.after {
            return Err("Patch restore blocked because a file changed since apply.".to_string());
        }
    }
    Ok(())
}

fn restore_checkpoint_files(
    proposal: &PatchProposalView,
    approved_roots: &[String],
) -> Result<(), String> {
    if proposal.checkpoint_files.is_empty() {
        return Err("Patch restore requires checkpoint file receipts.".to_string());
    }
    for file in &proposal.checkpoint_files {
        let path = checked_path(&file.path, approved_roots)?;
        match &file.contents {
            Some(contents) => fs::write(path, contents).map_err(|error| error.to_string())?,
            None if path.exists() => fs::remove_file(path).map_err(|error| error.to_string())?,
            None => {}
        }
    }
    Ok(())
}

fn checked_path(path: &str, approved_roots: &[String]) -> Result<PathBuf, String> {
    let normalized = normalized_path(Path::new(path))?;
    let roots = approved_roots
        .iter()
        .map(|root| fs::canonicalize(root).map_err(|_| "Patch restore approved root must exist."))
        .collect::<Result<Vec<_>, _>>()?;
    roots
        .iter()
        .any(|root| normalized.starts_with(root))
        .then_some(normalized)
        .ok_or_else(|| "Patch restore path must stay inside an approved root.".to_string())
}

fn normalized_path(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|error| error.to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "Patch restore parent path must exist.".to_string())?;
    let name = path
        .file_name()
        .ok_or_else(|| "Patch restore target path must be a file.".to_string())?;
    Ok(fs::canonicalize(parent)
        .map_err(|_| "Patch restore parent path must exist.".to_string())?
        .join(name))
}
