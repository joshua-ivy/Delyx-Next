use crate::external_agent::{ExternalAgentError, ExternalAgentScope};
use std::fs;
use std::path::{Path, PathBuf};

pub fn checked_approved_path(path: &Path, approved_roots: &[PathBuf]) -> Result<PathBuf, ExternalAgentError> {
    let normalized = fs::canonicalize(path).map_err(io_error)?;
    approved_roots
        .iter()
        .any(|root| normalized.starts_with(root))
        .then_some(normalized)
        .ok_or(ExternalAgentError::OutsideApprovedRoot)
}

pub fn checked_scoped_path(path: &Path, scope: &ExternalAgentScope) -> Result<PathBuf, ExternalAgentError> {
    let normalized = fs::canonicalize(path).map_err(io_error)?;
    let inside_root = normalized.starts_with(&scope.project_root);
    let inside_allowed = scope.allowed_paths.iter().any(|allowed| normalized.starts_with(allowed));
    (inside_root && inside_allowed).then_some(normalized).ok_or(ExternalAgentError::OutsideApprovedRoot)
}

fn io_error(error: std::io::Error) -> ExternalAgentError {
    ExternalAgentError::Io(error.to_string())
}
