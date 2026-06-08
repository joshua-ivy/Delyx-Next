#[cfg(test)]
mod tests {
    use crate::agent_scheduler_bridge::AgentScheduleRequest;
    use crate::agent_scheduler_patch_context::hydrate_patch_draft_request;
    use crate::approval_bridge::{
        decide_approval_record, propose_approval_record, ApprovalBridgeStore,
        ApprovalDecisionRequest, ApprovalProposalRequest, PermissionScopeView,
    };
    use crate::patch_bridge::{PatchBridgeStore, PatchProposalView};
    use crate::plan_bridge::{ExploreView, PlanView};
    use crate::review_bridge::{ReviewBridgeStore, ReviewFindingView, ReviewReportView};
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use crate::workspace_bridge::{WorkspaceGitView, WorkspaceIsolationView, WorkspaceProjectView};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn persisted_plan_approval_hydrates_patch_draft_context() {
        let path = temp_db("scheduler-patch-draft-plan");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        save_project_and_plan(&path, &record.thread.id);
        let mut approvals = ApprovalBridgeStore::default();
        let approval_id = approved_file_write(
            &mut approvals,
            &record.run.id,
            &format!("{}-plan-approval", record.run.id),
            vec!["src/main.ts"],
        );

        let hydrated = hydrate_patch_draft_request(
            &threads,
            &approvals,
            &PatchBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &path,
            request(&record.run.id),
        )
        .unwrap();

        assert_eq!(
            hydrated.patch_draft_approval_id.as_deref(),
            Some(approval_id.as_str())
        );
    }

    #[test]
    fn generic_file_write_approval_does_not_hydrate_patch_draft_context() {
        let path = temp_db("scheduler-patch-draft-generic");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        save_project_and_plan(&path, &record.thread.id);
        let mut approvals = ApprovalBridgeStore::default();
        approved_file_write(
            &mut approvals,
            &record.run.id,
            "generic-node",
            vec!["src/main.ts"],
        );

        let hydrated = hydrate_patch_draft_request(
            &threads,
            &approvals,
            &PatchBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &path,
            AgentScheduleRequest {
                patch_draft_approval_id: Some("ui-hint".to_string()),
                ..request(&record.run.id)
            },
        )
        .unwrap();

        assert_eq!(hydrated.patch_draft_approval_id, None);
    }

    #[test]
    fn persisted_repair_approval_hydrates_patch_draft_context() {
        let path = temp_db("scheduler-patch-draft-repair");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        save_project_and_plan(&path, &record.thread.id);
        let mut approvals = ApprovalBridgeStore::default();
        let approval_id = approved_file_write(
            &mut approvals,
            &record.run.id,
            &format!("{}-repair-review-1-finding-1", record.run.id),
            vec!["src/main.ts"],
        );
        let mut reviews = ReviewBridgeStore::default();
        reviews.reports.push(repair_review(&record.run.id));

        let hydrated = hydrate_patch_draft_request(
            &threads,
            &approvals,
            &PatchBridgeStore::default(),
            &reviews,
            &path,
            request(&record.run.id),
        )
        .unwrap();

        assert_eq!(
            hydrated.patch_draft_approval_id.as_deref(),
            Some(approval_id.as_str())
        );
    }

    #[test]
    fn existing_patch_prevents_plan_patch_draft_hydration() {
        let path = temp_db("scheduler-patch-draft-existing");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        save_project_and_plan(&path, &record.thread.id);
        let mut approvals = ApprovalBridgeStore::default();
        approved_file_write(
            &mut approvals,
            &record.run.id,
            &format!("{}-plan-approval", record.run.id),
            vec!["src/main.ts"],
        );
        let mut patches = PatchBridgeStore::default();
        patches.records.push(patch(&record.run.id));

        let hydrated = hydrate_patch_draft_request(
            &threads,
            &approvals,
            &patches,
            &ReviewBridgeStore::default(),
            &path,
            request(&record.run.id),
        )
        .unwrap();

        assert_eq!(hydrated.patch_draft_approval_id, None);
    }

    fn approved_file_write(
        approvals: &mut ApprovalBridgeStore,
        run_id: &str,
        node_id: &str,
        paths: Vec<&str>,
    ) -> String {
        let proposal = propose_approval_record(
            approvals,
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
        )
        .unwrap();
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

    fn save_project_and_plan(path: &PathBuf, thread_id: &str) {
        crate::workspace_persistence::save_recent_project(path, &project()).unwrap();
        crate::plan_persistence::save_plan_to_path(path, "project-1", &plan(thread_id)).unwrap();
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
            goal: "Draft the patch.".to_string(),
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

    fn repair_review(run_id: &str) -> ReviewReportView {
        ReviewReportView {
            decision: "revise_requested".to_string(),
            evidence_summary: "Stored review.".to_string(),
            findings: vec![ReviewFindingView {
                detail: "Fix parser output.".to_string(),
                file_path: "C:/repo/src/main.ts".to_string(),
                hunk_label: "patch-1:0".to_string(),
                id: "finding-1".to_string(),
                priority: "p1".to_string(),
                risk_label: "panic".to_string(),
                suggested_fix: "Handle the missing value.".to_string(),
                title: "Missing guard".to_string(),
            }],
            id: "review-1".to_string(),
            mode: "read_only".to_string(),
            risk_summary: "1 finding.".to_string(),
            run_id: run_id.to_string(),
            test_summary: "Tests passed.".to_string(),
        }
    }

    fn patch(run_id: &str) -> PatchProposalView {
        PatchProposalView {
            approval_id: "approval-1".to_string(),
            checkpoint_files: Vec::new(),
            checkpoint_id: None,
            files: Vec::new(),
            id: "patch-1".to_string(),
            restore_approval_id: None,
            run_id: run_id.to_string(),
            status: "proposed".to_string(),
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
