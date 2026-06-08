use serde::Serialize;

/// Read-only report of the isolation Delyx can actually offer external-agent runs
/// on the current platform. Adapted from Codex sandbox capability detection as a
/// design reference only: it imports no OS-specific sandbox backend and never
/// claims isolation Delyx does not implement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxCapabilityView {
    pub platform: String,
    pub modes: Vec<SandboxModeView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxModeView {
    pub id: String,
    pub available: bool,
    pub detail: String,
}

#[tauri::command]
pub fn sandbox_capability() -> SandboxCapabilityView {
    sandbox_capability_from(current_platform(), git_executable_available())
}

pub(crate) fn sandbox_capability_from(
    platform: &str,
    git_available: bool,
) -> SandboxCapabilityView {
    SandboxCapabilityView {
        platform: platform.to_string(),
        modes: vec![
            mode(
                "checkpoint",
                true,
                "Delyx temp-backed file checkpoints isolate external-agent writes and enable rollback.",
            ),
            mode("git_worktree", git_available, git_worktree_detail(git_available)),
            mode(
                "os_process_sandbox",
                false,
                "No OS-level process sandbox backend is wired; Delyx relies on approval gates plus checkpoint isolation.",
            ),
        ],
    }
}

fn git_worktree_detail(available: bool) -> &'static str {
    if available {
        "git is on PATH, so isolated worktrees are possible; Delyx does not create worktrees for external agents yet."
    } else {
        "git was not found on PATH, so worktree isolation is unavailable."
    }
}

fn mode(id: &str, available: bool, detail: &str) -> SandboxModeView {
    SandboxModeView {
        available,
        detail: detail.to_string(),
        id: id.to_string(),
    }
}

fn current_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

fn git_executable_available() -> bool {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let path_ext = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
    crate::external_agent_adapters::find_executable("git", &path_var, &path_ext).is_some()
}
