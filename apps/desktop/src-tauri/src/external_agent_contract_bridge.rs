use crate::external_agent::{ExternalAgentError, ExternalAgentKind};
use crate::external_agent_command_contracts::{
    build_external_agent_command_contract, ExternalAgentPermissionMode,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentContractPreviewRequest {
    pub kind: String,
    pub task: String,
    pub working_directory: String,
    pub permission_mode: String,
    pub run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAgentContractPreviewView {
    pub id: String,
    pub run_id: String,
    pub adapter_id: String,
    pub status: String,
    pub permission_mode: String,
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub transcript_format: String,
    pub required_delyx_tools: Vec<String>,
    pub safety_summary: String,
}

#[tauri::command]
pub fn external_agent_contract_preview(
    request: ExternalAgentContractPreviewRequest,
) -> Result<ExternalAgentContractPreviewView, String> {
    preview_external_agent_contract(request).map_err(external_agent_error)
}

pub fn preview_external_agent_contract(
    request: ExternalAgentContractPreviewRequest,
) -> Result<ExternalAgentContractPreviewView, ExternalAgentError> {
    let run_id = checked_value(&request.run_id)?;
    let working_directory = checked_value(&request.working_directory)?;
    let permission_mode = permission_mode(&request.permission_mode)?;
    let contract = build_external_agent_command_contract(
        agent_kind(&request.kind)?,
        &request.task,
        PathBuf::from(&working_directory),
        permission_mode,
    )?;
    Ok(ExternalAgentContractPreviewView {
        id: format!("contract-{run_id}-{}", contract.adapter_id),
        run_id,
        adapter_id: contract.adapter_id,
        status: "draft".to_string(),
        permission_mode: permission_mode_key(permission_mode).to_string(),
        program: contract.command.program,
        args: contract.command.args,
        working_directory,
        transcript_format: contract.transcript_format,
        required_delyx_tools: contract.required_delyx_tools,
        safety_summary: contract.safety_summary,
    })
}

fn checked_value(value: &str) -> Result<String, ExternalAgentError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(ExternalAgentError::EmptyTask);
    }
    Ok(value)
}

fn agent_kind(value: &str) -> Result<ExternalAgentKind, ExternalAgentError> {
    match value {
        "codex_cli" => Ok(ExternalAgentKind::CodexCli),
        "claude_code" => Ok(ExternalAgentKind::ClaudeCode),
        _ => Err(ExternalAgentError::AdapterUnavailable),
    }
}

fn permission_mode(value: &str) -> Result<ExternalAgentPermissionMode, ExternalAgentError> {
    match value {
        "read_only" => Ok(ExternalAgentPermissionMode::ReadOnly),
        "workspace_write" => Ok(ExternalAgentPermissionMode::WorkspaceWrite),
        _ => Err(ExternalAgentError::AdapterUnavailable),
    }
}

fn permission_mode_key(permission_mode: ExternalAgentPermissionMode) -> &'static str {
    match permission_mode {
        ExternalAgentPermissionMode::ReadOnly => "read_only",
        ExternalAgentPermissionMode::WorkspaceWrite => "workspace_write",
    }
}

fn external_agent_error(error: ExternalAgentError) -> String {
    match error {
        ExternalAgentError::AdapterUnavailable => {
            "External agent adapter is unavailable.".to_string()
        }
        ExternalAgentError::EmptyTask => {
            "External agent contract requires a task, run, and working directory.".to_string()
        }
        _ => format!("{error:?}"),
    }
}
