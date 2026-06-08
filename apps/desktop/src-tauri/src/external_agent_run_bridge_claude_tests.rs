#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::external_agent::{
        AdapterStatus, ExternalAgentAvailability, ExternalAgentBridge, ExternalAgentKind,
    };
    use crate::external_agent_command_contracts::{
        ExternalAgentCommandContract, ExternalAgentPermissionMode,
    };
    use crate::external_agent_run_bridge::{
        external_agent_run_snapshot_from_store, run_contract_agent_record,
        ExternalAgentCodexRunRequest, ExternalAgentRunBridgeStore,
    };
    use crate::external_agent_terminal::ExternalAgentCommand;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn claude_read_only_requires_both_approvals() {
        let root = temp_workspace("claude-needs-both-approvals");
        let mut approvals = ApprovalEngine::new();
        let external = approvals.propose(external_agent_input());
        approvals
            .approve(&external.id, 10, "approved external")
            .unwrap();
        let terminal = approvals.propose(terminal_command_input());
        let mut store = ExternalAgentRunBridgeStore::default();
        let mut bridge = claude_bridge(&root);

        let result = run_contract_agent_record(
            &mut store,
            &approvals,
            read_only_request(&external.id, &terminal.id, &root),
            fake_claude_contract(&root, passing_command()),
            &mut bridge,
        );

        assert!(result.unwrap_err().contains("NotApproved"));
        assert!(external_agent_run_snapshot_from_store(&store, "run-1").is_empty());
    }

    #[test]
    fn claude_write_requires_isolation() {
        let root = temp_workspace("claude-write-needs-isolation");
        let (approvals, external_id, terminal_id) = approved_pair();
        let mut store = ExternalAgentRunBridgeStore::default();
        let mut bridge = claude_bridge(&root);

        let result = run_contract_agent_record(
            &mut store,
            &approvals,
            read_only_request(&external_id, &terminal_id, &root),
            fake_claude_write_contract(&root, passing_command()),
            &mut bridge,
        );

        assert!(result
            .unwrap_err()
            .contains("requires a checkpoint or isolated worktree"));
    }

    #[test]
    fn claude_stream_error_result_marks_artifact_failed() {
        let root = temp_workspace("claude-stream-error");
        fs::write(
            root.join("stream.jsonl"),
            "{\"type\":\"result\",\"subtype\":\"error\",\"is_error\":true,\"result\":\"boom\"}\n",
        )
        .unwrap();
        let (approvals, external_id, terminal_id) = approved_pair();
        let mut store = ExternalAgentRunBridgeStore::default();
        let mut bridge = claude_bridge(&root);

        let artifact = run_contract_agent_record(
            &mut store,
            &approvals,
            read_only_request(&external_id, &terminal_id, &root),
            fake_claude_contract(&root, stream_file_command("stream.jsonl")),
            &mut bridge,
        )
        .unwrap();

        // The worker process exits 0 (`type`/`cat`), but the parsed result is an error.
        assert_eq!(artifact.status, "failed");
        assert!(artifact
            .transcript
            .iter()
            .any(|event| event.kind == "stderr" && event.message.contains("is_error")));
    }

    fn read_only_request(
        external_approval_id: &str,
        terminal_approval_id: &str,
        root: &std::path::Path,
    ) -> ExternalAgentCodexRunRequest {
        ExternalAgentCodexRunRequest {
            allowed_paths: vec![root.display().to_string()],
            approved_roots: vec![root.display().to_string()],
            capture_diff: false,
            changed_files: vec![],
            checkpoint_id: None,
            created_at_ms: 10,
            external_approval_id: external_approval_id.to_string(),
            permission_mode: "read_only".to_string(),
            run_id: "run-1".to_string(),
            task: "Inspect the project and report findings.".to_string(),
            terminal_approval_id: terminal_approval_id.to_string(),
            test_artifact_ids: vec![],
            timeout_ms: 60_000,
            working_directory: root.display().to_string(),
            worktree_id: None,
        }
    }

    fn claude_bridge(root: &std::path::Path) -> ExternalAgentBridge {
        ExternalAgentBridge::with_adapters(
            vec![root.to_path_buf()],
            vec![ExternalAgentAvailability {
                adapter_id: "claude-code".to_string(),
                detail: "available in deterministic test".to_string(),
                display_name: "Claude Code".to_string(),
                kind: ExternalAgentKind::ClaudeCode,
                status: AdapterStatus::Available,
            }],
        )
        .unwrap()
    }

    fn fake_claude_contract(
        root: &std::path::Path,
        command: (String, Vec<String>),
    ) -> ExternalAgentCommandContract {
        ExternalAgentCommandContract {
            adapter_id: "claude-code".to_string(),
            command: ExternalAgentCommand {
                args: command.1,
                program: command.0,
                working_directory: root.to_path_buf(),
            },
            kind: ExternalAgentKind::ClaudeCode,
            permission_mode: ExternalAgentPermissionMode::ReadOnly,
            required_delyx_tools: vec![
                "external_agent".to_string(),
                "terminal_command".to_string(),
            ],
            safety_summary: "test contract requires approvals".to_string(),
            transcript_format: "stream-json".to_string(),
        }
    }

    fn fake_claude_write_contract(
        root: &std::path::Path,
        command: (String, Vec<String>),
    ) -> ExternalAgentCommandContract {
        ExternalAgentCommandContract {
            permission_mode: ExternalAgentPermissionMode::WorkspaceWrite,
            ..fake_claude_contract(root, command)
        }
    }

    fn approved_pair() -> (ApprovalEngine, String, String) {
        let mut approvals = ApprovalEngine::new();
        let external = approvals.propose(external_agent_input());
        let terminal = approvals.propose(terminal_command_input());
        approvals
            .approve(&external.id, 10, "approved external")
            .unwrap();
        approvals
            .approve(&terminal.id, 10, "approved terminal")
            .unwrap();
        (approvals, external.id, terminal.id)
    }

    fn external_agent_input() -> ProposalInput {
        ProposalInput {
            action: RiskyAction::ExternalAgentExecution,
            expected_result: "Claude Code runs inside approved scope.".to_string(),
            expires_at: 100,
            node_id: "node-claude".to_string(),
            reason: "Launch approved Claude Code adapter.".to_string(),
            risk: RiskLevel::High,
            rollback_plan: "Restore checkpoint or discard isolated worktree if rejected."
                .to_string(),
            run_id: "run-1".to_string(),
            scope: "Run Claude in one approved project root.".to_string(),
        }
    }

    fn terminal_command_input() -> ProposalInput {
        ProposalInput {
            action: RiskyAction::TerminalCommand,
            expected_result: "Capture Claude worker command output.".to_string(),
            node_id: "node-claude-terminal".to_string(),
            reason: "Run the Claude Code command under Delyx capture.".to_string(),
            rollback_plan: "No durable mutation from the command artifact itself.".to_string(),
            scope: "Run one Claude terminal command inside the approved root.".to_string(),
            ..external_agent_input()
        }
    }

    fn passing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec!["/C".to_string(), "echo claude stream".to_string()],
            )
        } else {
            (
                "sh".to_string(),
                vec!["-c".to_string(), "echo claude stream".to_string()],
            )
        }
    }

    fn stream_file_command(file: &str) -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec!["/C".to_string(), format!("type {file}")],
            )
        } else {
            (
                "sh".to_string(),
                vec!["-c".to_string(), format!("cat {file}")],
            )
        }
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        path
    }
}
