#[cfg(test)]
mod tests {
    use crate::agent_test_executor_bridge::execute_test_run_record;
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::test_runner_bridge::{TestRunRequest, TestRunnerBridgeStore};
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn bridge_executes_test_run_against_thread_run_store() {
        let root = temp_workspace("test-executor-bridge");
        let mut thread_store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut thread_store, create_request()).unwrap();
        let mut test_store = TestRunnerBridgeStore::default();
        let (approvals, approval_id) = approved_terminal(&record.run.id);

        let view = execute_test_run_record(
            &mut thread_store,
            &mut test_store,
            &approvals,
            request(&record.run.id, &approval_id, &root),
        )
        .unwrap();

        assert_eq!(view.status, "completed");
        assert_eq!(test_store.artifacts[0].status, "passed");
        let run = thread_store.ledger.get_run(&record.run.id).unwrap();
        assert!(run.artifacts.iter().any(|artifact| artifact.kind == "test"));
        assert!(run
            .evidence
            .iter()
            .any(|evidence| evidence.source_kind == "test"));
        let _ = fs::remove_dir_all(root);
    }

    fn create_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Run tests through executor bridge".to_string(),
            project_id: "proj-1".to_string(),
        }
    }

    fn approved_terminal(run_id: &str) -> (ApprovalEngine, String) {
        let mut engine = ApprovalEngine::new();
        let proposal = engine.propose(ProposalInput {
            action: RiskyAction::TerminalCommand,
            expected_result: "Capture approved test command output.".to_string(),
            expires_at: 10_000,
            node_id: "agent-test-execution-bridge".to_string(),
            reason: "Run tests as part of the agent loop.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "No durable mutation; artifact can be discarded.".to_string(),
            run_id: run_id.to_string(),
            scope: "One test command in an approved root.".to_string(),
        });
        engine.approve(&proposal.id, 1, "test approval").unwrap();
        (engine, proposal.id)
    }

    fn request(run_id: &str, approval_id: &str, root: &Path) -> TestRunRequest {
        TestRunRequest {
            approval_id: approval_id.to_string(),
            approved_roots: vec![root.display().to_string()],
            args: vec!["test".to_string(), "--help".to_string()],
            completed_at: Some("2026-06-08T00:00:01.000Z".to_string()),
            created_at_ms: 2,
            program: "cargo".to_string(),
            run_id: run_id.to_string(),
            started_at: "2026-06-08T00:00:00.000Z".to_string(),
            timeout_ms: 60_000,
            working_directory: root.display().to_string(),
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
