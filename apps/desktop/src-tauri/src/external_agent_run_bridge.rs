use crate::approval::ApprovalEngine;
use crate::approval_bridge::ApprovalBridgeState;
use crate::external_agent::{
    ExternalAgentBridge, ExternalAgentCapturePlan, ExternalAgentError, ExternalAgentEvent,
    ExternalAgentEventKind, ExternalAgentKind, ExternalAgentRunArtifact, ExternalAgentRunRequest,
    ExternalAgentRunStatus, ExternalAgentScope, ExternalAgentTaskPolicy,
};
use crate::external_agent_command_contracts::{
    build_external_agent_command_contract, ExternalAgentCommandContract, ExternalAgentPermissionMode,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct ExternalAgentRunBridgeState {
    store: Mutex<ExternalAgentRunBridgeStore>,
}

#[derive(Default)]
pub struct ExternalAgentRunBridgeStore {
    artifacts: Vec<ExternalAgentRunArtifactView>,
    next_id: usize,
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

#[tauri::command]
pub fn external_agent_run_codex(
    state: tauri::State<ExternalAgentRunBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: ExternalAgentCodexRunRequest,
) -> Result<ExternalAgentRunArtifactView, String> {
    approvals.with_engine(|engine| {
        let mut store = state.store.lock().map_err(|_| "External agent run bridge lock failed.".to_string())?;
        run_codex_agent_record(&mut store, engine, request)
    })?
}

#[tauri::command]
pub fn external_agent_run_snapshot(
    state: tauri::State<ExternalAgentRunBridgeState>,
    run_id: String,
) -> Result<Vec<ExternalAgentRunArtifactView>, String> {
    let store = state.store.lock().map_err(|_| "External agent run bridge lock failed.".to_string())?;
    Ok(external_agent_run_snapshot_from_store(&store, &run_id))
}

pub fn run_codex_agent_record(
    store: &mut ExternalAgentRunBridgeStore,
    approvals: &ApprovalEngine,
    request: ExternalAgentCodexRunRequest,
) -> Result<ExternalAgentRunArtifactView, String> {
    validate_request(&request)?;
    let contract = build_external_agent_command_contract(
        ExternalAgentKind::CodexCli,
        &request.task,
        PathBuf::from(&request.working_directory),
        permission_mode(&request.permission_mode)?,
    )
    .map_err(external_agent_error)?;
    let roots = request.approved_roots.iter().map(PathBuf::from).collect::<Vec<_>>();
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
    store.artifacts.iter().filter(|artifact| artifact.run_id == run_id).cloned().collect()
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
    let requires_isolation = contract.permission_mode == ExternalAgentPermissionMode::WorkspaceWrite || request.capture_diff;
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
            approval_scope: "Launch Codex CLI inside the approved project root.".to_string(),
        },
        terminal_approval_id: Some(request.terminal_approval_id),
        timeout_ms: request.timeout_ms,
        worker_command: Some(contract.command),
    }
}

fn validate_request(request: &ExternalAgentCodexRunRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty() || request.task.trim().is_empty() {
        return Err("Codex launch requires a run ID and task.".to_string());
    }
    if request.external_approval_id.trim().is_empty() || request.terminal_approval_id.trim().is_empty() {
        return Err("Codex launch requires external-agent and terminal-command approval IDs.".to_string());
    }
    if request.working_directory.trim().is_empty() || request.approved_roots.is_empty() {
        return Err("Codex launch requires a working directory and approved root.".to_string());
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
        .or_else(|| scope.worktree_id.as_ref().map(|id| format!("worktree {id}")))
        .unwrap_or_else(|| "no isolation".to_string());
    format!("root: {}; isolation: {}", scope.project_root.display(), isolation)
}

fn permission_mode(value: &str) -> Result<ExternalAgentPermissionMode, String> {
    match value {
        "read_only" => Ok(ExternalAgentPermissionMode::ReadOnly),
        "workspace_write" => Ok(ExternalAgentPermissionMode::WorkspaceWrite),
        _ => Err("Unsupported Codex permission mode.".to_string()),
    }
}

fn command_label(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string()).chain(args.iter().cloned()).collect::<Vec<_>>().join(" ")
}

fn status_key(status: ExternalAgentRunStatus) -> &'static str {
    match status {
        ExternalAgentRunStatus::Accepted => "accepted",
        ExternalAgentRunStatus::Blocked => "blocked",
        ExternalAgentRunStatus::Completed => "completed",
        ExternalAgentRunStatus::Failed => "failed",
        ExternalAgentRunStatus::Reverted => "reverted",
    }
}

fn event_kind_key(kind: ExternalAgentEventKind) -> &'static str {
    match kind {
        ExternalAgentEventKind::Command => "command",
        ExternalAgentEventKind::Completed => "completed",
        ExternalAgentEventKind::DiffCaptured => "diff_captured",
        ExternalAgentEventKind::Failed => "failed",
        ExternalAgentEventKind::FileChanged => "file_changed",
        ExternalAgentEventKind::ReviewDecision => "review_decision",
        ExternalAgentEventKind::Started => "started",
        ExternalAgentEventKind::Stderr => "stderr",
        ExternalAgentEventKind::Stdout => "stdout",
        ExternalAgentEventKind::TestResult => "test_result",
    }
}

fn external_agent_error(error: ExternalAgentError) -> String {
    match error {
        ExternalAgentError::AdapterUnavailable => "Codex CLI adapter is unavailable.".to_string(),
        ExternalAgentError::EmptyTask => "Codex launch requires a non-empty task.".to_string(),
        ExternalAgentError::Io(message) => format!("Codex worker command failed to start: {message}"),
        ExternalAgentError::MissingIsolation => {
            "Codex launch requires a checkpoint or isolated worktree before execution.".to_string()
        }
        ExternalAgentError::MissingTerminalApproval => {
            "Codex launch requires a terminal-command approval before execution.".to_string()
        }
        error => format!("{error:?}"),
    }
}
