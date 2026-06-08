#[cfg(test)]
mod tests {
    use crate::agent_run::{AgentRunLedger, AgentRunStatus};
    use crate::agent_test_executor::{execute_test_run_node, AgentTestExecutionStatus};
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::test_runner_bridge::{TestRunRequest, TestRunnerBridgeStore};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_run_node_waits_without_artifact() {
        let root = temp_workspace("test-executor-wait");
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut tests = TestRunnerBridgeStore::default();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(approval_input(&run.id));

        let result = execute_test_run_node(
            &mut ledger,
            &mut tests,
            &approvals,
            request(&run.id, &approval.id, &root, passing_command()),
        )
        .unwrap();

        assert_eq!(result.status, AgentTestExecutionStatus::WaitingForApproval);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::WaitingForApproval
        );
        assert!(tests.artifacts.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn approved_test_run_node_records_test_receipts() {
        let root = temp_workspace("test-executor-pass");
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut tests = TestRunnerBridgeStore::default();
        let (approvals, approval_id) = approved_terminal(&run.id);

        let result = execute_test_run_node(
            &mut ledger,
            &mut tests,
            &approvals,
            request(&run.id, &approval_id, &root, passing_command()),
        )
        .unwrap();
        let run = ledger.get_run(&run.id).unwrap();

        assert_eq!(result.status, AgentTestExecutionStatus::Completed);
        assert_eq!(tests.artifacts[0].status, "passed");
        assert_eq!(run.nodes[0].kind, "test_execution");
        assert_eq!(run.nodes[0].status, AgentRunStatus::Completed);
        assert_eq!(run.artifacts[0].kind, "test");
        assert_eq!(run.evidence[0].source_kind, "test");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn non_test_command_fails_without_artifact() {
        let root = temp_workspace("test-executor-reject");
        let mut ledger = AgentRunLedger::new();
        let run = ledger.create_run("thread-1").unwrap();
        let mut tests = TestRunnerBridgeStore::default();
        let (approvals, approval_id) = approved_terminal(&run.id);

        let result = execute_test_run_node(
            &mut ledger,
            &mut tests,
            &approvals,
            request(
                &run.id,
                &approval_id,
                &root,
                ("cargo".to_string(), vec!["build".to_string()]),
            ),
        )
        .unwrap();

        assert_eq!(result.status, AgentTestExecutionStatus::Failed);
        assert_eq!(
            ledger.get_run(&run.id).unwrap().status,
            AgentRunStatus::Failed
        );
        assert!(tests.artifacts.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    fn approved_terminal(run_id: &str) -> (ApprovalEngine, String) {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(approval_input(run_id));
        engine.approve(&proposal.id, 1, "test approval").unwrap();
        (engine, proposal.id)
    }

    fn approval_input(run_id: &str) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::TerminalCommand,
            expected_result: "Capture approved test command output.".to_string(),
            expires_at: 10_000,
            node_id: "agent-test-execution".to_string(),
            reason: "Run tests as part of the agent loop.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "No durable mutation; artifact can be discarded.".to_string(),
            run_id: run_id.to_string(),
            scope: "One test command in an approved root.".to_string(),
        }
    }

    fn request(
        run_id: &str,
        approval_id: &str,
        root: &Path,
        command: (String, Vec<String>),
    ) -> TestRunRequest {
        TestRunRequest {
            approval_id: approval_id.to_string(),
            approved_roots: vec![root.display().to_string()],
            args: command.1,
            completed_at: Some("2026-06-08T00:00:01.000Z".to_string()),
            created_at_ms: 2,
            program: command.0,
            run_id: run_id.to_string(),
            started_at: "2026-06-08T00:00:00.000Z".to_string(),
            timeout_ms: 60_000,
            working_directory: root.display().to_string(),
        }
    }

    fn passing_command() -> (String, Vec<String>) {
        (
            "cargo".to_string(),
            vec!["test".to_string(), "--help".to_string()],
        )
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
