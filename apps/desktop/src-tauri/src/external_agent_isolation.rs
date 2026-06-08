use crate::external_agent::{
    ExternalAgentError, ExternalAgentEvent, ExternalAgentEventKind, ExternalAgentScope,
};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const MAX_CHECKPOINT_FILE_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentCheckpoint {
    pub id: String,
    pub receipts: Vec<String>,
}

pub fn ensure_external_agent_isolation(
    scope: &mut ExternalAgentScope,
    changed_files: &[PathBuf],
    required: bool,
    id: String,
) -> Result<Option<ExternalAgentCheckpoint>, ExternalAgentError> {
    if !required || scope.checkpoint_id.is_some() || scope.worktree_id.is_some() {
        return Ok(None);
    }
    let checkpoint = create_external_agent_checkpoint(&scope.project_root, changed_files, id)?;
    scope.checkpoint_id = Some(checkpoint.id.clone());
    Ok(Some(checkpoint))
}

pub fn checkpoint_events(
    checkpoint: &ExternalAgentCheckpoint,
    timestamp: u64,
) -> Vec<ExternalAgentEvent> {
    std::iter::once(event(
        &format!("External-agent checkpoint created: {}", checkpoint.id),
        timestamp,
    ))
    .chain(
        checkpoint
            .receipts
            .iter()
            .map(|receipt| event(receipt, timestamp)),
    )
    .collect()
}

fn create_external_agent_checkpoint(
    project_root: &Path,
    changed_files: &[PathBuf],
    id: String,
) -> Result<ExternalAgentCheckpoint, ExternalAgentError> {
    if changed_files.is_empty() {
        return Err(ExternalAgentError::MissingIsolation);
    }
    let root = checkpoint_root(&id)?;
    let files_root = root.join("files");
    fs::create_dir_all(&files_root).map_err(io_error)?;
    let mut receipts = Vec::new();
    for path in changed_files {
        let relative = path
            .strip_prefix(project_root)
            .map_err(|_| ExternalAgentError::OutsideApprovedRoot)?;
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => {
                if metadata.len() > MAX_CHECKPOINT_FILE_BYTES {
                    return Err(ExternalAgentError::Io(format!(
                        "External-agent checkpoint file is too large: {}",
                        path.display()
                    )));
                }
                let destination = files_root.join(relative);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).map_err(io_error)?;
                }
                fs::copy(path, &destination).map_err(io_error)?;
                receipts.push(format!(
                    "checkpointed: {} ({} bytes)",
                    path.display(),
                    metadata.len()
                ));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                receipts.push(format!("missing before run: {}", path.display()));
            }
            Err(error) => return Err(io_error(error)),
            _ => {
                return Err(ExternalAgentError::Io(format!(
                    "External-agent checkpoint path is not a file: {}",
                    path.display()
                )));
            }
        }
    }
    fs::write(root.join("manifest.txt"), receipts.join("\n")).map_err(io_error)?;
    Ok(ExternalAgentCheckpoint { id, receipts })
}

fn checkpoint_root(id: &str) -> Result<PathBuf, ExternalAgentError> {
    let root = std::env::temp_dir()
        .join("delyx-next-external-agent-checkpoints")
        .join(id);
    if root.exists() {
        fs::remove_dir_all(&root).map_err(io_error)?;
    }
    Ok(root)
}

fn io_error(error: std::io::Error) -> ExternalAgentError {
    ExternalAgentError::Io(error.to_string())
}

fn event(message: &str, timestamp: u64) -> ExternalAgentEvent {
    ExternalAgentEvent {
        kind: ExternalAgentEventKind::CheckpointCreated,
        message: message.to_string(),
        timestamp,
    }
}
