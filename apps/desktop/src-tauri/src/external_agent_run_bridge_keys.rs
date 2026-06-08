use crate::external_agent::{ExternalAgentError, ExternalAgentEventKind, ExternalAgentRunStatus};
use crate::external_agent_command_contracts::ExternalAgentPermissionMode;

pub(crate) fn permission_mode(value: &str) -> Result<ExternalAgentPermissionMode, String> {
    match value {
        "read_only" => Ok(ExternalAgentPermissionMode::ReadOnly),
        "workspace_write" => Ok(ExternalAgentPermissionMode::WorkspaceWrite),
        _ => Err("Unsupported Codex permission mode.".to_string()),
    }
}

pub(crate) fn status_key(status: ExternalAgentRunStatus) -> &'static str {
    match status {
        ExternalAgentRunStatus::Accepted => "accepted",
        ExternalAgentRunStatus::Blocked => "blocked",
        ExternalAgentRunStatus::Completed => "completed",
        ExternalAgentRunStatus::Failed => "failed",
        ExternalAgentRunStatus::Reverted => "reverted",
    }
}

pub(crate) fn event_kind_key(kind: ExternalAgentEventKind) -> &'static str {
    match kind {
        ExternalAgentEventKind::CheckpointCreated => "checkpoint_created",
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

pub(crate) fn external_agent_error(error: ExternalAgentError) -> String {
    match error {
        ExternalAgentError::AdapterUnavailable => "Codex CLI adapter is unavailable.".to_string(),
        ExternalAgentError::EmptyTask => "Codex launch requires a non-empty task.".to_string(),
        ExternalAgentError::Io(message) => {
            format!("Codex worker command failed to start: {message}")
        }
        ExternalAgentError::MissingIsolation => {
            "Codex launch requires a checkpoint or isolated worktree before execution.".to_string()
        }
        ExternalAgentError::MissingTerminalApproval => {
            "Codex launch requires a terminal-command approval before execution.".to_string()
        }
        error => format!("{error:?}"),
    }
}
