#[cfg(test)]
mod tests {
    use crate::agent_patch_apply_step::{
        execute_scheduler_patch_apply_record, scheduler_patch_apply_request,
        AgentPatchApplyStepRequest,
    };
    use crate::approval_bridge::{
        decide_approval_record, propose_approval_record, ApprovalBridgeStore,
        ApprovalDecisionRequest, ApprovalProposalRequest, PermissionScopeView,
    };
    use crate::patch_bridge::{DiffLineView, PatchBridgeStore, PatchFileView, PatchProposalView};
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
    fn patch_apply_step_executes_scheduler_selected_apply() {
        let fixture = Fixture::new("patch-apply-step-exec");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        threads
            .manager
            .set_status(&record.thread.id, ThreadStatus::Planning)
            .unwrap();
        let mut patches = PatchBridgeStore::default();
        patches.records.push(patch(&record.run.id, &fixture.file));
        let mut approvals = ApprovalBridgeStore::default();
        let approval_id = approved_file_write(
            &mut approvals,
            &record.run.id,
            &format!("{}-patch-apply-patch-1", record.run.id),
        );
        save_project(&fixture.path, &fixture.root);

        let request = scheduler_patch_apply_request(
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
        assert_eq!(request.proposal_id, "patch-1");
        assert_eq!(
            request.approved_roots,
            vec![fixture.root.display().to_string()]
        );
        let view = execute_scheduler_patch_apply_record(
            &mut threads,
            &mut patches,
            &approvals.engine,
            request,
            "2026-06-08T01:00:00.000Z",
        )
        .unwrap();

        assert_eq!(view.status, "completed");
        assert_eq!(fs::read_to_string(&fixture.file).unwrap(), "after\n");
        assert_eq!(patches.records[0].status, "applied");
        assert_eq!(
            threads
                .manager
                .get_thread(&record.thread.id)
                .unwrap()
                .status,
            ThreadStatus::Testing
        );
        assert_eq!(threads.records[0].updated_at, "2026-06-08T01:00:00.000Z");
        fixture.cleanup();
    }

    #[test]
    fn patch_apply_step_rejects_generic_file_write_approval() {
        let fixture = Fixture::new("patch-apply-step-generic");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        let mut patches = PatchBridgeStore::default();
        patches.records.push(patch(&record.run.id, &fixture.file));
        let mut approvals = ApprovalBridgeStore::default();
        approved_file_write(&mut approvals, &record.run.id, "generic-file-write");
        save_project(&fixture.path, &fixture.root);

        let error = scheduler_patch_apply_request(
            &threads,
            &approvals,
            &patches,
            &TestRunnerBridgeStore::default(),
            &ReviewBridgeStore::default(),
            &fixture.path,
            &step_request(&record.run.id),
        )
        .unwrap_err();

        assert!(error.contains("instead of patch apply"));
        fixture.cleanup();
    }

    struct Fixture {
        file: PathBuf,
        path: PathBuf,
        root: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = temp_workspace(label);
            let file = root.join("settings.toml");
            fs::write(&file, "before\n").unwrap();
            Self {
                file,
                path: temp_db(label),
                root,
            }
        }

        fn cleanup(self) {
            let _ = fs::remove_dir_all(self.root);
            let _ = fs::remove_file(self.path);
        }
    }

    fn approved_file_write(
        approvals: &mut ApprovalBridgeStore,
        run_id: &str,
        node_id: &str,
    ) -> String {
        let proposal = propose_approval_record(
            approvals,
            ApprovalProposalRequest {
                action_type: "edit_file".to_string(),
                client_id: format!("approval-{run_id}-{node_id}"),
                expected_result: "Apply one proposed patch.".to_string(),
                expires_at: "2999-01-01T00:00:00.000Z".to_string(),
                expires_at_ms: 32_503_680_000_000,
                node_id: node_id.to_string(),
                rationale: "Allow the scheduler-selected patch apply.".to_string(),
                required_permission: "write_file".to_string(),
                risk_label: "high".to_string(),
                rollback_plan: Some(
                    "Use checkpoint receipts to restore changed files.".to_string(),
                ),
                run_id: run_id.to_string(),
                scope: PermissionScopeView {
                    commands: None,
                    connector_id: None,
                    kind: "file".to_string(),
                    paths: Some(vec!["settings.toml".to_string()]),
                    project_id: Some("project-1".to_string()),
                    root: None,
                    summary: "Apply one proposed patch inside the approved root.".to_string(),
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

    fn patch(run_id: &str, file: &Path) -> PatchProposalView {
        PatchProposalView {
            approval_id: "approval-draft".to_string(),
            checkpoint_files: Vec::new(),
            checkpoint_id: None,
            files: vec![PatchFileView {
                after: "after\n".to_string(),
                before: "before\n".to_string(),
                change_kind: "modify".to_string(),
                diff: vec![DiffLineView {
                    kind: "added".to_string(),
                    text: "after".to_string(),
                }],
                path: file.display().to_string(),
            }],
            id: "patch-1".to_string(),
            restore_approval_id: None,
            run_id: run_id.to_string(),
            status: "proposed".to_string(),
        }
    }

    fn step_request(run_id: &str) -> AgentPatchApplyStepRequest {
        AgentPatchApplyStepRequest {
            now_ms: 43,
            run_id: run_id.to_string(),
            updated_at: "2026-06-08T01:00:00.000Z".to_string(),
        }
    }

    fn thread_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Apply the generated patch.".to_string(),
            project_id: "project-1".to_string(),
        }
    }

    fn save_project(path: &Path, root: &Path) {
        crate::workspace_persistence::save_recent_project(path, &project(root)).unwrap();
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
            indexed_files: vec!["settings.toml".to_string()],
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
