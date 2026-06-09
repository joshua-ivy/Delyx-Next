//! Read-only QA/QC review of generated output by the Claude Code / Codex CLIs.
//!
//! Use case: a small local model (e.g. a ~30B Ollama coder) produces code; a
//! stronger CLI agent reviews it read-only before it reaches the user. CLI-first
//! (subscription cost), read-only, and reuses the `cli_chat` execution path.

use crate::cli_chat::{run_cli_chat, CliChatRequest};
use serde::{Deserialize, Serialize};

// Cap the reviewed content so the prompt fits in a command-line argument.
const MAX_CONTENT_CHARS: usize = 24_000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliReviewRequest {
    pub adapter_id: String,
    pub task: String,
    pub content: String,
    pub working_directory: String,
    pub timeout_ms: u64,
    pub started_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliReviewResult {
    pub adapter_id: String,
    /// "pass", "fail", or "unclear".
    pub verdict: String,
    pub text: String,
}

#[tauri::command]
pub fn cli_review(request: CliReviewRequest) -> Result<CliReviewResult, String> {
    run_cli_review(request)
}

pub fn run_cli_review(request: CliReviewRequest) -> Result<CliReviewResult, String> {
    let chat = run_cli_chat(CliChatRequest {
        adapter_id: request.adapter_id.clone(),
        prompt: build_review_prompt(&request.task, &request.content),
        started_at_ms: request.started_at_ms,
        timeout_ms: request.timeout_ms,
        working_directory: request.working_directory,
    })?;
    Ok(CliReviewResult {
        adapter_id: request.adapter_id,
        verdict: parse_review_verdict(&chat.text).to_string(),
        text: chat.text,
    })
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
exactly \"VERDICT: PASS\" or \"VERDICT: FAIL\". Then list concise findings. If there \
is no code to review, reply \"VERDICT: PASS\" and say no code was present."
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
