//! Read-only chat answers from the Claude Code / Codex CLIs.
//!
//! CLI-first by design: these run on the user's flat-rate subscription, which is
//! cheaper than per-token cloud APIs. The call is read-only (`claude -p`,
//! `codex exec --sandbox read-only`) and captured as a command artifact. Consent
//! is at model-selection time in the UI (the user explicitly picks the CLI as
//! their chat model), so individual answers are not separately approval-gated,
//! matching how selecting a local Ollama model works.

use crate::command_exec::{
    run_command_exec, CommandExecArtifact, CommandExecError, CommandExecRequest, CommandExecStatus,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliChatRequest {
    pub adapter_id: String,
    pub prompt: String,
    pub working_directory: String,
    pub timeout_ms: u64,
    pub started_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliChatResult {
    pub adapter_id: String,
    pub text: String,
}

#[tauri::command]
pub fn cli_chat(request: CliChatRequest) -> Result<CliChatResult, String> {
    run_cli_chat(request)
}

pub fn run_cli_chat(request: CliChatRequest) -> Result<CliChatResult, String> {
    let prompt = request.prompt.trim();
    if prompt.is_empty() {
        return Err("CLI chat requires a non-empty prompt.".to_string());
    }
    if request.working_directory.trim().is_empty() {
        return Err("CLI chat requires a working directory.".to_string());
    }
    let (program, args) = cli_chat_command(&request.adapter_id, prompt)?;
    let artifact = run_command_exec(CommandExecRequest {
        approval_id: format!("cli-chat-{}", request.adapter_id),
        args,
        prepare_terminal: true,
        program,
        run_id: format!("cli-chat-{}", request.adapter_id),
        started_at_ms: request.started_at_ms,
        timeout_ms: request.timeout_ms,
        working_directory: PathBuf::from(&request.working_directory),
    })
    .map_err(cli_chat_error)?;
    Ok(CliChatResult {
        adapter_id: request.adapter_id,
        text: cli_chat_text(&artifact)?,
    })
}

pub(crate) fn cli_chat_command(
    adapter_id: &str,
    prompt: &str,
) -> Result<(String, Vec<String>), String> {
    match adapter_id {
        "claude-code" => Ok((
            "claude".to_string(),
            vec!["-p".to_string(), prompt.to_string()],
        )),
        "codex-cli" => Ok((
            "codex".to_string(),
            vec![
                "exec".to_string(),
                "--sandbox".to_string(),
                "read-only".to_string(),
                prompt.to_string(),
            ],
        )),
        other => Err(format!("CLI chat is not supported for adapter `{other}`.")),
    }
}

pub(crate) fn cli_chat_text(artifact: &CommandExecArtifact) -> Result<String, String> {
    if artifact.status == CommandExecStatus::Failed {
        let detail = if artifact.stderr.trim().is_empty() {
            artifact.stdout.trim()
        } else {
            artifact.stderr.trim()
        };
        return Err(format!(
            "CLI chat command failed (exit {:?}): {detail}",
            artifact.exit_code
        ));
    }
    let text = artifact.stdout.trim();
    if text.is_empty() {
        return Err("CLI chat returned no output.".to_string());
    }
    Ok(text.to_string())
}

fn cli_chat_error(error: CommandExecError) -> String {
    match error {
        CommandExecError::EmptyCommand => "CLI chat command was empty.".to_string(),
        CommandExecError::Io(message) => format!("CLI chat could not start: {message}"),
        CommandExecError::Timeout => "CLI chat timed out.".to_string(),
    }
}
