use crate::command_exec::{
    run_command_exec, CommandExecArtifact, CommandExecError, CommandExecEventKind,
    CommandExecRequest, CommandExecStatus,
};
use crate::external_agent::{
    ExternalAgentError, ExternalAgentEvent, ExternalAgentEventKind, ExternalAgentRunRequest,
    ExternalAgentRunStatus, ExternalAgentScope,
};
use crate::external_agent_scope::checked_scoped_path;
use std::path::{Path, PathBuf};

const TERMINAL_TOOL: &str = "terminal_command";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentCommand {
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalWorkerExecution {
    pub status: ExternalAgentRunStatus,
    pub terminal_output: Option<String>,
    pub transcript: Vec<ExternalAgentEvent>,
}

pub fn run_worker_command(
    request: &ExternalAgentRunRequest,
    scope: &ExternalAgentScope,
    now: u64,
) -> Result<ExternalWorkerExecution, ExternalAgentError> {
    let Some(command) = &request.worker_command else {
        return Ok(ExternalWorkerExecution { status: ExternalAgentRunStatus::Completed, terminal_output: None, transcript: Vec::new() });
    };
    if !request.allowed_tools.iter().any(|tool| tool == TERMINAL_TOOL) {
        return Err(ExternalAgentError::ToolNotAllowed(TERMINAL_TOOL.to_string()));
    }
    let working_directory = checked_working_directory(&command.working_directory, scope)?;
    let output = run_command(command, &working_directory, request, now)?;
    let status = status_from_exec(&output);
    Ok(ExternalWorkerExecution {
        status,
        terminal_output: Some(output.terminal_output()),
        transcript: output.events.iter().map(external_event).collect(),
    })
}

fn checked_working_directory(path: &Path, scope: &ExternalAgentScope) -> Result<PathBuf, ExternalAgentError> {
    checked_scoped_path(path, scope)
}

fn run_command(
    command: &ExternalAgentCommand,
    working_directory: &Path,
    request: &ExternalAgentRunRequest,
    now: u64,
) -> Result<CommandExecArtifact, ExternalAgentError> {
    run_command_exec(CommandExecRequest {
        approval_id: request.terminal_approval_id.clone().unwrap_or_default(),
        args: command.args.clone(),
        prepare_terminal: true,
        program: command.program.clone(),
        run_id: request.run_id.clone(),
        started_at_ms: now,
        timeout_ms: request.timeout_ms,
        working_directory: working_directory.to_path_buf(),
    })
    .map_err(command_exec_error)
}

fn status_from_exec(output: &CommandExecArtifact) -> ExternalAgentRunStatus {
    match output.status {
        CommandExecStatus::Succeeded => ExternalAgentRunStatus::Completed,
        CommandExecStatus::Failed => ExternalAgentRunStatus::Failed,
    }
}

fn external_event(event: &crate::command_exec::CommandExecEvent) -> ExternalAgentEvent {
    ExternalAgentEvent {
        kind: external_event_kind(event.kind),
        message: event.message.clone(),
        timestamp: event.timestamp_ms,
    }
}

fn external_event_kind(kind: CommandExecEventKind) -> ExternalAgentEventKind {
    match kind {
        CommandExecEventKind::Started => ExternalAgentEventKind::Command,
        CommandExecEventKind::Stdout => ExternalAgentEventKind::Stdout,
        CommandExecEventKind::Stderr => ExternalAgentEventKind::Stderr,
        CommandExecEventKind::Completed => ExternalAgentEventKind::Completed,
        CommandExecEventKind::Failed => ExternalAgentEventKind::Failed,
    }
}

fn command_exec_error(error: CommandExecError) -> ExternalAgentError {
    match error {
        CommandExecError::EmptyCommand => ExternalAgentError::EmptyCommand,
        CommandExecError::Io(message) => ExternalAgentError::Io(message),
        CommandExecError::Timeout => ExternalAgentError::Timeout,
    }
}
