#[cfg(test)]
mod tests {
    use crate::agent_review_revision_bridge::{
        request_review_revision_record, AgentReviewRevisionRequest,
    };
    use crate::agent_run::AgentRunStatus;
    use crate::review_bridge::{
        create_review_record, DiffLineReviewInput, PatchFileReviewInput, PatchReviewInput,
        ReviewBridgeStore, ReviewCreateRequest,
    };
    use crate::thread_run_bridge::{
        create_thread_run_record, update_thread_status_record, ThreadRunCreateRequest,
        ThreadRunStore, ThreadStatusUpdateRequest,
    };

    #[test]
    fn request_revision_marks_review_and_records_repair_node() {
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, create_request()).unwrap();
        move_to_reviewing(&mut threads, &record.thread.id);
        let mut reviews = ReviewBridgeStore::default();
        let report = create_review_record(&mut reviews, review_request(&record.run.id)).unwrap();
        let finding_id = report.findings[0].id.clone();

        let result = request_review_revision_record(
            &mut threads,
            &mut reviews,
            revision_request(&record.run.id, &report.id, &finding_id),
        )
        .unwrap();

        assert_eq!(result.status, "revise_requested");
        assert_eq!(reviews.reports[0].decision, "revise_requested");
        assert_eq!(result.next_flow, vec!["plan", "build"]);
        assert_eq!(
            threads
                .manager
                .get_thread(&record.thread.id)
                .unwrap()
                .status,
            crate::threads::ThreadStatus::Building
        );
        let run = threads.ledger.get_run(&record.run.id).unwrap();
        assert_eq!(run.nodes[0].kind, "repair");
        assert_eq!(run.nodes[0].status, AgentRunStatus::Completed);
        assert_eq!(run.artifacts[0].kind, "review_revision");
        assert_eq!(run.events.last().unwrap().kind, "repair.requested");
    }

    #[test]
    fn request_revision_rejects_missing_finding_without_mutating_report() {
        let mut threads = ThreadRunStore::default();
        let record = create_thread_run_record(&mut threads, create_request()).unwrap();
        move_to_reviewing(&mut threads, &record.thread.id);
        let mut reviews = ReviewBridgeStore::default();
        let report = create_review_record(&mut reviews, review_request(&record.run.id)).unwrap();

        let result = request_review_revision_record(
            &mut threads,
            &mut reviews,
            revision_request(&record.run.id, &report.id, "missing-finding"),
        );

        assert!(result.unwrap_err().contains("finding"));
        assert_eq!(reviews.reports[0].decision, "pending");
        assert!(threads
            .ledger
            .get_run(&record.run.id)
            .unwrap()
            .nodes
            .is_empty());
    }

    fn create_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Review repair bridge".to_string(),
            project_id: "project-1".to_string(),
        }
    }

    fn move_to_reviewing(threads: &mut ThreadRunStore, thread_id: &str) {
        for status in ["planning", "building", "testing", "reviewing"] {
            update_thread_status_record(threads, status_request(thread_id, status)).unwrap();
        }
    }

    fn status_request(thread_id: &str, status: &str) -> ThreadStatusUpdateRequest {
        ThreadStatusUpdateRequest {
            status: status.to_string(),
            thread_id: thread_id.to_string(),
            updated_at: "2026-06-08T00:00:01.000Z".to_string(),
        }
    }

    fn revision_request(
        run_id: &str,
        report_id: &str,
        finding_id: &str,
    ) -> AgentReviewRevisionRequest {
        AgentReviewRevisionRequest {
            finding_id: finding_id.to_string(),
            review_report_id: report_id.to_string(),
            run_id: run_id.to_string(),
            updated_at: "2026-06-08T00:00:02.000Z".to_string(),
        }
    }

    fn review_request(run_id: &str) -> ReviewCreateRequest {
        ReviewCreateRequest {
            patches: vec![PatchReviewInput {
                approval_id: "approval-1".to_string(),
                files: vec![PatchFileReviewInput {
                    diff: vec![DiffLineReviewInput {
                        kind: "added".to_string(),
                        text: "let value = maybe.unwrap();".to_string(),
                    }],
                    path: "src/main.rs".to_string(),
                }],
                id: "patch-1".to_string(),
                run_id: run_id.to_string(),
                status: "proposed".to_string(),
            }],
            run_id: run_id.to_string(),
            tests: Vec::new(),
        }
    }
}
