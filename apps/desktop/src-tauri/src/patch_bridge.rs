use crate::patch::{DiffLine, DiffLineKind, PatchEngine, PatchFileInput, PatchProposal};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct PatchBridgeState {
    store: Mutex<PatchBridgeStore>,
    database_path: Option<PathBuf>,
}

#[derive(Default)]
pub struct PatchBridgeStore {
    pub(crate) next_patch_id: usize,
    pub(crate) records: Vec<PatchProposalView>,
}

impl PatchBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        let store = crate::patch_persistence::load_from_path(&database_path)?;
        Ok(Self {
            store: Mutex::new(store),
            database_path: Some(database_path),
        })
    }

    fn save_if_persistent(&self, store: &PatchBridgeStore) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::patch_persistence::save_to_path(store, path),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchProposalRequest {
    pub client_id: String,
    pub run_id: String,
    pub approval_id: String,
    pub approved_roots: Vec<String>,
    pub files: Vec<PatchFileRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchFileRequest {
    pub path: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchProposalView {
    pub id: String,
    pub run_id: String,
    pub approval_id: String,
    pub status: String,
    pub checkpoint_id: Option<String>,
    pub checkpoint_files: Vec<PatchCheckpointFileView>,
    pub files: Vec<PatchFileView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchFileView {
    pub path: String,
    pub before: String,
    pub after: String,
    pub diff: Vec<DiffLineView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchCheckpointFileView {
    pub path: String,
    pub contents: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLineView {
    pub kind: String,
    pub text: String,
}

#[tauri::command]
pub fn patch_propose(
    state: tauri::State<PatchBridgeState>,
    request: PatchProposalRequest,
) -> Result<PatchProposalView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Patch bridge lock failed.".to_string())?;
    let proposal = propose_patch_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(proposal)
}

#[tauri::command]
pub fn patch_apply_approved(
    state: tauri::State<PatchBridgeState>,
    approvals: tauri::State<crate::approval_bridge::ApprovalBridgeState>,
    request: crate::patch_apply_bridge::PatchApplyRequest,
) -> Result<PatchProposalView, String> {
    approvals.with_engine(|engine| {
        let mut store = state
            .store
            .lock()
            .map_err(|_| "Patch bridge lock failed.".to_string())?;
        let proposal = crate::patch_apply_bridge::apply_patch_record(&mut store, engine, request)?;
        state.save_if_persistent(&store)?;
        Ok(proposal)
    })?
}

#[tauri::command]
pub fn patch_snapshot(
    state: tauri::State<PatchBridgeState>,
    run_id: String,
) -> Result<Vec<PatchProposalView>, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "Patch bridge lock failed.".to_string())?;
    Ok(patch_snapshot_from_store(&store, &run_id))
}

pub fn propose_patch_record(
    store: &mut PatchBridgeStore,
    request: PatchProposalRequest,
) -> Result<PatchProposalView, String> {
    validate_request(&request)?;
    if !request.client_id.trim().is_empty() {
        if let Some(existing) = store
            .records
            .iter()
            .find(|record| record.id == request.client_id)
        {
            return Ok(existing.clone());
        }
    }
    let client_id = request.client_id.clone();
    let roots = request
        .approved_roots
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let mut engine = PatchEngine::new(roots).map_err(|error| format!("{error:?}"))?;
    let proposal = engine
        .propose_patch(crate::patch::PatchInput {
            approval_id: request.approval_id,
            files: request.files.into_iter().map(file_input).collect(),
            run_id: request.run_id,
        })
        .map_err(|error| format!("{error:?}"))?;
    let id = if client_id.trim().is_empty() {
        store.next_patch_id += 1;
        format!("patch-{}", store.next_patch_id)
    } else {
        client_id
    };
    let view = patch_view(&proposal, id);
    store.records.push(view.clone());
    Ok(view)
}

pub fn patch_snapshot_from_store(store: &PatchBridgeStore, run_id: &str) -> Vec<PatchProposalView> {
    store
        .records
        .iter()
        .filter(|record| record.run_id == run_id)
        .cloned()
        .collect()
}

fn validate_request(request: &PatchProposalRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty() || request.approval_id.trim().is_empty() {
        return Err("Patch proposal requires run and approval IDs.".to_string());
    }
    if request.approved_roots.is_empty() {
        return Err("Patch proposal requires at least one approved root.".to_string());
    }
    Ok(())
}

fn file_input(file: PatchFileRequest) -> PatchFileInput {
    PatchFileInput {
        after: file.after,
        path: PathBuf::from(file.path),
    }
}

fn patch_view(proposal: &PatchProposal, id: String) -> PatchProposalView {
    PatchProposalView {
        approval_id: proposal.approval_id.clone(),
        checkpoint_id: proposal.checkpoint_id.clone(),
        checkpoint_files: Vec::new(),
        files: proposal.files.iter().map(file_view).collect(),
        id,
        run_id: proposal.run_id.clone(),
        status: "proposed".to_string(),
    }
}

fn file_view(file: &crate::patch::PatchFile) -> PatchFileView {
    PatchFileView {
        after: file.after.clone(),
        before: file.before.clone(),
        diff: file.diff.iter().map(diff_view).collect(),
        path: file.path.display().to_string(),
    }
}

fn diff_view(line: &DiffLine) -> DiffLineView {
    DiffLineView {
        kind: diff_kind(line.kind).to_string(),
        text: line.text.clone(),
    }
}

fn diff_kind(kind: DiffLineKind) -> &'static str {
    match kind {
        DiffLineKind::Added => "added",
        DiffLineKind::Context => "context",
        DiffLineKind::Removed => "removed",
    }
}
