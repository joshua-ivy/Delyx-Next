#[cfg(test)]
mod tests {
    use crate::agent_drive::{drive_run, AgentDriveContext};
    use crate::agent_run::AgentRunStatus;
    use crate::approval_bridge::ApprovalBridgeStore;
    use crate::patch_bridge::{DiffLineView, PatchBridgeStore, PatchFileView, PatchProposalView};
    use crate::review_bridge::ReviewBridgeStore;
    use crate::test_runner_bridge::TestRunnerBridgeStore;
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use crate::threads::ThreadStatus;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn drive_stops_at_ungranted_apply_approval() {
        let db = temp_db("drive-apply-boundary");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        let mut patches = PatchBridgeStore::default();
        patches
            .records
            .push(patch(&record.run.id, "proposed", "let value = 1;"));
        let mut tests = TestRunnerBridgeStore::default();
        let mut reviews = ReviewBridgeStore::default();
        let approvals = ApprovalBridgeStore::default();
        let mut persists = 0;

        let outcome = drive_run(
            &mut context(
                &mut threads,
                &approvals,
                &mut patches,
                &mut tests,
                &mut reviews,
                &db,
            ),
            |_, _, _, _| {
                persists += 1;
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(outcome.steps.len(), 0);
        assert_eq!(outcome.stopped_because.kind, "needs_patch_apply_approval");
        assert_eq!(
            outcome.stopped_because.proposal_id,
            Some("patch-1".to_string())
        );
        assert_eq!(persists, 0);
        let _ = fs::remove_file(db);
    }

    #[test]
    fn drive_runs_review_then_final_support() {
        let db = temp_db("drive-review-final");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        move_to_building(&mut threads, &record.thread.id);
        let mut patches = PatchBridgeStore::default();
        patches
            .records
            .push(patch(&record.run.id, "restored", "let value = 1;"));
        let mut tests = TestRunnerBridgeStore::default();
        let mut reviews = ReviewBridgeStore::default();
        let approvals = ApprovalBridgeStore::default();
        let mut persists = 0;

        let mut ctx = context(
            &mut threads,
            &approvals,
            &mut patches,
            &mut tests,
            &mut reviews,
            &db,
        );
        ctx.final_summary = Some("Implemented and reviewed.".to_string());
        let outcome = drive_run(&mut ctx, |_, _, _, _| {
            persists += 1;
            Ok(())
        })
        .unwrap();

        assert_eq!(
            decisions(&outcome),
            vec!["run_review", "ready_for_final_support"]
        );
        assert_eq!(outcome.stopped_because.kind, "completed");
        assert_eq!(reviews.reports.len(), 1);
        assert_eq!(
            threads.ledger.get_run(&record.run.id).unwrap().status,
            AgentRunStatus::Completed
        );
        assert_eq!(
            threads
                .manager
                .get_thread(&record.thread.id)
                .unwrap()
                .status,
            ThreadStatus::Done
        );
        assert_eq!(persists, 2);
        let _ = fs::remove_file(db);
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

    fn patch(run_id: &str, status: &str, added: &str) -> PatchProposalView {
        PatchProposalView {
            approval_id: "approval-draft".to_string(),
            checkpoint_files: Vec::new(),
            checkpoint_id: None,
            files: vec![PatchFileView {
                after: added.to_string(),
                before: String::new(),
                change_kind: "create".to_string(),
                diff: vec![DiffLineView {
                    kind: "added".to_string(),
                    text: added.to_string(),
                }],
                path: "src/main.rs".to_string(),
            }],
            id: "patch-1".to_string(),
            restore_approval_id: None,
            run_id: run_id.to_string(),
            status: status.to_string(),
        }
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

    fn decisions(outcome: &crate::agent_drive_types::AgentDriveOutcomeView) -> Vec<&str> {
        outcome
            .steps
            .iter()
            .map(|step| step.decision.as_str())
            .collect()
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
