#[cfg(test)]
mod tests {
    use crate::external_agent_contract_bridge::{
        preview_external_agent_contract, ExternalAgentContractPreviewRequest,
    };

    #[test]
    fn preview_contract_returns_codex_view_without_launching() {
        let view = preview_external_agent_contract(request(
            "codex_cli",
            "run-1",
            "Review the active thread.",
            "read_only",
        ))
        .unwrap();

        assert_eq!(view.id, "contract-run-1-codex-cli");
        assert_eq!(view.adapter_id, "codex-cli");
        assert_eq!(view.status, "draft");
        assert_eq!(view.permission_mode, "read_only");
        assert_eq!(view.program, "codex");
        assert!(view.args.contains(&"exec".to_string()));
        assert_eq!(view.working_directory, "C:/repo");
        assert_eq!(view.transcript_format, "jsonl");
        assert!(view
            .required_delyx_tools
            .contains(&"external_agent".to_string()));
        assert!(view.safety_summary.contains("approvals"));
    }

    #[test]
    fn preview_contract_returns_claude_workspace_write_view() {
        let view = preview_external_agent_contract(request(
            "claude_code",
            "run-2",
            "Inspect and edit approved files.",
            "workspace_write",
        ))
        .unwrap();

        assert_eq!(view.id, "contract-run-2-claude-code");
        assert_eq!(view.program, "claude");
        assert_eq!(view.permission_mode, "workspace_write");
        assert!(view.args.contains(&"acceptEdits".to_string()));
        assert_eq!(view.transcript_format, "stream-json");
    }

    #[test]
    fn preview_contract_rejects_missing_run_and_unknown_kind() {
        let missing_run = preview_external_agent_contract(request(
            "codex_cli",
            " ",
            "Review the active thread.",
            "read_only",
        ));
        let unknown_kind = preview_external_agent_contract(request(
            "unknown",
            "run-1",
            "Review the active thread.",
            "read_only",
        ));

        assert!(missing_run.is_err());
        assert!(unknown_kind.is_err());
    }

    fn request(
        kind: &str,
        run_id: &str,
        task: &str,
        permission_mode: &str,
    ) -> ExternalAgentContractPreviewRequest {
        ExternalAgentContractPreviewRequest {
            kind: kind.to_string(),
            permission_mode: permission_mode.to_string(),
            run_id: run_id.to_string(),
            task: task.to_string(),
            working_directory: "C:/repo".to_string(),
        }
    }
}
