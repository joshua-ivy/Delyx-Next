#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::test_runner_bridge::{
        run_test_record, test_snapshot_from_store, TestRunRequest, TestRunnerBridgeStore,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn approved_test_run_returns_ui_ready_artifact() {
        let root = temp_workspace("bridge-pass");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(terminal_approval(100));
        approvals
            .approve(&approval.id, 10, "approved test")
            .unwrap();
        let mut store = TestRunnerBridgeStore::default();

        let artifact = run_test_record(
            &mut store,
            &approvals,
            request(&approval.id, &root, passing_command()),
        )
        .unwrap();

        assert_eq!(artifact.status, "passed");
        assert_eq!(artifact.run_id, "run-1");
        assert_eq!(artifact.approval_id.as_deref(), Some(approval.id.as_str()));
        assert!(artifact.stdout.to_lowercase().contains("cargo"));
        assert_eq!(test_snapshot_from_store(&store, "run-1"), vec![artifact]);
    }

    #[test]
    fn pending_approval_blocks_test_execution_without_artifact() {
        let root = temp_workspace("bridge-pending");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(terminal_approval(100));
        let mut store = TestRunnerBridgeStore::default();

        let result = run_test_record(
            &mut store,
            &approvals,
            request(&approval.id, &root, passing_command()),
        );

        assert!(result.unwrap_err().contains("NotApproved"));
        assert!(test_snapshot_from_store(&store, "run-1").is_empty());
    }

    #[test]
    fn non_test_command_is_rejected_without_artifact() {
        let root = temp_workspace("bridge-non-test");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(terminal_approval(100));
        approvals
            .approve(&approval.id, 10, "approved test")
            .unwrap();
        let mut store = TestRunnerBridgeStore::default();

        let result = run_test_record(
            &mut store,
            &approvals,
            request(
                &approval.id,
                &root,
                ("cargo".to_string(), vec!["build".to_string()]),
            ),
        );

        assert!(result.unwrap_err().contains("NotTestCommand"));
        assert!(test_snapshot_from_store(&store, "run-1").is_empty());
    }

    #[test]
    fn snapshot_filters_test_artifacts_by_run() {
        let root = temp_workspace("bridge-snapshot");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(terminal_approval(100));
        approvals
            .approve(&approval.id, 10, "approved test")
            .unwrap();
        let mut store = TestRunnerBridgeStore::default();
        run_test_record(
            &mut store,
            &approvals,
            request(&approval.id, &root, passing_command()),
        )
        .unwrap();

        assert_eq!(test_snapshot_from_store(&store, "run-1").len(), 1);
        assert!(test_snapshot_from_store(&store, "run-2").is_empty());
    }

    fn request(
        approval_id: &str,
        root: &std::path::Path,
        command: (String, Vec<String>),
    ) -> TestRunRequest {
        TestRunRequest {
            approval_id: approval_id.to_string(),
            approved_roots: vec![root.display().to_string()],
            args: command.1,
            completed_at: Some("2026-06-07T00:00:01.000Z".to_string()),
            created_at_ms: 10,
            program: command.0,
            run_id: "run-1".to_string(),
            started_at: "2026-06-07T00:00:00.000Z".to_string(),
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

    fn terminal_approval(expires_at: u64) -> ProposalInput {
        ProposalInput {
            action: RiskyAction::TerminalCommand,
            expected_result: "Capture test command output.".to_string(),
            expires_at,
            node_id: "node-test-runner".to_string(),
            reason: "Bridge test artifact execution.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "No durable mutation; artifact can be discarded.".to_string(),
            run_id: "run-1".to_string(),
            scope: "Run one test command in an approved root.".to_string(),
        }
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
