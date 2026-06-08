#[cfg(test)]
mod tests {
    use crate::external_agent::{AdapterStatus, ExternalAgentAvailability, ExternalAgentKind};
    use crate::external_agent_status_bridge::external_agent_status_from_adapters;

    #[test]
    fn external_agent_status_maps_detected_adapters_for_ui() {
        let status = external_agent_status_from_adapters(vec![
            adapter(
                "codex-cli",
                ExternalAgentKind::CodexCli,
                "Codex CLI",
                AdapterStatus::Available,
            ),
            adapter(
                "claude-code",
                ExternalAgentKind::ClaudeCode,
                "Claude Code",
                AdapterStatus::Missing,
            ),
            adapter(
                "generic-terminal",
                ExternalAgentKind::GenericTerminal,
                "Generic terminal agent",
                AdapterStatus::Available,
            ),
        ]);

        assert_eq!(status.adapters.len(), 3);
        assert_eq!(status.adapters[0].id, "codex-cli");
        assert_eq!(status.adapters[0].kind, "codex_cli");
        assert_eq!(status.adapters[0].status, "available");
        assert_eq!(status.adapters[1].status, "missing");
        assert!(status.adapters[1]
            .detail
            .contains("Approval-gated read and write launch"));
        assert_eq!(status.adapters[2].kind, "generic_terminal");
    }

    fn adapter(
        id: &str,
        kind: ExternalAgentKind,
        label: &str,
        status: AdapterStatus,
    ) -> ExternalAgentAvailability {
        ExternalAgentAvailability {
            adapter_id: id.to_string(),
            detail: "detected in test".to_string(),
            display_name: label.to_string(),
            kind,
            status,
        }
    }
}
