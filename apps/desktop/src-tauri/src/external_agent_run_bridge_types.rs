use crate::external_agent::{
    ExternalAgentCapturePlan, ExternalAgentEvent, ExternalAgentRunArtifact,
    ExternalAgentRunRequest, ExternalAgentScope, ExternalAgentTaskPolicy,
};
use crate::external_agent_command_contracts::{
    ExternalAgentCommandContract, ExternalAgentPermissionMode,
};
use crate::external_agent_run_bridge_keys::{command_label, event_kind_key, status_key};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

pub(crate) fn run_request(
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

pub(crate) fn validate_request(request: &ExternalAgentCodexRunRequest) -> Result<(), String> {
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

pub(crate) fn artifact_view(
    artifact: &ExternalAgentRunArtifact,
    id: String,
) -> ExternalAgentRunArtifactView {
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
