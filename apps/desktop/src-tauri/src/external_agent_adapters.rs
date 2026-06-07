use crate::external_agent::{AdapterStatus, ExternalAgentAvailability, ExternalAgentKind};
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

const DEFAULT_WINDOWS_EXTENSIONS: &str = ".COM;.EXE;.BAT;.CMD";

pub(crate) fn default_adapters() -> Vec<ExternalAgentAvailability> {
    let path_var = env::var_os("PATH").unwrap_or_else(OsString::new);
    let path_ext = env::var("PATHEXT").unwrap_or_else(|_| DEFAULT_WINDOWS_EXTENSIONS.to_string());
    adapters_from_path(&path_var, &path_ext)
}

pub(crate) fn adapters_from_path(path_var: &OsStr, path_ext: &str) -> Vec<ExternalAgentAvailability> {
    let codex = detect_executable("codex", path_var, path_ext);
    let claude = detect_executable("claude", path_var, path_ext);
    vec![
        adapter(
            "codex-cli",
            ExternalAgentKind::CodexCli,
            "Codex CLI",
            codex.0,
            &codex.1,
        ),
        adapter(
            "claude-code",
            ExternalAgentKind::ClaudeCode,
            "Claude Code",
            claude.0,
            &claude.1,
        ),
        adapter(
            "generic-terminal",
            ExternalAgentKind::GenericTerminal,
            "Generic terminal agent",
            AdapterStatus::Available,
            "Approved terminal_command runs inside scoped isolation.",
        ),
    ]
}

fn detect_executable(command: &str, path_var: &OsStr, path_ext: &str) -> (AdapterStatus, String) {
    if path_var.to_string_lossy().trim().is_empty() {
        return (AdapterStatus::NotChecked, "PATH is empty; executable was not checked.".to_string());
    }
    match find_executable(command, path_var, path_ext) {
        Some(path) => (AdapterStatus::Available, format!("Executable found at {}.", path.display())),
        None => (AdapterStatus::Missing, format!("{command} executable was not found on PATH.")),
    }
}

fn find_executable(command: &str, path_var: &OsStr, path_ext: &str) -> Option<PathBuf> {
    let names = executable_names(command, path_ext);
    env::split_paths(path_var).find_map(|directory| {
        names.iter().map(|name| directory.join(name)).find(|candidate| candidate.is_file())
    })
}

fn executable_names(command: &str, path_ext: &str) -> Vec<String> {
    let mut names = vec![command.to_string()];
    names.extend(path_ext.split(';').filter_map(|extension| {
        let trimmed = extension.trim();
        if trimmed.is_empty() {
            None
        } else if trimmed.starts_with('.') {
            Some(format!("{command}{}", trimmed.to_ascii_lowercase()))
        } else {
            Some(format!("{command}.{}", trimmed.to_ascii_lowercase()))
        }
    }));
    names.sort();
    names.dedup();
    names
}

fn adapter(
    id: &str,
    kind: ExternalAgentKind,
    display_name: &str,
    status: AdapterStatus,
    detail: &str,
) -> ExternalAgentAvailability {
    ExternalAgentAvailability {
        adapter_id: id.to_string(),
        kind,
        display_name: display_name.to_string(),
        status,
        detail: detail.to_string(),
    }
}
