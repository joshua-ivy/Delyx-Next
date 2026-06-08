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
    fn approved_codex_contract_runs_and_stores_ui_artifact() {
        let root = temp_workspace("codex-run");
        let (approvals, external_id, terminal_id) = approved_pair(100);
        let mut store = ExternalAgentRunBridgeStore::default();
        let mut bridge = codex_bridge(&root);

        let artifact = run_contract_agent_record(
            &mut store,
            &approvals,
            request(&external_id, &terminal_id, &root, Some("checkpoint-1")),
            fake_codex_contract(&root, mutating_command()),
            &mut bridge,
        )
        .unwrap();

        assert_eq!(artifact.adapter_id, "codex-cli");
        assert_eq!(artifact.status, "completed");
        assert!(artifact.terminal_output.contains("codex jsonl"));
        assert!(artifact.review_required);
        assert!(artifact
            .diff_summary
            .as_ref()
            .unwrap()
            .contains("1 modified"));
        assert!(artifact.transcript.iter().any(|event| {
            event.kind == "diff_captured"
                && event.message.contains("src")
                && event.message.contains("modified")
        }));
        assert_eq!(
            external_agent_run_snapshot_from_store(&store, "run-1"),
            vec![artifact]
        );
    }

    #[test]
    fn pending_terminal_approval_blocks_codex_launch_without_artifact() {
        let root = temp_workspace("codex-terminal-pending");
        let mut approvals = ApprovalEngine::new();
        let external = approvals.propose(external_agent_input(100));
        approvals
            .approve(&external.id, 10, "approved external")
            .unwrap();
        let terminal = approvals.propose(terminal_command_input(100));
        let mut store = ExternalAgentRunBridgeStore::default();
        let mut bridge = codex_bridge(&root);

        let result = run_contract_agent_record(
            &mut store,
            &approvals,
            request(&external.id, &terminal.id, &root, Some("checkpoint-1")),
            fake_codex_contract(&root, passing_command()),
            &mut bridge,
        );

        assert!(result.unwrap_err().contains("NotApproved"));
        assert!(external_agent_run_snapshot_from_store(&store, "run-1").is_empty());
    }

    #[test]
    fn write_codex_contract_creates_checkpoint_before_launch() {
        let root = temp_workspace("codex-auto-checkpoint");
        let (approvals, external_id, terminal_id) = approved_pair(100);
        let mut store = ExternalAgentRunBridgeStore::default();
        let mut bridge = codex_bridge(&root);

        let artifact = run_contract_agent_record(
            &mut store,
            &approvals,
            request(&external_id, &terminal_id, &root, None),
            fake_codex_write_contract(&root, passing_command()),
            &mut bridge,
        )
        .unwrap();

        assert_eq!(artifact.status, "completed");
        assert!(artifact
            .scope
            .contains("checkpoint external-agent-checkpoint"));
        assert!(artifact
            .transcript
            .iter()
            .any(|event| event.kind == "checkpoint_created"
                && event.message.contains("checkpointed:")));
    }

    #[test]
    fn read_only_codex_launch_can_run_without_checkpoint() {
        let root = temp_workspace("codex-read-only-no-isolation");
        let (approvals, external_id, terminal_id) = approved_pair(100);
        let mut store = ExternalAgentRunBridgeStore::default();
        let mut bridge = codex_bridge(&root);

        let artifact = run_contract_agent_record(
            &mut store,
            &approvals,
            read_only_request(&external_id, &terminal_id, &root),
            fake_codex_contract(&root, passing_command()),
            &mut bridge,
        )
        .unwrap();

        assert_eq!(artifact.status, "completed");
        assert!(artifact.scope.contains("no isolation"));
    }

    fn request(
        external_approval_id: &str,
        terminal_approval_id: &str,
        root: &std::path::Path,
        checkpoint_id: Option<&str>,
    ) -> ExternalAgentCodexRunRequest {
        ExternalAgentCodexRunRequest {
            allowed_paths: vec![root.display().to_string()],
            approved_roots: vec![root.display().to_string()],
            capture_diff: true,
            changed_files: vec![root.join("src").join("main.rs").display().to_string()],
            checkpoint_id: checkpoint_id.map(str::to_string),
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

    fn read_only_request(
        external_approval_id: &str,
        terminal_approval_id: &str,
        root: &std::path::Path,
    ) -> ExternalAgentCodexRunRequest {
        ExternalAgentCodexRunRequest {
            capture_diff: false,
            changed_files: vec![],
            ..request(external_approval_id, terminal_approval_id, root, None)
        }
    }

    fn fake_codex_contract(
        root: &std::path::Path,
        command: (String, Vec<String>),
    ) -> ExternalAgentCommandContract {
        ExternalAgentCommandContract {
            adapter_id: "codex-cli".to_string(),
            command: ExternalAgentCommand {
                args: command.1,
                program: command.0,
                working_directory: root.to_path_buf(),
            },
            kind: ExternalAgentKind::CodexCli,
            permission_mode: ExternalAgentPermissionMode::ReadOnly,
            required_delyx_tools: vec![
                "external_agent".to_string(),
                "terminal_command".to_string(),
            ],
            safety_summary: "test contract requires approvals".to_string(),
            transcript_format: "jsonl".to_string(),
        }
    }

    fn fake_codex_write_contract(
        root: &std::path::Path,
        command: (String, Vec<String>),
    ) -> ExternalAgentCommandContract {
        ExternalAgentCommandContract {
            permission_mode: ExternalAgentPermissionMode::WorkspaceWrite,
            ..fake_codex_contract(root, command)
        }
    }

    fn codex_bridge(root: &std::path::Path) -> ExternalAgentBridge {
        ExternalAgentBridge::with_adapters(
            vec![root.to_path_buf()],
            vec![ExternalAgentAvailability {
                adapter_id: "codex-cli".to_string(),
                detail: "available in deterministic test".to_string(),
                display_name: "Codex CLI".to_string(),
                kind: ExternalAgentKind::CodexCli,
                status: AdapterStatus::Available,
            }],
        )
        .unwrap()
    }

    fn approved_pair(expires_at: u64) -> (ApprovalEngine, String, String) {
        let mut approvals = ApprovalEngine::new();
        let external = approvals.propose(external_agent_input(expires_at));
        let terminal = approvals.propose(terminal_command_input(expires_at));
        approvals
            .approve(&external.id, 10, "approved external")
            .unwrap();
        approvals
            .approve(&terminal.id, 10, "approved terminal")
            .unwrap();
        (approvals, external.id, terminal.id)
    }

    fn external_agent_input(expires_at: u64) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::ExternalAgentExecution,
            expected_result: "Codex CLI runs inside approved scope.".to_string(),
            expires_at,
            node_id: "node-codex".to_string(),
            reason: "Launch approved Codex CLI adapter.".to_string(),
            risk: RiskLevel::High,
            rollback_plan:
                "Restore checkpoint or discard isolated worktree if changes are rejected."
                    .to_string(),
            run_id: "run-1".to_string(),
            scope: "Run Codex in one approved project root.".to_string(),
        }
    }

    fn terminal_command_input(expires_at: u64) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::TerminalCommand,
            expected_result: "Capture Codex worker command output.".to_string(),
            node_id: "node-codex-terminal".to_string(),
            reason: "Run the Codex CLI command under Delyx capture.".to_string(),
            rollback_plan: "No durable mutation from the command artifact itself.".to_string(),
            scope: "Run one Codex terminal command inside the approved root.".to_string(),
            ..external_agent_input(expires_at)
        }
    }

    fn passing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec!["/C".to_string(), "echo codex jsonl".to_string()],
            )
        } else {
            (
                "sh".to_string(),
                vec!["-c".to_string(), "echo codex jsonl".to_string()],
            )
        }
    }

    fn mutating_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec![
                    "/C".to_string(),
                    "echo changed> src\\main.rs && echo codex jsonl".to_string(),
                ],
            )
        } else {
            (
                "sh".to_string(),
                vec![
                    "-c".to_string(),
                    "printf 'changed\\n' > src/main.rs && echo codex jsonl".to_string(),
                ],
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
