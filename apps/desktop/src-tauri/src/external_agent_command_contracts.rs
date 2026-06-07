use crate::external_agent::{ExternalAgentError, ExternalAgentKind};
use crate::external_agent_terminal::ExternalAgentCommand;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalAgentPermissionMode {
    ReadOnly,
    WorkspaceWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentCommandContract {
    pub adapter_id: String,
    pub kind: ExternalAgentKind,
    pub permission_mode: ExternalAgentPermissionMode,
    pub command: ExternalAgentCommand,
    pub required_delyx_tools: Vec<String>,
    pub transcript_format: String,
    pub safety_summary: String,
}

pub fn build_external_agent_command_contract(
    kind: ExternalAgentKind,
    task: &str,
    working_directory: PathBuf,
    permission_mode: ExternalAgentPermissionMode,
) -> Result<ExternalAgentCommandContract, ExternalAgentError> {
    let task = task.trim();
    if task.is_empty() {
        return Err(ExternalAgentError::EmptyTask);
    }
    match kind {
        ExternalAgentKind::CodexCli => Ok(contract(
            "codex-cli",
            kind,
            permission_mode,
            ExternalAgentCommand {
                args: codex_args(task, permission_mode),
                program: "codex".to_string(),
                working_directory,
            },
            "jsonl",
        )),
        ExternalAgentKind::ClaudeCode => Ok(contract(
            "claude-code",
            kind,
            permission_mode,
            ExternalAgentCommand {
                args: claude_args(task, permission_mode),
                program: "claude".to_string(),
                working_directory,
            },
            "stream-json",
        )),
        ExternalAgentKind::GenericTerminal => Err(ExternalAgentError::AdapterUnavailable),
    }
}

fn contract(
    adapter_id: &str,
    kind: ExternalAgentKind,
    permission_mode: ExternalAgentPermissionMode,
    command: ExternalAgentCommand,
    transcript_format: &str,
) -> ExternalAgentCommandContract {
    ExternalAgentCommandContract {
        adapter_id: adapter_id.to_string(),
        kind,
        permission_mode,
        command,
        required_delyx_tools: vec!["external_agent".to_string(), "terminal_command".to_string()],
        safety_summary: safety_summary(permission_mode).to_string(),
        transcript_format: transcript_format.to_string(),
    }
}

fn codex_args(task: &str, permission_mode: ExternalAgentPermissionMode) -> Vec<String> {
    vec![
        "exec".to_string(),
        "--json".to_string(),
        "--sandbox".to_string(),
        codex_sandbox(permission_mode).to_string(),
        task.to_string(),
    ]
}

fn claude_args(task: &str, permission_mode: ExternalAgentPermissionMode) -> Vec<String> {
    vec![
        "-p".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--permission-mode".to_string(),
        claude_permission_mode(permission_mode).to_string(),
        "--tools".to_string(),
        claude_tools(permission_mode).to_string(),
        task.to_string(),
    ]
}

fn codex_sandbox(permission_mode: ExternalAgentPermissionMode) -> &'static str {
    match permission_mode {
        ExternalAgentPermissionMode::ReadOnly => "read-only",
        ExternalAgentPermissionMode::WorkspaceWrite => "workspace-write",
    }
}

fn claude_permission_mode(permission_mode: ExternalAgentPermissionMode) -> &'static str {
    match permission_mode {
        ExternalAgentPermissionMode::ReadOnly => "plan",
        ExternalAgentPermissionMode::WorkspaceWrite => "acceptEdits",
    }
}

fn claude_tools(permission_mode: ExternalAgentPermissionMode) -> &'static str {
    match permission_mode {
        ExternalAgentPermissionMode::ReadOnly => "Read",
        ExternalAgentPermissionMode::WorkspaceWrite => "Read,Edit",
    }
}

fn safety_summary(permission_mode: ExternalAgentPermissionMode) -> &'static str {
    match permission_mode {
        ExternalAgentPermissionMode::ReadOnly => {
            "Read-only command contract; launch still requires external_agent and terminal_command approvals."
        }
        ExternalAgentPermissionMode::WorkspaceWrite => {
            "Workspace-write command contract; launch still requires isolation, approvals, captured output, diff review, and rollback."
        }
    }
}
