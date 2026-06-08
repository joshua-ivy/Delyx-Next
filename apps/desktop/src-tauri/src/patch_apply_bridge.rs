use crate::approval::{ApprovalEngine, RiskyAction};
use crate::patch::{CheckpointFile, PatchEngine, PatchFileInput, PatchInput};
use crate::patch_bridge::{PatchBridgeStore, PatchCheckpointFileView, PatchProposalView};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchApplyRequest {
    pub proposal_id: String,
    pub approved_roots: Vec<String>,
    pub created_at_ms: u64,
}

pub fn apply_patch_record(
    store: &mut PatchBridgeStore,
    approvals: &ApprovalEngine,
    request: PatchApplyRequest,
) -> Result<PatchProposalView, String> {
    validate_request(&request)?;
    let index = store
        .records
        .iter()
        .position(|proposal| proposal.id == request.proposal_id)
        .ok_or_else(|| "Patch proposal not found.".to_string())?;
    let proposal = store.records[index].clone();
    if proposal.status != "proposed" {
        return Err("Patch proposal must be proposed before apply.".to_string());
    }
    approvals
        .assert_can_execute_action_for_run(
            &proposal.approval_id,
            request.created_at_ms,
            RiskyAction::FileWrite,
            &proposal.run_id,
        )
        .map_err(|error| format!("Patch apply approval blocked: {error:?}"))?;
    ensure_files_unchanged(&proposal)?;

    let mut engine =
        PatchEngine::new(root_paths(&request)).map_err(|error| format!("{error:?}"))?;
    let runtime_patch = engine
        .propose_patch(PatchInput {
            approval_id: proposal.approval_id.clone(),
            files: proposal.files.iter().map(file_input).collect(),
            run_id: proposal.run_id.clone(),
        })
        .map_err(|error| format!("{error:?}"))?;
    let checkpoint = engine
        .apply_approved_patch(&runtime_patch.id, request.created_at_ms, approvals)
        .map_err(|error| format!("{error:?}"))?;

    let mut applied = proposal;
    applied.status = "applied".to_string();
    applied.checkpoint_id = Some(format!("{}-{}", applied.id, checkpoint.id));
    applied.checkpoint_files = checkpoint.files.iter().map(checkpoint_file_view).collect();
    store.records[index] = applied.clone();
    Ok(applied)
}

fn validate_request(request: &PatchApplyRequest) -> Result<(), String> {
    if request.proposal_id.trim().is_empty() || request.created_at_ms == 0 {
        return Err("Patch apply requires proposal ID and clock.".to_string());
    }
    if request.approved_roots.is_empty() {
        return Err("Patch apply requires at least one approved root.".to_string());
    }
    Ok(())
}

fn ensure_files_unchanged(proposal: &PatchProposalView) -> Result<(), String> {
    for file in &proposal.files {
        let current = fs::read_to_string(&file.path).unwrap_or_default();
        if current != file.before {
            return Err("Patch apply blocked because a file changed since proposal.".to_string());
        }
    }
    Ok(())
}

fn root_paths(request: &PatchApplyRequest) -> Vec<PathBuf> {
    request.approved_roots.iter().map(PathBuf::from).collect()
}

fn file_input(file: &crate::patch_bridge::PatchFileView) -> PatchFileInput {
    PatchFileInput {
        after: file.after.clone(),
        path: PathBuf::from(&file.path),
    }
}

fn checkpoint_file_view(file: &CheckpointFile) -> PatchCheckpointFileView {
    PatchCheckpointFileView {
        contents: file.contents.clone(),
        path: file.path.display().to_string(),
    }
}
