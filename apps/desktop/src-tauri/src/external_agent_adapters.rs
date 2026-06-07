use crate::external_agent::{AdapterStatus, ExternalAgentAvailability, ExternalAgentKind};

pub(crate) fn default_adapters() -> Vec<ExternalAgentAvailability> {
    vec![
        adapter(
            "codex-cli",
            ExternalAgentKind::CodexCli,
            "Codex CLI",
            AdapterStatus::Missing,
            "Adapter placeholder; executable not detected.",
        ),
        adapter(
            "claude-code",
            ExternalAgentKind::ClaudeCode,
            "Claude Code",
            AdapterStatus::Missing,
            "Adapter placeholder; executable not detected.",
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
