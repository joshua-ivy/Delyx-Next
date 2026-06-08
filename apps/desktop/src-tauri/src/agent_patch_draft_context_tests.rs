#[cfg(test)]
mod tests {
    use crate::agent_patch_draft_context::{
        context_execute_request, AgentPatchDraftContextRequest,
    };
    use crate::approval_bridge::{
        decide_approval_record, propose_approval_record, ApprovalBridgeState,
        ApprovalDecisionRequest, ApprovalProposalRequest, PermissionScopeView,
    };
    use crate::plan_bridge::{ExploreView, PlanBridgeState, PlanView};
    use crate::review_bridge::ReviewBridgeState;
    use crate::thread_run_bridge::{create_thread_run_record, ThreadRunCreateRequest};
    use crate::thread_run_bridge_state::ThreadRunBridgeState;
    use crate::workspace_bridge::{WorkspaceGitView, WorkspaceIsolationView, WorkspaceProjectView};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn context_dispatch_builds_patch_draft_request_from_persisted_state() {
        let path = temp_db("patch-draft-context");
        let threads = ThreadRunBridgeState::default();
        let approvals = ApprovalBridgeState::default();
        let reviews = ReviewBridgeState::default();
        let plans = PlanBridgeState::persistent(path.clone());
        let record = {
            let mut store = threads.store.lock().unwrap();
            create_thread_run_record(
                &mut store,
                ThreadRunCreateRequest {
                    created_at: "2026-06-08T00:00:00.000Z".to_string(),
                    goal: "Update parser output.".to_string(),
                    project_id: "project-1".to_string(),
                },
            )
            .unwrap()
        };
        crate::workspace_persistence::save_recent_project(&path, &project()).unwrap();
        crate::plan_persistence::save_plan_to_path(&path, "project-1", &plan(&record.thread.id))
            .unwrap();
        let approval_id = approved_patch_draft_scope(&approvals);

        let execute = context_execute_request(
            &threads,
            &reviews,
            &approvals,
            &plans,
            &AgentPatchDraftContextRequest {
                approval_id: approval_id.clone(),
                has_supported_test_command: true,
                max_bytes_per_file: Some(12_000),
                model: "qwen3-coder:30b".to_string(),
                now_ms: 42,
                project_id: "project-1".to_string(),
                run_id: record.run.id,
                test_approval_id: None,
            },
        )
        .unwrap();

        assert_eq!(execute.approval_id, approval_id);
        assert_eq!(execute.approved_roots, vec!["C:/repo"]);
        assert_eq!(execute.files_likely_involved, vec!["src/main.ts"]);
        assert_eq!(execute.goal, "Update parser output.");
        assert_eq!(execute.plan_steps, vec!["Change parser output"]);
        assert_eq!(execute.project_path, "C:/repo");
        assert_eq!(execute.scope_paths, vec!["src/main.ts"]);
        let _ = fs::remove_file(path);
    }

    fn approved_patch_draft_scope(approvals: &ApprovalBridgeState) -> String {
        approvals
            .with_store_mut(|store| {
                let proposal = propose_approval_record(
                    store,
                    ApprovalProposalRequest {
                        action_type: "edit_file".to_string(),
                        client_id: "approval-client-1".to_string(),
                        expected_result: "Draft a patch proposal.".to_string(),
                        expires_at: "2999-01-01T00:00:00.000Z".to_string(),
                        expires_at_ms: 32_503_680_000_000,
                        node_id: "node-1".to_string(),
                        rationale: "Need a scoped patch draft.".to_string(),
                        required_permission: "edit_file".to_string(),
                        risk_label: "high".to_string(),
                        rollback_plan: Some("PatchDraft does not write files.".to_string()),
                        run_id: "run-1".to_string(),
                        scope: PermissionScopeView {
                            commands: None,
                            connector_id: None,
                            kind: "file".to_string(),
                            paths: Some(vec!["src/main.ts".to_string()]),
                            project_id: Some("project-1".to_string()),
                            root: Some("C:/repo".to_string()),
                            summary: "Draft src/main.ts".to_string(),
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

    fn plan(thread_id: &str) -> PlanView {
        PlanView {
            decision: "approved".to_string(),
            explore: ExploreView {
                architecture_summary: "Rust app.".to_string(),
                project_commands: vec!["cargo test --workspace".to_string()],
                relevant_files: vec!["src/main.ts".to_string()],
                relevant_symbols: Vec::new(),
                risks: Vec::new(),
                suggested_next_steps: Vec::new(),
                unknowns: Vec::new(),
            },
            files_likely_involved: vec!["src/main.ts".to_string(), "src/other.ts".to_string()],
            goal_understanding: "Update parser output.".to_string(),
            permissions_needed: vec!["edit_file".to_string()],
            rollback_strategy: "Restore checkpoint.".to_string(),
            risks: Vec::new(),
            steps: vec!["Change parser output".to_string()],
            tests_to_run: vec!["cargo test --workspace".to_string()],
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

    fn temp_db(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
