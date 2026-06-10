use crate::approval::ApprovalEngine;
use crate::approval_bridge::ApprovalBridgeState;
use crate::external_agent::{
    ExternalAgentBridge, ExternalAgentCapturePlan, ExternalAgentEvent, ExternalAgentKind,
    ExternalAgentRunArtifact, ExternalAgentRunRequest, ExternalAgentScope, ExternalAgentTaskPolicy,
};
use crate::external_agent_command_contracts::{
    build_external_agent_command_contract, ExternalAgentCommandContract,
    ExternalAgentPermissionMode,
};
use crate::external_agent_run_bridge_keys::{
    command_label, event_kind_key, external_agent_error, permission_mode, status_key,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentCodexRunRequest {
    pub run_id: String,
    pub external_approval_id: String,
    pub terminal_approval_id: String,
    pub task: String,
    pub working_directory: String,
    pub approved_roots: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub permission_mode: String,
    pub timeout_ms: u64,
    pub created_at_ms: u64,
    pub checkpoint_id: Option<String>,
    pub worktree_id: Option<String>,
    pub capture_diff: bool,
    pub changed_files: Vec<String>,
    pub test_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentRunArtifactView {
    pub id: String,
    pub run_id: String,
    pub adapter_id: String,
    pub status: String,
    pub scope: String,
    pub transcript: Vec<ExternalAgentEventView>,
    pub terminal_output: String,
    pub diff_summary: Option<String>,
    pub test_artifact_ids: Vec<String>,
    pub review_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentEventView {
    pub kind: String,
    pub message: String,
    pub timestamp: String,
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

fn run_request(
    request: ExternalAgentCodexRunRequest,
    contract: ExternalAgentCommandContract,
) -> ExternalAgentRunRequest {
    let allowed_paths = if request.allowed_paths.is_empty() {
        vec![PathBuf::from(&request.working_directory)]
    } else {
        request.allowed_paths.iter().map(PathBuf::from).collect()
    };
    let command_label = command_label(&contract.command.program, &contract.command.args);
    let requires_isolation = contract.permission_mode
        == ExternalAgentPermissionMode::WorkspaceWrite
        || request.capture_diff;
    ExternalAgentRunRequest {
        adapter_id: contract.adapter_id,
        allowed_tools: contract.required_delyx_tools.clone(),
        approval_id: request.external_approval_id,
        capture_plan: ExternalAgentCapturePlan {
            capture_diff: request.capture_diff,
            changed_files: request.changed_files.iter().map(PathBuf::from).collect(),
            commands: vec![command_label],
            test_artifact_ids: request.test_artifact_ids,
        },
        run_id: request.run_id,
        requires_isolation,
        scope: ExternalAgentScope {
            allowed_paths,
            checkpoint_id: request.checkpoint_id,
            project_root: PathBuf::from(request.working_directory),
            worktree_id: request.worktree_id,
        },
        task: request.task,
        task_policy: ExternalAgentTaskPolicy {
            allowed_tools: contract.required_delyx_tools,
            approval_scope: "Launch external agent inside the approved project root.".to_string(),
        },
        terminal_approval_id: Some(request.terminal_approval_id),
        timeout_ms: request.timeout_ms,
        parse_stream_json: contract.transcript_format == "stream-json",
        worker_command: Some(contract.command),
    }
}

fn validate_request(request: &ExternalAgentCodexRunRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty() || request.task.trim().is_empty() {
        return Err("External agent launch requires a run ID and task.".to_string());
    }
    if request.external_approval_id.trim().is_empty()
        || request.terminal_approval_id.trim().is_empty()
    {
        return Err(
            "External agent launch requires external-agent and terminal-command approval IDs."
                .to_string(),
        );
    }
    if request.working_directory.trim().is_empty() || request.approved_roots.is_empty() {
        return Err(
            "External agent launch requires a working directory and approved root.".to_string(),
        );
    }
    Ok(())
}

fn artifact_view(artifact: &ExternalAgentRunArtifact, id: String) -> ExternalAgentRunArtifactView {
    ExternalAgentRunArtifactView {
        adapter_id: artifact.adapter_id.clone(),
        diff_summary: artifact.diff_summary.clone(),
        id,
        review_required: artifact.review_required,
        run_id: artifact.run_id.clone(),
        scope: scope_summary(&artifact.scope),
        status: status_key(artifact.status).to_string(),
        terminal_output: artifact.terminal_output.clone(),
        test_artifact_ids: artifact.test_artifact_ids.clone(),
        transcript: artifact.transcript.iter().map(event_view).collect(),
    }
}

fn event_view(event: &ExternalAgentEvent) -> ExternalAgentEventView {
    ExternalAgentEventView {
        kind: event_kind_key(event.kind).to_string(),
        message: event.message.clone(),
        timestamp: event.timestamp.to_string(),
    }
}

fn scope_summary(scope: &ExternalAgentScope) -> String {
    let isolation = scope
        .checkpoint_id
        .as_ref()
        .map(|id| format!("checkpoint {id}"))
        .or_else(|| {
            scope
                .worktree_id
                .as_ref()
                .map(|id| format!("worktree {id}"))
        })
        .unwrap_or_else(|| "no isolation".to_string());
    format!(
        "root: {}; isolation: {}",
        scope.project_root.display(),
        isolation
    )
}
