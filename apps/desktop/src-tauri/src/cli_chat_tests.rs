#[cfg(test)]
mod tests {
    use crate::cli_chat::{cli_chat_command, cli_chat_text};
    use crate::command_exec::{CommandExecArtifact, CommandExecStatus};
    use std::path::PathBuf;

    #[test]
    fn claude_uses_print_mode_and_codex_uses_read_only_sandbox() {
        let (claude_program, claude_args) =
            cli_chat_command("claude-code", "explain this bug").unwrap();
        assert_eq!(claude_program, "claude");
        assert_eq!(claude_args, vec!["-p", "explain this bug"]);

        let (codex_program, codex_args) =
            cli_chat_command("codex-cli", "explain this bug").unwrap();
        assert_eq!(codex_program, "codex");
        assert!(codex_args.contains(&"--sandbox".to_string()));
        assert!(codex_args.contains(&"read-only".to_string()));
    }

    #[test]
    fn unknown_adapter_is_rejected() {
        assert!(cli_chat_command("gemini-cli", "hi")
            .unwrap_err()
            .contains("not supported"));
    }

    #[test]
    fn successful_output_is_trimmed_to_text() {
        let text = cli_chat_text(&artifact(
            CommandExecStatus::Succeeded,
            "  the answer  \n",
            "",
        ))
        .unwrap();
        assert_eq!(text, "the answer");
    }

    #[test]
    fn failed_command_surfaces_stderr() {
        let error =
            cli_chat_text(&artifact(CommandExecStatus::Failed, "", "not logged in")).unwrap_err();
        assert!(error.contains("not logged in"));
        assert!(error.contains("failed"));
    }

    #[test]
    fn empty_output_is_an_error() {
        let error = cli_chat_text(&artifact(CommandExecStatus::Succeeded, "   ", "")).unwrap_err();
        assert!(error.contains("no output"));
    }

    fn artifact(status: CommandExecStatus, stdout: &str, stderr: &str) -> CommandExecArtifact {
        CommandExecArtifact {
            approval_id: "cli-chat".to_string(),
            command: "claude -p test".to_string(),
            completed_at_ms: 10,
            duration_ms: 5,
            events: Vec::new(),
            exit_code: Some(if status == CommandExecStatus::Succeeded {
                0
            } else {
                1
            }),
            run_id: "cli-chat".to_string(),
            started_at_ms: 5,
            status,
            stderr: stderr.to_string(),
            stderr_truncated: false,
            stdout: stdout.to_string(),
            stdout_truncated: false,
            timeout_ms: 60_000,
            working_directory: PathBuf::from("C:/repo"),
        }
    }
}
