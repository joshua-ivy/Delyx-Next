use crate::external_agent_types::{
    ExternalAgentEvent, ExternalAgentEventKind, ExternalAgentRunStatus,
};
use serde_json::Value;

/// A single tool invocation parsed from a Claude Code `stream-json` transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeToolUse {
    pub name: String,
    pub file_path: Option<String>,
}

/// Structured summary of a Claude Code headless `stream-json` stdout stream.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClaudeStreamSummary {
    pub assistant_texts: Vec<String>,
    pub tool_uses: Vec<ClaudeToolUse>,
    pub edited_files: Vec<String>,
    pub result_text: Option<String>,
    pub is_error: bool,
    pub saw_result: bool,
}

/// Parse Claude Code `--output-format stream-json` stdout.
///
/// Each non-empty line is one JSON object. Unparseable lines are ignored so a
/// noisy or partially captured stream still yields the structured events Delyx
/// can verify. `Edit`, `Write`, and `MultiEdit` tool uses contribute edited
/// file paths (deduplicated, order preserved).
pub fn parse_claude_stream_json(stdout: &str) -> ClaudeStreamSummary {
    let mut summary = ClaudeStreamSummary::default();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match value.get("type").and_then(Value::as_str) {
            Some("assistant") => collect_message_content(&value, &mut summary),
            Some("result") => collect_result(&value, &mut summary),
            _ => {}
        }
    }
    summary
}

fn collect_message_content(value: &Value, summary: &mut ClaudeStreamSummary) {
    let Some(content) = value
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_array)
    else {
        return;
    };
    for block in content {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    let text = text.trim();
                    if !text.is_empty() {
                        summary.assistant_texts.push(text.to_string());
                    }
                }
            }
            Some("tool_use") => collect_tool_use(block, summary),
            _ => {}
        }
    }
}

fn collect_tool_use(block: &Value, summary: &mut ClaudeStreamSummary) {
    let Some(name) = block.get("name").and_then(Value::as_str) else {
        return;
    };
    let file_path = block
        .get("input")
        .and_then(|input| input.get("file_path"))
        .and_then(Value::as_str)
        .map(str::to_string);
    if is_edit_tool(name) {
        if let Some(path) = &file_path {
            push_unique(&mut summary.edited_files, path);
        }
    }
    summary.tool_uses.push(ClaudeToolUse {
        name: name.to_string(),
        file_path,
    });
}

fn collect_result(value: &Value, summary: &mut ClaudeStreamSummary) {
    summary.saw_result = true;
    if value.get("is_error").and_then(Value::as_bool) == Some(true) {
        summary.is_error = true;
    }
    if let Some(text) = value.get("result").and_then(Value::as_str) {
        let text = text.trim();
        if !text.is_empty() {
            summary.result_text = Some(text.to_string());
        }
    }
}

/// Parse Claude `stream-json` worker stdout, append the structured transcript
/// events to `transcript`, and return the effective run status: a parsed
/// `result.is_error` marks the run failed even when the process exits 0.
pub(crate) fn apply_stream_summary(
    transcript: &mut Vec<ExternalAgentEvent>,
    stdout: &str,
    worker_status: ExternalAgentRunStatus,
    now: u64,
) -> ExternalAgentRunStatus {
    let summary = parse_claude_stream_json(stdout);
    transcript.extend(stream_summary_events(&summary, now));
    if summary.is_error {
        ExternalAgentRunStatus::Failed
    } else {
        worker_status
    }
}

/// Convert a parsed Claude `stream-json` summary into transcript events so the UI
/// shows assistant turns, tool uses, and edited files instead of raw stdout.
fn stream_summary_events(summary: &ClaudeStreamSummary, now: u64) -> Vec<ExternalAgentEvent> {
    let mut events = Vec::new();
    for text in &summary.assistant_texts {
        events.push(event(ExternalAgentEventKind::Stdout, text, now));
    }
    for tool in &summary.tool_uses {
        let label = match &tool.file_path {
            Some(path) => format!("tool_use {} {path}", tool.name),
            None => format!("tool_use {}", tool.name),
        };
        events.push(event(ExternalAgentEventKind::Command, &label, now));
    }
    for path in &summary.edited_files {
        events.push(event(ExternalAgentEventKind::FileChanged, path, now));
    }
    if let Some(result) = &summary.result_text {
        events.push(event(
            ExternalAgentEventKind::Stdout,
            &format!("result: {result}"),
            now,
        ));
    }
    if summary.is_error {
        events.push(event(
            ExternalAgentEventKind::Stderr,
            "Claude reported result.is_error = true.",
            now,
        ));
    }
    events
}

fn event(kind: ExternalAgentEventKind, message: &str, timestamp: u64) -> ExternalAgentEvent {
    ExternalAgentEvent {
        kind,
        message: message.to_string(),
        timestamp,
    }
}

fn is_edit_tool(name: &str) -> bool {
    matches!(name, "Edit" | "Write" | "MultiEdit")
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}
