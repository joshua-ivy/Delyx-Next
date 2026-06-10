//! Read-only QA/QC review of generated output by the Claude Code / Codex CLIs.
//!
//! Use case: a small local model (e.g. a ~30B Ollama coder) produces code; a
//! stronger CLI agent reviews it read-only before it reaches the user. CLI-first
//! (subscription cost), read-only, and reuses the `cli_chat` execution path.

use crate::cli_chat::cli_chat_text;
use crate::command_exec::{run_command_exec, CommandExecError, CommandExecRequest};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Cap the reviewed content so the prompt fits in a command-line argument.
const MAX_CONTENT_CHARS: usize = 24_000;

// QA/QC reviews a small local model's output, so a fast, cheap reviewer model is
// the economical default per CLI. Callers can override per request.
const DEFAULT_CLAUDE_REVIEW_MODEL: &str = "haiku";
const DEFAULT_CODEX_REVIEW_MODEL: &str = "gpt-5.4-mini";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliReviewRequest {
    pub adapter_id: String,
    pub task: String,
    pub content: String,
    pub working_directory: String,
    pub timeout_ms: u64,
    pub started_at_ms: u64,
    /// Optional reviewer model override (e.g. "haiku", "sonnet" for Claude).
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliReviewResult {
    pub adapter_id: String,
    /// "pass", "fail", or "unclear".
    pub verdict: String,
    /// The verdict line plus findings, with any corrected-code block removed.
    pub text: String,
    /// The reviewer's corrected, complete code (no fences) when it found fixable
    /// issues; `None` on a clean pass or when no corrected code was returned.
    pub fix: Option<String>,
}

#[tauri::command]
pub async fn cli_review(request: CliReviewRequest) -> Result<CliReviewResult, String> {
    // The review spawns a CLI subprocess and blocks until it exits. Run it on a
    // blocking thread so the Tauri main thread (and the webview) stays responsive
    // — a sync command would freeze the UI for the whole review.
    tauri::async_runtime::spawn_blocking(move || run_cli_review(request))
        .await
        .map_err(|error| format!("CLI review task failed: {error}"))?
}

pub fn run_cli_review(request: CliReviewRequest) -> Result<CliReviewResult, String> {
    if request.working_directory.trim().is_empty() {
        return Err("CLI review requires a working directory.".to_string());
    }
    let prompt = build_review_prompt(&request.task, &request.content);
    let model = request
        .model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let (program, args) = cli_review_command(&request.adapter_id, model, &prompt)?;
    let artifact = run_command_exec(CommandExecRequest {
        approval_id: format!("cli-review-{}", request.adapter_id),
        args,
        prepare_terminal: true,
        program,
        run_id: format!("cli-review-{}", request.adapter_id),
        started_at_ms: request.started_at_ms,
        timeout_ms: request.timeout_ms,
        working_directory: PathBuf::from(&request.working_directory),
    })
    .map_err(cli_review_error)?;
    let raw = cli_chat_text(&artifact)?;
    let verdict = parse_review_verdict(&raw).to_string();
    let (text, fix) = split_review_text(&raw);
    Ok(CliReviewResult {
        verdict,
        adapter_id: request.adapter_id,
        text,
        fix,
    })
}

/// Separate the reviewer's findings from the corrected code it appends on a
/// failure. The fix is the last fenced ``` block (returned without fences); the
/// findings are everything before it. Falls back to (whole text, None).
pub(crate) fn split_review_text(raw: &str) -> (String, Option<String>) {
    let lines: Vec<&str> = raw.lines().collect();
    // Find the last fenced block by scanning fence lines from the end.
    let fences: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("```"))
        .map(|(index, _)| index)
        .collect();
    if fences.len() < 2 {
        return (raw.trim().to_string(), None);
    }
    let close = fences[fences.len() - 1];
    let open = fences[fences.len() - 2];
    let code = lines[open + 1..close].join("\n");
    if code.trim().is_empty() {
        return (raw.trim().to_string(), None);
    }
    let findings = lines[..open].join("\n").trim().to_string();
    (findings, Some(code.trim_end().to_string()))
}

/// Build a cheap, read-only one-shot reviewer invocation. Unlike the CLI chat
/// path (where the CLI is the coding agent and wants tools), QA/QC only judges a
/// self-contained snippet, so we strip everything that costs tokens: a cheaper
/// model, `--safe-mode` (no CLAUDE.md / MCP / hooks, but OAuth subscription auth
/// is preserved — `--bare` would force an API key), and no tools so it cannot
/// agentically read the repo.
pub(crate) fn cli_review_command(
    adapter_id: &str,
    model: Option<&str>,
    prompt: &str,
) -> Result<(String, Vec<String>), String> {
    match adapter_id {
        "claude-code" => {
            let model = model.unwrap_or(DEFAULT_CLAUDE_REVIEW_MODEL);
            Ok((
                "claude".to_string(),
                vec![
                    "-p".to_string(),
                    prompt.to_string(),
                    "--model".to_string(),
                    model.to_string(),
                    "--safe-mode".to_string(),
                    // Variadic flag kept last with an empty value so it disables
                    // all tools without swallowing the prompt argument.
                    "--tools".to_string(),
                    String::new(),
                ],
            ))
        }
        "codex-cli" => {
            let model = model.unwrap_or(DEFAULT_CODEX_REVIEW_MODEL);
            Ok((
                "codex".to_string(),
                vec![
                    "exec".to_string(),
                    "--sandbox".to_string(),
                    "read-only".to_string(),
                    "-m".to_string(),
                    model.to_string(),
                    prompt.to_string(),
                ],
            ))
        }
        other => Err(format!(
            "CLI review is not supported for adapter `{other}`."
        )),
    }
}

fn cli_review_error(error: CommandExecError) -> String {
    match error {
        CommandExecError::EmptyCommand => "CLI review command was empty.".to_string(),
        CommandExecError::Io(message) => format!("CLI review could not start: {message}"),
        CommandExecError::Timeout => "CLI review timed out.".to_string(),
    }
}

pub(crate) fn build_review_prompt(task: &str, content: &str) -> String {
    let task = if task.trim().is_empty() {
        "(no explicit task was given)"
    } else {
        task.trim()
    };
    let content = capped(content.trim());
    format!(
        "You are a strict senior code reviewer doing QA/QC on output from a smaller \
local model. The output was meant to accomplish this task:\n\n{task}\n\nOutput to \
review:\n\n{content}\n\nReview it for correctness bugs, missing edge cases, security \
issues, and whether it actually accomplishes the task. Your first line must be \
exactly \"VERDICT: PASS\" or \"VERDICT: FAIL\". Then list at most 5 concise findings \
as short bullets — no preamble, no restating the code. If VERDICT: FAIL, after the \
findings output the corrected, complete, runnable code as the LAST thing in your \
reply, in a single fenced code block (and nothing after it). If VERDICT: PASS, do \
not include any code block. If there is no code to review, reply \"VERDICT: PASS\" \
and nothing else."
    )
}

pub(crate) fn parse_review_verdict(text: &str) -> &'static str {
    let upper = text.to_uppercase();
    if upper.contains("VERDICT: FAIL") || upper.contains("VERDICT:FAIL") {
        return "fail";
    }
    if upper.contains("VERDICT: PASS") || upper.contains("VERDICT:PASS") {
        return "pass";
    }
    // Fall back to a lone keyword, but never silently pass on an ambiguous reply.
    match (upper.contains("FAIL"), upper.contains("PASS")) {
        (true, false) => "fail",
        (false, true) => "pass",
        _ => "unclear",
    }
}

fn capped(content: &str) -> String {
    if content.chars().count() <= MAX_CONTENT_CHARS {
        return content.to_string();
    }
    let truncated: String = content.chars().take(MAX_CONTENT_CHARS).collect();
    format!("{truncated}\n...[truncated for review]")
}
