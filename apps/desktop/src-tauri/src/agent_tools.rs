//! Read-only tools the local model can call during a chat turn: read a file,
//! list a directory, or search the project. Every call is scoped to the project
//! root, capped in size, and never writes — write/command tools stay behind the
//! approval engine and are not exposed here.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const MAX_TOOL_RESULT_BYTES: usize = 8_000;
const MAX_LIST_ENTRIES: usize = 200;
const MAX_GREP_MATCHES: usize = 30;
const MAX_GREP_FILE_BYTES: u64 = 262_144;
const MAX_GREP_FILES: usize = 600;
const SKIP_DIRS: [&str; 6] = [".git", "node_modules", "target", "dist", ".tools", ".vite"];

/// A tool call the model asked for, parsed from its JSON reply.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum ToolCall {
    ReadFile {
        path: String,
        #[serde(default)]
        start_line: Option<usize>,
        #[serde(default)]
        end_line: Option<usize>,
    },
    ListDir {
        #[serde(default)]
        path: Option<String>,
    },
    Grep {
        query: String,
    },
}

impl ToolCall {
    pub fn summary(&self) -> String {
        match self {
            ToolCall::ReadFile { path, .. } => format!("read_file {path}"),
            ToolCall::ListDir { path } => {
                format!("list_dir {}", path.as_deref().unwrap_or("."))
            }
            ToolCall::Grep { query } => format!("grep \"{query}\""),
        }
    }
}

/// Parse a model reply as a tool call. Accepts a bare JSON object or one inside
/// a ```json fence; anything else is a normal answer (None).
pub fn parse_tool_call(reply: &str) -> Option<ToolCall> {
    let trimmed = reply.trim();
    let body = if let Some(rest) = trimmed.strip_prefix("```json") {
        rest.trim_start_matches(['\r', '\n'])
            .trim_end_matches('`')
            .trim()
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest.trim_start_matches(['\r', '\n'])
            .trim_end_matches('`')
            .trim()
    } else {
        trimmed
    };
    if !body.starts_with('{') {
        return None;
    }
    // Take only the first JSON object line block in case the model added prose.
    let candidate = body.lines().take(12).collect::<Vec<_>>().join("\n");
    serde_json::from_str::<ToolCall>(candidate.trim())
        .ok()
        .or_else(|| serde_json::from_str::<ToolCall>(body).ok())
}

/// True when a reply is shaped like a tool call (bare JSON object or fenced
/// block) even if it failed to parse. This is the trigger for a constrained
/// repair turn: without repair, malformed tool JSON would fall through and be
/// shown to the user as the "final answer".
pub fn looks_like_tool_attempt(reply: &str) -> bool {
    let trimmed = reply.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with("```")
}

/// JSON Schema mirroring `ToolCall`'s serde shape, applied as a sampler-level
/// decoding constraint (llguidance via mistral.rs) during repair turns so the
/// regenerated tool call is valid by construction — illegal tokens are masked
/// at sampling time. Keep in sync with `ToolCall`.
pub fn tool_call_json_schema() -> serde_json::Value {
    serde_json::json!({
        "anyOf": [
            {
                "type": "object",
                "properties": {
                    "tool": { "const": "read_file" },
                    "path": { "type": "string" },
                    "start_line": { "type": "integer", "minimum": 1 },
                    "end_line": { "type": "integer", "minimum": 1 }
                },
                "required": ["tool", "path"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "tool": { "const": "list_dir" },
                    "path": { "type": "string" }
                },
                "required": ["tool"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "tool": { "const": "grep" },
                    "query": { "type": "string" }
                },
                "required": ["tool", "query"],
                "additionalProperties": false
            }
        ]
    })
}

/// Execute a read-only tool inside `root`. Errors are returned as readable text
/// so the model can adjust (wrong path, etc.) instead of the loop dying.
pub fn execute_tool(root: &Path, call: &ToolCall) -> String {
    let result = match call {
        ToolCall::ReadFile {
            path,
            start_line,
            end_line,
        } => read_file(root, path, *start_line, *end_line),
        ToolCall::ListDir { path } => list_dir(root, path.as_deref().unwrap_or("")),
        ToolCall::Grep { query } => grep(root, query),
    };
    match result {
        Ok(text) => cap_text(&text),
        Err(error) => format!("Tool error: {error}"),
    }
}

fn read_file(
    root: &Path,
    path: &str,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> Result<String, String> {
    let resolved = scoped_path(root, path)?;
    let contents =
        fs::read_to_string(&resolved).map_err(|error| format!("cannot read {path}: {error}"))?;
    let lines: Vec<&str> = contents.lines().collect();
    let start = start_line.unwrap_or(1).max(1);
    let end = end_line.unwrap_or(lines.len()).min(lines.len());
    if start > end || lines.is_empty() {
        return Ok(format!(
            "{path} is empty or the range {start}-{end} is out of bounds (file has {} lines).",
            lines.len()
        ));
    }
    let body = lines[start - 1..end]
        .iter()
        .enumerate()
        .map(|(offset, line)| format!("{}: {line}", start + offset))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "{path} lines {start}-{end} of {}:\n{body}",
        lines.len()
    ))
}

fn list_dir(root: &Path, path: &str) -> Result<String, String> {
    let resolved = if path.is_empty() || path == "." {
        root.to_path_buf()
    } else {
        scoped_path(root, path)?
    };
    let mut entries = Vec::new();
    let read = fs::read_dir(&resolved).map_err(|error| format!("cannot list {path}: {error}"))?;
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        let marker = if entry.path().is_dir() { "/" } else { "" };
        entries.push(format!("{name}{marker}"));
        if entries.len() >= MAX_LIST_ENTRIES {
            entries.push("… (truncated)".to_string());
            break;
        }
    }
    entries.sort();
    Ok(format!(
        "Entries in {}:\n{}",
        if path.is_empty() { "." } else { path },
        entries.join("\n")
    ))
}

fn grep(root: &Path, query: &str) -> Result<String, String> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Err("grep needs a non-empty query".to_string());
    }
    let mut matches = Vec::new();
    let mut scanned = 0usize;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(path);
                }
                continue;
            }
            if scanned >= MAX_GREP_FILES || matches.len() >= MAX_GREP_MATCHES {
                break;
            }
            let Ok(meta) = entry.metadata() else { continue };
            if meta.len() > MAX_GREP_FILE_BYTES {
                continue;
            }
            scanned += 1;
            let Ok(contents) = fs::read_to_string(&path) else {
                continue;
            };
            for (index, line) in contents.lines().enumerate() {
                if line.to_lowercase().contains(&needle) {
                    let relative = path
                        .strip_prefix(root)
                        .unwrap_or(&path)
                        .display()
                        .to_string();
                    matches.push(format!("{relative}:{}: {}", index + 1, line.trim()));
                    if matches.len() >= MAX_GREP_MATCHES {
                        break;
                    }
                }
            }
        }
    }
    if matches.is_empty() {
        return Ok(format!(
            "No matches for \"{query}\" in {scanned} scanned file(s)."
        ));
    }
    Ok(format!("Matches for \"{query}\":\n{}", matches.join("\n")))
}

/// Resolve a model-supplied relative path inside the project root, refusing
/// traversal and absolute paths.
fn scoped_path(root: &Path, path: &str) -> Result<PathBuf, String> {
    let normalized = path.replace('\\', "/");
    let relative = Path::new(&normalized);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!("path must be relative to the project root: {path}"));
    }
    let resolved = root.join(relative);
    if !resolved.exists() {
        return Err(format!("path does not exist: {path}"));
    }
    Ok(resolved)
}

fn cap_text(text: &str) -> String {
    if text.len() <= MAX_TOOL_RESULT_BYTES {
        return text.to_string();
    }
    let mut end = MAX_TOOL_RESULT_BYTES;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n… (tool result truncated)", &text[..end])
}

/// The system-prompt contract that teaches any GGUF model the tool protocol.
pub fn tool_protocol_prompt() -> &'static str {
    "You can inspect the project with read-only tools before answering. To call a tool, reply with ONLY one JSON object and nothing else:\n\
{\"tool\":\"read_file\",\"path\":\"src/main.rs\",\"start_line\":1,\"end_line\":120}\n\
{\"tool\":\"list_dir\",\"path\":\"src\"}\n\
{\"tool\":\"grep\",\"query\":\"fn main\"}\n\
You will receive the tool result, then you may call another tool (a few calls maximum). When you have enough context, reply normally with your final answer — never wrap it in JSON."
}
