#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::test_runner_bridge::{
        run_test_record, test_snapshot_from_store, TestRunRequest, TestRunnerBridgeStore,
    };
    use crate::test_runner_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_artifact_bridge_store_survives_sqlite_reload() {
        let root = temp_workspace("test-artifact-persistence");
        let db_path = root.join("delyx.sqlite3");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(terminal_approval());
        approvals
            .approve(&approval.id, 10, "approved test")
            .unwrap();
        let mut store = TestRunnerBridgeStore::default();

        let artifact = run_test_record(
            &mut store,
            &approvals,
            request(&approval.id, &root, failing_command()),
        )
        .unwrap();
        let second = run_test_record(
            &mut store,
            &approvals,
            request(&approval.id, &root, failing_command()),
        )
        .unwrap();
        save_to_path(&store, &db_path).unwrap();
        let loaded = load_from_path(&db_path).unwrap();
        let loaded_snapshot = test_snapshot_from_store(&loaded, "run-1");

        assert!(fs::read(&db_path).unwrap().starts_with(b"SQLite format 3"));
        assert_eq!(loaded_snapshot, vec![artifact.clone(), second.clone()]);
        assert_ne!(artifact.id, second.id);
        assert_eq!(artifact.status, "failed");
        assert!(artifact
            .parsed_failures
            .as_ref()
            .is_some_and(|failures| failures.len() == 1));
        assert!(artifact
            .exec_events
            .iter()
            .any(|event| event.kind == "failed"));
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

    fn failing_command() -> (String, Vec<String>) {
        (
            "cargo".to_string(),
            vec![
                "test".to_string(),
                "--manifest-path".to_string(),
                "missing-Cargo.toml".to_string(),
            ],
        )
    }

    fn terminal_approval() -> ProposalInput {
        ProposalInput {
            action: RiskyAction::TerminalCommand,
            expected_result: "Capture test command output.".to_string(),
            expires_at: 100,
            node_id: "node-test-runner".to_string(),
            reason: "Persist bridge test artifact execution.".to_string(),
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
