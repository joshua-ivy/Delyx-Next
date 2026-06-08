#[cfg(test)]
mod tests {
    use crate::agent_drive::{drive_run, AgentDriveContext};
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
    fn drive_applies_approved_patch_then_yields_without_tests() {
        let fixture = Fixture::new("drive-apply");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        move_to_building(&mut threads, &record.thread.id);
        let mut patches = PatchBridgeStore::default();
        patches
            .records
            .push(file_patch(&record.run.id, &fixture.file));
        let mut approvals = ApprovalBridgeStore::default();
        approved_apply(&mut approvals, &record.run.id);
        let mut tests = TestRunnerBridgeStore::default();
        let mut reviews = ReviewBridgeStore::default();
        save_project(&fixture.db, &fixture.root);

        let outcome = drive_run(
            &mut context(
                &mut threads,
                &approvals,
                &mut patches,
                &mut tests,
                &mut reviews,
                &fixture.db,
            ),
            |_, _, _, _| Ok(()),
        )
        .unwrap();

        assert_eq!(decisions(&outcome), vec!["run_patch_apply"]);
        assert_eq!(outcome.stopped_because.kind, "blocked");
        assert_eq!(fs::read_to_string(&fixture.file).unwrap(), "after\n");
        assert_eq!(patches.records[0].status, "applied");
        fixture.cleanup();
    }

    #[test]
    fn drive_halts_on_failed_apply_node() {
        let fixture = Fixture::new("drive-apply-fail");
        // The file no longer matches the proposal `before`, so apply must fail.
        fs::write(&fixture.file, "drifted\n").unwrap();
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        move_to_building(&mut threads, &record.thread.id);
        let mut patches = PatchBridgeStore::default();
        patches
            .records
            .push(file_patch(&record.run.id, &fixture.file));
        let mut approvals = ApprovalBridgeStore::default();
        approved_apply(&mut approvals, &record.run.id);
        let mut tests = TestRunnerBridgeStore::default();
        let mut reviews = ReviewBridgeStore::default();
        save_project(&fixture.db, &fixture.root);

        let outcome = drive_run(
            &mut context(
                &mut threads,
                &approvals,
                &mut patches,
                &mut tests,
                &mut reviews,
                &fixture.db,
            ),
            |_, _, _, _| Ok(()),
        )
        .unwrap();

        // Apply is the only step, the driver halts, and no later steps run.
        assert_eq!(decisions(&outcome), vec!["run_patch_apply"]);
        assert_eq!(outcome.stopped_because.kind, "failed");
        assert_ne!(patches.records[0].status, "applied");
        assert_eq!(fs::read_to_string(&fixture.file).unwrap(), "drifted\n");
        fixture.cleanup();
    }

    fn context<'a>(
        threads: &'a mut ThreadRunStore,
        approvals: &'a ApprovalBridgeStore,
        patches: &'a mut PatchBridgeStore,
        tests: &'a mut TestRunnerBridgeStore,
        reviews: &'a mut ReviewBridgeStore,
        db: &'a Path,
    ) -> AgentDriveContext<'a> {
        AgentDriveContext {
            approvals,
            final_summary: None,
            now_ms: 42,
            patches,
            plan_db: db,
            reviews,
            run_id: "run-1".to_string(),
            tests,
            threads,
            timeout_ms: Some(60_000),
            updated_at: "2026-06-08T01:00:00.000Z".to_string(),
        }
    }

    fn file_patch(run_id: &str, file: &Path) -> PatchProposalView {
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

    fn approved_apply(approvals: &mut ApprovalBridgeStore, run_id: &str) {
        let proposal = propose_approval_record(
            approvals,
            ApprovalProposalRequest {
                action_type: "edit_file".to_string(),
                client_id: "approval-apply".to_string(),
                expected_result: "Apply one proposed patch.".to_string(),
                expires_at: "2999-01-01T00:00:00.000Z".to_string(),
                expires_at_ms: 32_503_680_000_000,
                node_id: format!("{run_id}-patch-apply-patch-1"),
                rationale: "Allow apply.".to_string(),
                required_permission: "write_file".to_string(),
                risk_label: "high".to_string(),
                rollback_plan: Some("Use checkpoint receipts.".to_string()),
                run_id: run_id.to_string(),
                scope: PermissionScopeView {
                    commands: None,
                    connector_id: None,
                    kind: "file".to_string(),
                    paths: Some(vec!["settings.toml".to_string()]),
                    project_id: Some("project-1".to_string()),
                    root: None,
                    summary: "Apply one patch.".to_string(),
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
                proposal_id: proposal.id,
            },
        )
        .unwrap();
    }

    fn move_to_building(threads: &mut ThreadRunStore, thread_id: &str) {
        threads
            .manager
            .set_status(thread_id, ThreadStatus::Planning)
            .unwrap();
        threads
            .manager
            .set_status(thread_id, ThreadStatus::Building)
            .unwrap();
    }

    fn thread_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Drive the run.".to_string(),
            project_id: "project-1".to_string(),
        }
    }

    fn save_project(db: &Path, root: &Path) {
        crate::workspace_persistence::save_recent_project(db, &project(root)).unwrap();
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

    fn decisions(outcome: &crate::agent_drive_types::AgentDriveOutcomeView) -> Vec<&str> {
        outcome
            .steps
            .iter()
            .map(|step| step.decision.as_str())
            .collect()
    }

    struct Fixture {
        db: PathBuf,
        file: PathBuf,
        root: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = temp_workspace(label);
            let file = root.join("settings.toml");
            fs::write(&file, "before\n").unwrap();
            Self {
                db: temp_db(label),
                file,
                root,
            }
        }

        fn cleanup(self) {
            let _ = fs::remove_dir_all(self.root);
            let _ = fs::remove_file(self.db);
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
