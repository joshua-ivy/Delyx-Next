#[cfg(test)]
mod tests {
    use crate::agent_test_step::{
        execute_scheduler_test_record, scheduler_test_request, AgentTestStepRequest,
    };
    use crate::approval_bridge::{
        decide_approval_record, propose_approval_record, ApprovalBridgeStore,
        ApprovalDecisionRequest, ApprovalProposalRequest, PermissionScopeView,
    };
    use crate::patch_bridge::PatchBridgeStore;
    use crate::plan_bridge::{ExploreView, PlanView};
    use crate::review_bridge::ReviewBridgeStore;
    use crate::test_runner_bridge::TestRunnerBridgeStore;
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use crate::threads::ThreadStatus;
    use crate::workspace_bridge::{WorkspaceGitView, WorkspaceIsolationView, WorkspaceProjectView};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_step_executes_scheduler_selected_test_command() {
        let fixture = Fixture::new("test-step-exec");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        threads
            .manager
            .set_status(&record.thread.id, ThreadStatus::Planning)
            .unwrap();
        let mut patches = PatchBridgeStore::default();
        patches.records.push(applied_patch(&record.run.id));
        let mut approvals = ApprovalBridgeStore::default();
        let approval_id =
            approved_test_command(&mut approvals, &record.run.id, "cargo test --help");
        save_project_and_plan(&fixture.path, &fixture.root, &record.thread.id);
        let request = scheduler_test_request(
            &threads,
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &fixture.path,
            &step_request(&record.run.id),
        )
        .unwrap();

        assert_eq!(request.approval_id, approval_id);
        assert_eq!(request.program, "cargo");
        assert_eq!(request.args, vec!["test", "--help"]);
        assert_eq!(
            request.working_directory,
            fixture.root.display().to_string()
        );
        assert_eq!(
            request.approved_roots,
            vec![fixture.root.display().to_string()]
        );
        let mut tests = TestRunnerBridgeStore::default();
        let view = execute_scheduler_test_record(
            &mut threads,
            &mut tests,
            &approvals.engine,
            request,
            "2026-06-08T01:00:00.000Z",
            "2026-06-08T01:00:01.000Z",
        )
        .unwrap();

        assert_eq!(view.status, "completed");
        assert_eq!(tests.artifacts[0].status, "passed");
        assert_eq!(
            threads
                .manager
                .get_thread(&record.thread.id)
                .unwrap()
                .status,
            ThreadStatus::Reviewing
        );
        assert_eq!(threads.records[0].updated_at, "2026-06-08T01:00:01.000Z");
        fixture.cleanup();
    }

    #[test]
    fn test_step_rejects_missing_executable_test_approval() {
        let fixture = Fixture::new("test-step-no-approval");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        let mut patches = PatchBridgeStore::default();
        patches.records.push(applied_patch(&record.run.id));
        save_project_and_plan(&fixture.path, &fixture.root, &record.thread.id);

        let error = scheduler_test_request(
            &threads,
            &ApprovalBridgeStore::default(),
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &fixture.path,
            &step_request(&record.run.id),
        )
        .unwrap_err();

        assert!(error.contains("executable test approval"));
        fixture.cleanup();
    }

    struct Fixture {
        path: PathBuf,
        root: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = temp_workspace(label);
            Self {
                path: temp_db(label),
                root,
            }
        }

        fn cleanup(self) {
            let _ = fs::remove_dir_all(self.root);
            let _ = fs::remove_file(self.path);
        }
    }

    fn approved_test_command(
        approvals: &mut ApprovalBridgeStore,
        run_id: &str,
        command: &str,
    ) -> String {
        let proposal =
            propose_approval_record(approvals, approval_request(run_id, command)).unwrap();
        decide_approval_record(
            approvals,
            ApprovalDecisionRequest {
                decided_at_ms: 42,
                decision: "approved".to_string(),
                note: None,
                proposal_id: proposal.id.clone(),
            },
        )
        .unwrap();
        proposal.id
    }

    fn approval_request(run_id: &str, command: &str) -> ApprovalProposalRequest {
        ApprovalProposalRequest {
            action_type: "run_terminal".to_string(),
            client_id: format!("approval-{run_id}-test"),
            expected_result: "Run tests and capture an artifact.".to_string(),
            expires_at: "2999-01-01T00:00:00.000Z".to_string(),
            expires_at_ms: 32_503_680_000_000,
            node_id: format!("{run_id}-test-cargo-test-help"),
            rationale: "Validate the applied patch.".to_string(),
            required_permission: "terminal_command".to_string(),
            risk_label: "medium".to_string(),
            rollback_plan: Some("Discard the captured test artifact.".to_string()),
            run_id: run_id.to_string(),
            scope: PermissionScopeView {
                commands: Some(vec![command.to_string()]),
                connector_id: None,
                kind: "terminal".to_string(),
                paths: None,
                project_id: Some("project-1".to_string()),
                root: None,
                summary: "Run one supported test command.".to_string(),
            },
        }
    }

    fn applied_patch(run_id: &str) -> crate::patch_bridge::PatchProposalView {
        crate::patch_bridge::PatchProposalView {
            approval_id: "approval-draft".to_string(),
            checkpoint_files: Vec::new(),
            checkpoint_id: Some("checkpoint-1".to_string()),
            files: Vec::new(),
            id: "patch-1".to_string(),
            restore_approval_id: None,
            run_id: run_id.to_string(),
            status: "applied".to_string(),
        }
    }

    fn save_project_and_plan(path: &Path, root: &Path, thread_id: &str) {
        crate::workspace_persistence::save_recent_project(path, &project(root)).unwrap();
        crate::plan_persistence::save_plan_to_path(path, "project-1", &plan(thread_id)).unwrap();
    }

    fn step_request(run_id: &str) -> AgentTestStepRequest {
        AgentTestStepRequest {
            now_ms: 43,
            run_id: run_id.to_string(),
            started_at: "2026-06-08T01:00:00.000Z".to_string(),
            timeout_ms: Some(60_000),
            updated_at: "2026-06-08T01:00:01.000Z".to_string(),
        }
    }

    fn thread_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Run the tests.".to_string(),
            project_id: "project-1".to_string(),
        }
    }

    fn plan(thread_id: &str) -> PlanView {
        PlanView {
            decision: "approved".to_string(),
            explore: ExploreView {
                architecture_summary: String::new(),
                project_commands: Vec::new(),
                relevant_files: Vec::new(),
                relevant_symbols: Vec::new(),
                risks: Vec::new(),
                suggested_next_steps: Vec::new(),
                unknowns: Vec::new(),
            },
            files_likely_involved: vec!["src/lib.rs".to_string()],
            goal_understanding: "Run test step.".to_string(),
            permissions_needed: vec!["run_terminal".to_string()],
            risks: Vec::new(),
            rollback_strategy: "No file writes.".to_string(),
            steps: vec!["Run cargo test help.".to_string()],
            tests_to_run: vec!["cargo test --help".to_string()],
            thread_id: thread_id.to_string(),
        }
    }

    fn project(root: &Path) -> WorkspaceProjectView {
        WorkspaceProjectView {
            approval_policy: "manual".to_string(),
            approved_roots: vec![root.display().to_string()],
            git: WorkspaceGitView {
                branch: "main".to_string(),
                is_repo: true,
                uncommitted_changes: Some(0),
            },
            id: "project-1".to_string(),
            indexed_files: vec!["src/lib.rs".to_string()],
            isolation: WorkspaceIsolationView {
                detail: "none".to_string(),
                label: "none".to_string(),
                mode: "none".to_string(),
            },
            last_opened_label: "now".to_string(),
            name: "Repo".to_string(),
            path: root.display().to_string(),
            pinned: true,
            rules_files: Vec::new(),
        }
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{}", stamp()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn temp_db(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("delyx-next-{label}-{}.sqlite3", stamp()))
    }

    fn stamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
