use crate::agent_executor_bridge::AgentPatchProposalExecuteRequest;
use crate::agent_patch_draft_bridge::AgentPatchDraftExecuteRequest;
use crate::model_ollama::OllamaChatMessage;
use crate::patch_bridge::PatchFileRequest;
use crate::workspace_bridge::{workspace_read_files_from_path, WorkspaceFileReadView};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

const DEFAULT_MAX_BYTES: usize = 20_000;
const MAX_DRAFT_FILES: usize = 4;

#[derive(Debug, Deserialize)]
struct PatchDraftPayload {
    files: Vec<PatchDraftFile>,
}

#[derive(Debug, Deserialize)]
struct PatchDraftFile {
    path: String,
    after: String,
}

pub(crate) fn validate_request(request: &AgentPatchDraftExecuteRequest) -> Result<(), String> {
    if request.client_id.trim().is_empty()
        || request.run_id.trim().is_empty()
        || request.approval_id.trim().is_empty()
        || request.model.trim().is_empty()
        || request.project_path.trim().is_empty()
        || request.created_at_ms == 0
    {
        return Err(
            "PatchDraft requires client, run, approval, project, model, and timestamp.".to_string(),
        );
    }
    if request.approved_roots.is_empty() {
        return Err("PatchDraft requires at least one approved root.".to_string());
    }
    Ok(())
}

pub(crate) fn read_draft_files(
    request: &AgentPatchDraftExecuteRequest,
) -> Result<Vec<WorkspaceFileReadView>, String> {
    let paths = scoped_plan_files(request)?;
    workspace_read_files_from_path(
        Path::new(&request.project_path),
        &paths,
        request.max_bytes_per_file.unwrap_or(DEFAULT_MAX_BYTES),
    )
}

pub(crate) fn draft_messages(
    request: &AgentPatchDraftExecuteRequest,
    files: &[WorkspaceFileReadView],
) -> Vec<OllamaChatMessage> {
    vec![
        message(
            "system",
            "You are Delyx Next PatchDraftAgent. Return only JSON. Generate complete replacement file contents for the provided files only. Do not claim commands, tests, or file writes ran.",
        ),
        message(
            "user",
            format!(
                "Goal:\n{}\n\nPlan steps:\n{}\n\nFiles:\n{}\n\nReturn JSON exactly like {{\"files\":[{{\"path\":\"relative/path\",\"after\":\"complete file contents\"}}]}}.",
                request.goal.trim(),
                draft_steps(request),
                file_blocks(files),
            ),
        ),
    ]
}

pub(crate) fn patch_request_from_draft_text(
    request: &AgentPatchDraftExecuteRequest,
    files: &[WorkspaceFileReadView],
    text: &str,
) -> Result<AgentPatchProposalExecuteRequest, String> {
    if files.iter().any(|file| file.truncated) {
        return Err("PatchDraft refused truncated file input.".to_string());
    }
    let payload = extract_json_payload(text)?;
    let draft = serde_json::from_str::<PatchDraftPayload>(&payload)
        .map_err(|error| format!("PatchDraft JSON was not parseable: {error}."))?;
    if draft.files.is_empty() || draft.files.len() > MAX_DRAFT_FILES {
        return Err("PatchDraft must return 1-4 files.".to_string());
    }

    let allowed = allowed_file_map(files);
    let mut seen = HashSet::new();
    let mut patch_files = Vec::new();
    for file in draft.files {
        let normalized = normalize_path(&file.path)?;
        if !seen.insert(normalized.clone()) {
            return Err(format!(
                "PatchDraft returned duplicate file `{normalized}`."
            ));
        }
        let source = allowed
            .get(&normalized)
            .ok_or_else(|| format!("PatchDraft returned unapproved file `{normalized}`."))?;
        if file.after.trim().is_empty() {
            return Err(format!(
                "PatchDraft returned empty contents for `{normalized}`."
            ));
        }
        if file.after == source.contents {
            return Err(format!(
                "PatchDraft returned unchanged contents for `{normalized}`."
            ));
        }
        patch_files.push(PatchFileRequest {
            after: file.after,
            path: absolute_workspace_path(&request.project_path, &source.path)?,
        });
    }

    Ok(AgentPatchProposalExecuteRequest {
        approval_id: request.approval_id.clone(),
        approved_roots: request.approved_roots.clone(),
        client_id: request.client_id.clone(),
        created_at_ms: request.created_at_ms,
        files: patch_files,
        run_id: request.run_id.clone(),
    })
}

fn scoped_plan_files(request: &AgentPatchDraftExecuteRequest) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for candidate in request
        .files_likely_involved
        .iter()
        .chain(request.scope_paths.iter())
    {
        let normalized = normalize_path(candidate)?;
        if seen.insert(normalized.clone()) {
            paths.push(normalized);
        }
        if paths.len() == MAX_DRAFT_FILES {
            break;
        }
    }
    if paths.is_empty() {
        return Err("PatchDraft requires approved plan files to read.".to_string());
    }
    Ok(paths)
}

fn draft_steps(request: &AgentPatchDraftExecuteRequest) -> String {
    request
        .plan_steps
        .iter()
        .map(|step| format!("- {}", step.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn file_blocks(files: &[WorkspaceFileReadView]) -> String {
    files
        .iter()
        .map(|file| format!("--- {}\n{}", file.path, file.contents))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn allowed_file_map(files: &[WorkspaceFileReadView]) -> HashMap<String, WorkspaceFileReadView> {
    files
        .iter()
        .filter_map(|file| {
            normalize_path(&file.path)
                .ok()
                .map(|path| (path, file.clone()))
        })
        .collect()
}

fn extract_json_payload(text: &str) -> Result<String, String> {
    let start = text
        .find('{')
        .ok_or_else(|| "PatchDraft response did not contain JSON.".to_string())?;
    let end = text
        .rfind('}')
        .ok_or_else(|| "PatchDraft response did not contain a complete JSON object.".to_string())?;
    if end < start {
        return Err("PatchDraft response JSON was malformed.".to_string());
    }
    Ok(text[start..=end].to_string())
}

fn absolute_workspace_path(project_path: &str, relative_path: &str) -> Result<String, String> {
    let relative = normalize_path_buf(relative_path)?;
    Ok(Path::new(project_path)
        .join(relative)
        .to_string_lossy()
        .replace('\\', "/"))
}

fn message(role: &str, content: impl Into<String>) -> OllamaChatMessage {
    OllamaChatMessage {
        content: content.into(),
        role: role.to_string(),
    }
}

fn normalize_path(path: &str) -> Result<String, String> {
    Ok(normalize_path_buf(path)?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn normalize_path_buf(path: &str) -> Result<PathBuf, String> {
    let candidate = Path::new(path.trim());
    if candidate.is_absolute() {
        return Err("PatchDraft file paths must be relative project paths.".to_string());
    }
    let mut normalized = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            _ => return Err("PatchDraft file paths must stay inside the project.".to_string()),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("PatchDraft file path is empty.".to_string());
    }
    Ok(normalized)
}
