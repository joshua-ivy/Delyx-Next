#[cfg(test)]
mod tests {
    use crate::agent_patch_draft_step::{
        scheduler_patch_draft_context_request, AgentPatchDraftStepRequest,
    };
    use crate::approval_bridge::{
        decide_approval_record, propose_approval_record, ApprovalBridgeState,
        ApprovalDecisionRequest, ApprovalProposalRequest, PermissionScopeView,
    };
    use crate::patch_bridge::PatchBridgeState;
    use crate::plan_bridge::{ExploreView, PlanBridgeState, PlanView};
    use crate::review_bridge::ReviewBridgeState;
    use crate::test_runner_bridge::TestRunnerBridgeState;
    use crate::thread_run_bridge::{create_thread_run_record, ThreadRunCreateRequest};
    use crate::thread_run_bridge_state::ThreadRunBridgeState;
    use crate::workspace_bridge::{WorkspaceGitView, WorkspaceIsolationView, WorkspaceProjectView};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn patch_draft_step_selects_exact_scheduler_approval() {
        let fixture = Fixture::new("patch-draft-step-approved");
        save_project_and_plan(&fixture.path, &fixture.thread_id);
        let approval_id = approved_file_write(
            &fixture.approvals,
            &fixture.run_id,
            &format!("{}-plan-approval", fixture.run_id),
            vec!["src/main.ts"],
        );

        let context = scheduler_patch_draft_context_request(
            &fixture.threads,
            &fixture.patches,
            &fixture.tests,
            &fixture.reviews,
            &fixture.approvals,
            &fixture.plans,
            &step_request(&fixture.run_id),
        )
        .unwrap();

        assert_eq!(context.approval_id, approval_id);
        assert_eq!(context.project_id, "project-1");
        assert_eq!(context.run_id, fixture.run_id);
        assert_eq!(context.model, "qwen3-coder:30b");
        assert_eq!(context.max_bytes_per_file, Some(20_000));
        assert!(!context.has_supported_test_command);
        assert_eq!(context.test_approval_id, None);
        let _ = fs::remove_file(fixture.path);
    }

    #[test]
    fn patch_draft_step_rejects_generic_file_write_approval() {
        let fixture = Fixture::new("patch-draft-step-generic");
        save_project_and_plan(&fixture.path, &fixture.thread_id);
        approved_file_write(
            &fixture.approvals,
            &fixture.run_id,
            "generic-node",
            vec!["src/main.ts"],
        );

        let error = scheduler_patch_draft_context_request(
            &fixture.threads,
            &fixture.patches,
            &fixture.tests,
            &fixture.reviews,
            &fixture.approvals,
            &fixture.plans,
            &step_request(&fixture.run_id),
        )
        .unwrap_err();

        assert!(error.contains("instead of PatchDraft"));
        let _ = fs::remove_file(fixture.path);
    }

    struct Fixture {
        approvals: ApprovalBridgeState,
        path: PathBuf,
        patches: PatchBridgeState,
        plans: PlanBridgeState,
        reviews: ReviewBridgeState,
        run_id: String,
        tests: TestRunnerBridgeState,
        thread_id: String,
        threads: ThreadRunBridgeState,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let path = temp_db(name);
            let threads = ThreadRunBridgeState::default();
            let record = {
                let mut store = threads.store.lock().unwrap();
                create_thread_run_record(
                    &mut store,
                    ThreadRunCreateRequest {
                        created_at: "2026-06-08T00:00:00.000Z".to_string(),
                        goal: "Draft the patch.".to_string(),
                        project_id: "project-1".to_string(),
                    },
                )
                .unwrap()
            };
            Self {
                approvals: ApprovalBridgeState::default(),
                path: path.clone(),
                patches: PatchBridgeState::default(),
                plans: PlanBridgeState::persistent(path),
                reviews: ReviewBridgeState::default(),
                run_id: record.run.id,
                tests: TestRunnerBridgeState::default(),
                thread_id: record.thread.id,
                threads,
            }
        }
    }

    fn approved_file_write(
        approvals: &ApprovalBridgeState,
        run_id: &str,
        node_id: &str,
        paths: Vec<&str>,
    ) -> String {
        approvals
            .with_store_mut(|store| {
                let proposal = propose_approval_record(
                    store,
                    ApprovalProposalRequest {
                        action_type: "edit_file".to_string(),
                        client_id: format!("approval-{run_id}-{node_id}"),
                        expected_result: "Draft a patch proposal.".to_string(),
                        expires_at: "2999-01-01T00:00:00.000Z".to_string(),
                        expires_at_ms: 32_503_680_000_000,
                        node_id: node_id.to_string(),
                        rationale: "Allow PatchDraft to propose a diff.".to_string(),
                        required_permission: "edit_file".to_string(),
                        risk_label: "high".to_string(),
                        rollback_plan: Some("PatchDraft does not write files.".to_string()),
                        run_id: run_id.to_string(),
                        scope: PermissionScopeView {
                            commands: None,
                            connector_id: None,
                            kind: "file".to_string(),
                            paths: Some(paths.into_iter().map(str::to_string).collect()),
                            project_id: Some("project-1".to_string()),
                            root: Some("C:/repo".to_string()),
                            summary: "Draft scoped patch.".to_string(),
                        },
                    },
                )?;
                decide_approval_record(
                    store,
                    ApprovalDecisionRequest {
                        decided_at_ms: 42,
                        decision: "approved".to_string(),
                        note: None,
                        proposal_id: proposal.id.clone(),
                    },
                )?;
                Ok(proposal.id)
            })
            .unwrap()
    }

    fn save_project_and_plan(path: &PathBuf, thread_id: &str) {
        crate::workspace_persistence::save_recent_project(path, &project()).unwrap();
        crate::plan_persistence::save_plan_to_path(path, "project-1", &plan(thread_id)).unwrap();
    }

    fn step_request(run_id: &str) -> AgentPatchDraftStepRequest {
        AgentPatchDraftStepRequest {
            max_bytes_per_file: Some(20_000),
            model: "qwen3-coder:30b".to_string(),
            now_ms: 43,
            project_id: "project-1".to_string(),
            run_id: run_id.to_string(),
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
            files_likely_involved: vec!["src/main.ts".to_string()],
            goal_understanding: "Draft patch context.".to_string(),
            permissions_needed: vec!["edit_file".to_string()],
            risks: Vec::new(),
            rollback_strategy: "PatchDraft only proposes a diff.".to_string(),
            steps: vec!["Update src/main.ts".to_string()],
            tests_to_run: Vec::new(),
            thread_id: thread_id.to_string(),
        }
    }

    fn project() -> WorkspaceProjectView {
        WorkspaceProjectView {
            approval_policy: "manual".to_string(),
            approved_roots: vec!["C:/repo".to_string()],
            git: WorkspaceGitView {
                branch: "main".to_string(),
                is_repo: true,
                uncommitted_changes: Some(0),
            },
            id: "project-1".to_string(),
            indexed_files: vec!["src/main.ts".to_string()],
            isolation: WorkspaceIsolationView {
                detail: "none".to_string(),
                label: "none".to_string(),
                mode: "none".to_string(),
            },
            last_opened_label: "now".to_string(),
            name: "Repo".to_string(),
            path: "C:/repo".to_string(),
            pinned: true,
            rules_files: Vec::new(),
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
