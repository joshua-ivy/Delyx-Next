use crate::approval::ApprovalEngine;
use crate::approval_bridge::ApprovalBridgeState;
use crate::external_agent::{ExternalAgentBridge, ExternalAgentKind};
use crate::external_agent_command_contracts::{
    build_external_agent_command_contract, ExternalAgentCommandContract,
};
use crate::external_agent_run_bridge_keys::{external_agent_error, permission_mode};
use crate::external_agent_run_bridge_types::{artifact_view, run_request, validate_request};
pub use crate::external_agent_run_bridge_types::{
    ExternalAgentCodexRunRequest, ExternalAgentEventView, ExternalAgentRunArtifactView,
};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct ExternalAgentRunBridgeState {
    store: Mutex<ExternalAgentRunBridgeStore>,
    database_path: Option<PathBuf>,
}

impl ExternalAgentRunBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        let store = crate::external_agent_run_persistence::load_from_path(&database_path)?;
        Ok(Self {
            store: Mutex::new(store),
            database_path: Some(database_path),
        })
    }

    fn save_if_persistent(&self, store: &ExternalAgentRunBridgeStore) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::external_agent_run_persistence::save_to_path(store, path),
            None => Ok(()),
        }
    }
}

#[derive(Default)]
pub struct ExternalAgentRunBridgeStore {
    pub(crate) artifacts: Vec<ExternalAgentRunArtifactView>,
    pub(crate) next_id: usize,
}

// The CLI agent runs to completion (possibly minutes); run it on a blocking
// thread so the Tauri main thread — and the webview — stay responsive. The
// AppHandle moves into the task because `tauri::State` borrows cannot.
#[tauri::command]
pub async fn external_agent_run_codex(
    app: tauri::AppHandle,
    request: ExternalAgentCodexRunRequest,
) -> Result<ExternalAgentRunArtifactView, String> {
    run_kind_agent_async(app, ExternalAgentKind::CodexCli, request).await
}

#[tauri::command]
pub async fn external_agent_run_claude(
    app: tauri::AppHandle,
    request: ExternalAgentCodexRunRequest,
) -> Result<ExternalAgentRunArtifactView, String> {
    run_kind_agent_async(app, ExternalAgentKind::ClaudeCode, request).await
}

async fn run_kind_agent_async(
    app: tauri::AppHandle,
    kind: ExternalAgentKind,
    request: ExternalAgentCodexRunRequest,
) -> Result<ExternalAgentRunArtifactView, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use tauri::Manager;
        // Snapshot the planned files before the worker runs so its edits can be
        // promoted into a reviewable, restorable Delyx patch afterwards.
        let promote = request.capture_diff
            && request.permission_mode == "workspace_write"
            && !request.changed_files.is_empty();
        let planned_paths: Vec<std::path::PathBuf> =
            request.changed_files.iter().map(PathBuf::from).collect();
        let pre_snapshot = promote
            .then(|| crate::external_agent_diff::snapshot_external_agent_diff(&planned_paths));
        let run_id = request.run_id.clone();
        let approval_id = request.external_approval_id.clone();

        let state = app.state::<ExternalAgentRunBridgeState>();
        let approvals = app.state::<ApprovalBridgeState>();
        let view = approvals.with_engine(|engine| {
            let mut store = state
                .store
                .lock()
                .map_err(|_| "External agent run bridge lock failed.".to_string())?;
            let view = run_kind_agent_record(kind, &mut store, engine, request)?;
            state.save_if_persistent(&store)?;
            Ok::<_, String>(view)
        })??;

        // Promote real changes into the patch store (applied + checkpointed) so
        // the existing diff review/restore UI owns the worker's edits.
        if let Some(snapshot) = pre_snapshot.filter(|_| view.status != "failed") {
            let changes = crate::external_agent_diff::external_diff_file_changes(&snapshot);
            if !changes.is_empty() {
                let patches = app.state::<crate::patch_bridge::PatchBridgeState>();
                let mut patch_store = patches
                    .store
                    .lock()
                    .map_err(|_| "Patch bridge lock failed.".to_string())?;
                crate::external_agent_patch_promotion::promote_worker_diff_to_patch(
                    &mut patch_store,
                    &run_id,
                    &approval_id,
                    &changes,
                );
                patches.save_if_persistent(&patch_store)?;
            }
        }
        Ok(view)
    })
    .await
    .map_err(|error| format!("External agent task failed: {error}"))?
}

#[tauri::command]
pub fn external_agent_run_snapshot(
    state: tauri::State<ExternalAgentRunBridgeState>,
    run_id: String,
) -> Result<Vec<ExternalAgentRunArtifactView>, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "External agent run bridge lock failed.".to_string())?;
    Ok(external_agent_run_snapshot_from_store(&store, &run_id))
}

pub fn run_kind_agent_record(
    kind: ExternalAgentKind,
    store: &mut ExternalAgentRunBridgeStore,
    approvals: &ApprovalEngine,
    request: ExternalAgentCodexRunRequest,
) -> Result<ExternalAgentRunArtifactView, String> {
    validate_request(&request)?;
    let contract = build_external_agent_command_contract(
        kind,
        &request.task,
        PathBuf::from(&request.working_directory),
        permission_mode(&request.permission_mode)?,
    )
    .map_err(external_agent_error)?;
    let roots = request
        .approved_roots
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let mut bridge = ExternalAgentBridge::new(roots).map_err(external_agent_error)?;
    run_contract_agent_record(store, approvals, request, contract, &mut bridge)
}

pub fn run_contract_agent_record(
    store: &mut ExternalAgentRunBridgeStore,
    approvals: &ApprovalEngine,
    request: ExternalAgentCodexRunRequest,
    contract: ExternalAgentCommandContract,
    bridge: &mut ExternalAgentBridge,
) -> Result<ExternalAgentRunArtifactView, String> {
    validate_request(&request)?;
    let created_at_ms = request.created_at_ms;
    let artifact = bridge
        .run_approved_worker(run_request(request, contract), created_at_ms, approvals)
        .map_err(external_agent_error)?;
    store.next_id += 1;
    let view = artifact_view(&artifact, format!("external-agent-run-{}", store.next_id));
    store.artifacts.push(view.clone());
    Ok(view)
}

pub fn external_agent_run_snapshot_from_store(
    store: &ExternalAgentRunBridgeStore,
    run_id: &str,
) -> Vec<ExternalAgentRunArtifactView> {
    store
        .artifacts
        .iter()
        .filter(|artifact| artifact.run_id == run_id)
        .cloned()
        .collect()
}
