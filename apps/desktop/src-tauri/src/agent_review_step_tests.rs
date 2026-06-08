#[cfg(test)]
mod tests {
    use crate::agent_review_step::{
        execute_scheduler_review_record, scheduler_review_request, AgentReviewStepRequest,
    };
    use crate::approval_bridge::ApprovalBridgeStore;
    use crate::patch_bridge::{DiffLineView, PatchBridgeStore, PatchFileView, PatchProposalView};
    use crate::review_bridge::ReviewBridgeStore;
    use crate::test_runner_bridge::TestRunnerBridgeStore;
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use crate::threads::ThreadStatus;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn review_step_executes_scheduler_selected_review() {
        let db = temp_db("review-step-exec");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        move_to_testing(&mut threads, &record.thread.id);
        let mut patches = PatchBridgeStore::default();
        patches.records.push(restored_patch(&record.run.id));
        let tests = TestRunnerBridgeStore::default();
        let mut reviews = ReviewBridgeStore::default();
        let approvals = ApprovalBridgeStore::default();

        let request = scheduler_review_request(
            &threads,
            &approvals,
            &patches,
            &tests,
            &reviews,
            &db,
            &step_request(&record.run.id),
        )
        .unwrap();
        let view = execute_scheduler_review_record(
            &mut threads,
            &patches,
            &tests,
            &mut reviews,
            request,
            "2026-06-08T01:00:00.000Z",
        )
        .unwrap();

        assert_eq!(view.status, "completed");
        assert_eq!(view.review_report_id, Some("review-1".to_string()));
        assert_eq!(reviews.reports.len(), 1);
        assert_eq!(
            threads
                .manager
                .get_thread(&record.thread.id)
                .unwrap()
                .status,
            ThreadStatus::Reviewing
        );
        assert_eq!(threads.records[0].updated_at, "2026-06-08T01:00:00.000Z");
        let _ = fs::remove_file(db);
    }

    #[test]
    fn review_step_rejects_when_review_already_exists() {
        let db = temp_db("review-step-existing");
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, thread_request()).unwrap();
        let mut patches = PatchBridgeStore::default();
        patches.records.push(restored_patch(&record.run.id));
        let tests = TestRunnerBridgeStore::default();
        let mut reviews = ReviewBridgeStore::default();
        let approvals = ApprovalBridgeStore::default();
        let first = scheduler_review_request(
            &threads,
            &approvals,
            &patches,
            &tests,
            &reviews,
            &db,
            &step_request(&record.run.id),
        )
        .unwrap();
        move_to_building(&mut threads, &record.thread.id);
        execute_scheduler_review_record(
            &mut threads,
            &patches,
            &tests,
            &mut reviews,
            first,
            "2026-06-08T01:00:00.000Z",
        )
        .unwrap();

        let error = scheduler_review_request(
            &threads,
            &approvals,
            &patches,
            &tests,
            &reviews,
            &db,
            &step_request(&record.run.id),
        )
        .unwrap_err();

        assert!(error.contains("instead of review"));
        let _ = fs::remove_file(db);
    }

    fn restored_patch(run_id: &str) -> PatchProposalView {
        PatchProposalView {
            approval_id: "approval-1".to_string(),
            checkpoint_files: Vec::new(),
            checkpoint_id: None,
            files: vec![PatchFileView {
                after: "let value = maybe.unwrap();".to_string(),
                before: String::new(),
                change_kind: "create".to_string(),
                diff: vec![DiffLineView {
                    kind: "added".to_string(),
                    text: "let value = maybe.unwrap();".to_string(),
                }],
                path: "src/main.rs".to_string(),
            }],
            id: "patch-1".to_string(),
            restore_approval_id: None,
            run_id: run_id.to_string(),
            status: "restored".to_string(),
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

    fn move_to_testing(threads: &mut ThreadRunStore, thread_id: &str) {
        move_to_building(threads, thread_id);
        threads
            .manager
            .set_status(thread_id, ThreadStatus::Testing)
            .unwrap();
    }

    fn step_request(run_id: &str) -> AgentReviewStepRequest {
        AgentReviewStepRequest {
            now_ms: 42,
            run_id: run_id.to_string(),
            updated_at: "2026-06-08T01:00:00.000Z".to_string(),
        }
    }

    fn thread_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Review the generated patch.".to_string(),
            project_id: "project-1".to_string(),
        }
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
