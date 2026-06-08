#[cfg(test)]
mod tests {
    use crate::agent_scheduler_bridge::AgentScheduleRequest;
    use crate::agent_scheduler_test_context::hydrate_schedule_request;
    use crate::approval_bridge::{
        decide_approval_record, propose_approval_record, ApprovalBridgeStore,
        ApprovalDecisionRequest, ApprovalProposalRequest, PermissionScopeView,
    };
    use crate::plan_bridge::{ExploreView, PlanView};
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn persisted_plan_and_approval_hydrate_test_scheduler_context() {
        let path = temp_db("scheduler-test-context");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        crate::plan_persistence::save_plan_to_path(
            &path,
            "project-1",
            &plan(&record.thread.id, vec!["npm test"]),
        )
        .unwrap();
        let mut approvals = ApprovalBridgeStore::default();
        let approval_id = approved_test_command(&mut approvals, &record.run.id, "npm test");

        let hydrated =
            hydrate_schedule_request(&threads, &approvals, &path, request(&record.run.id)).unwrap();

        assert!(hydrated.has_supported_test_command);
        assert_eq!(
            hydrated.test_approval_id.as_deref(),
            Some(approval_id.as_str())
        );
        fs::remove_file(path).ok();
    }

    #[test]
    fn shell_control_test_text_is_not_hydrated_as_supported() {
        let path = temp_db("scheduler-test-unsafe");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        crate::plan_persistence::save_plan_to_path(
            &path,
            "project-1",
            &plan(&record.thread.id, vec!["npm test && whoami"]),
        )
        .unwrap();

        let hydrated = hydrate_schedule_request(
            &threads,
            &ApprovalBridgeStore::default(),
            &path,
            AgentScheduleRequest {
                has_supported_test_command: true,
                test_approval_id: Some("ui-hint".to_string()),
                ..request(&record.run.id)
            },
        )
        .unwrap();

        assert!(!hydrated.has_supported_test_command);
        assert_eq!(hydrated.test_approval_id, None);
        fs::remove_file(path).ok();
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
            node_id: format!("{run_id}-test-npm-test"),
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
                root: Some("C:/repo".to_string()),
                summary: "Run one supported test command.".to_string(),
            },
        }
    }

    fn request(run_id: &str) -> AgentScheduleRequest {
        AgentScheduleRequest {
            has_supported_test_command: false,
            now_ms: 43,
            patch_apply_approval_id: None,
            patch_draft_approval_id: None,
            run_id: run_id.to_string(),
            test_approval_id: None,
        }
    }

    fn thread_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Run the tests.".to_string(),
            project_id: "project-1".to_string(),
        }
    }

    fn plan(thread_id: &str, tests_to_run: Vec<&str>) -> PlanView {
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
            files_likely_involved: vec!["src/main.ts".to_string()],
            goal_understanding: "Run test context.".to_string(),
            permissions_needed: vec!["run_terminal".to_string()],
            risks: Vec::new(),
            rollback_strategy: "No file writes.".to_string(),
            steps: vec!["Run tests.".to_string()],
            tests_to_run: tests_to_run.into_iter().map(str::to_string).collect(),
            thread_id: thread_id.to_string(),
        }
    }

    fn temp_db(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
