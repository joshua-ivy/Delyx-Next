#[cfg(test)]
mod tests {
    use crate::external_agent::{ExternalAgentError, ExternalAgentKind};
    use crate::external_agent_command_contracts::{
        build_external_agent_command_contract, ExternalAgentPermissionMode,
    };
    use std::path::PathBuf;

    #[test]
    fn codex_command_contract_uses_exec_json_and_read_only_sandbox() {
        let contract = build_external_agent_command_contract(
            ExternalAgentKind::CodexCli,
            "Review the auth module.",
            PathBuf::from("C:/repo"),
            ExternalAgentPermissionMode::ReadOnly,
        )
        .unwrap();

        assert_eq!(contract.adapter_id, "codex-cli");
        assert_eq!(contract.command.program, "codex");
        assert_eq!(
            contract.command.args,
            strings(&["exec", "--json", "--sandbox", "read-only", "Review the auth module."])
        );
        assert_eq!(contract.transcript_format, "jsonl");
        assert!(contract.required_delyx_tools.contains(&"external_agent".to_string()));
        assert!(contract.required_delyx_tools.contains(&"terminal_command".to_string()));
    }

    #[test]
    fn codex_workspace_write_contract_is_explicit() {
        let contract = build_external_agent_command_contract(
            ExternalAgentKind::CodexCli,
            "Implement the approved plan.",
            PathBuf::from("C:/repo"),
            ExternalAgentPermissionMode::WorkspaceWrite,
        )
        .unwrap();

        assert!(contract.command.args.contains(&"workspace-write".to_string()));
        assert!(contract.safety_summary.contains("diff review"));
    }

    #[test]
    fn claude_command_contract_uses_headless_stream_json() {
        let contract = build_external_agent_command_contract(
            ExternalAgentKind::ClaudeCode,
            "Inspect failures.",
            PathBuf::from("C:/repo"),
            ExternalAgentPermissionMode::ReadOnly,
        )
        .unwrap();

        assert_eq!(contract.adapter_id, "claude-code");
        assert_eq!(contract.command.program, "claude");
        assert_eq!(contract.command.args, strings(&[
            "-p",
            "--output-format",
            "stream-json",
            "--permission-mode",
            "plan",
            "--tools",
            "Read",
            "Inspect failures.",
        ]));
        assert_eq!(contract.transcript_format, "stream-json");
    }

    #[test]
    fn claude_workspace_write_contract_limits_tools_to_read_and_edit() {
        let contract = build_external_agent_command_contract(
            ExternalAgentKind::ClaudeCode,
            "Apply the accepted fix.",
            PathBuf::from("C:/repo"),
            ExternalAgentPermissionMode::WorkspaceWrite,
        )
        .unwrap();

        assert!(contract.command.args.contains(&"acceptEdits".to_string()));
        assert!(contract.command.args.contains(&"Read,Edit".to_string()));
        assert!(!contract.command.args.iter().any(|arg| arg.contains("Bash")));
    }

    #[test]
    fn command_contract_rejects_empty_tasks_and_generic_terminal() {
        let empty = build_external_agent_command_contract(
            ExternalAgentKind::CodexCli,
            " ",
            PathBuf::from("C:/repo"),
            ExternalAgentPermissionMode::ReadOnly,
        );
        let generic = build_external_agent_command_contract(
            ExternalAgentKind::GenericTerminal,
            "Run a command.",
            PathBuf::from("C:/repo"),
            ExternalAgentPermissionMode::ReadOnly,
        );

        assert_eq!(empty.unwrap_err(), ExternalAgentError::EmptyTask);
        assert_eq!(generic.unwrap_err(), ExternalAgentError::AdapterUnavailable);
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }
}
