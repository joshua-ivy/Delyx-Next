#[cfg(test)]
mod tests {
    use crate::agent_run::{EvidenceRecordInput, EvidenceRelevance};
    use crate::test_runner_bridge::{
        CommandExecEventView, TestArtifactView, TestRunnerBridgeStore,
    };
    use crate::thread_run_bridge::{
        create_thread_run_record, ThreadRunCreateRequest, ThreadRunStore,
    };
    use crate::thread_run_final_answer::{
        finalize_thread_record, passed_tests, ThreadFinalAnswerRequest,
    };
    use crate::thread_run_final_support::{ApprovalSupportRecord, FinalSupportInput};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn final_answer_support_links_existing_evidence_and_passed_tests() {
        let mut thread_store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut thread_store, create_request()).unwrap();
        thread_store
            .ledger
            .record_evidence_detail(&record.run.id, evidence_input())
            .unwrap();
        let tests = TestRunnerBridgeStore {
            artifacts: vec![
                test_artifact(&record.run.id, "test-pass", "passed"),
                test_artifact(&record.run.id, "test-fail", "failed"),
            ],
        };

        let view = finalize_thread_record(
            &mut thread_store,
            final_request(&record.thread.id),
            support(passed_tests(&tests, &record.run.id), Vec::new()),
        )
        .unwrap();

        let outcome = view.run.outcome.unwrap();
        assert_eq!(view.thread.status, "done");
        assert_eq!(view.run.status, "succeeded");
        assert_eq!(outcome["evidenceRecordIds"][0], "evidence-1");
        assert_eq!(outcome["testArtifactIds"][0], "test-pass");
        assert_eq!(outcome["testArtifactIds"].as_array().unwrap().len(), 1);
        assert!(view
            .run
            .events
            .iter()
            .any(|event| event.kind == "final_answer.support_synthesized"));
    }

    #[test]
    fn final_answer_support_synthesizes_artifacts_and_approval_receipts() {
        let mut thread_store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut thread_store, create_request()).unwrap();
        thread_store
            .ledger
            .record_evidence_detail(&record.run.id, evidence_input())
            .unwrap();
        thread_store
            .ledger
            .record_artifact(&record.run.id, "patch_proposal", "patch-1")
            .unwrap();
        thread_store
            .ledger
            .record_artifact(&record.run.id, "model_response", "PatchDraftAgent response")
            .unwrap();
        thread_store
            .ledger
            .record_artifact(&record.run.id, "review_report", "review-1")
            .unwrap();

        let view = finalize_thread_record(
            &mut thread_store,
            final_request(&record.thread.id),
            support(
                vec![
                    test_artifact(&record.run.id, "test-pass", "passed"),
                    test_artifact(&record.run.id, "test-pass", "passed"),
                    test_artifact(&record.run.id, "", "passed"),
                ],
                vec![approval_view(&record.run.id)],
            ),
        )
        .unwrap();

        let outcome = view.run.outcome.unwrap();
        let source_kinds = view
            .run
            .evidence
            .iter()
            .filter_map(|item| item["sourceKind"].as_str())
            .collect::<Vec<_>>();
        assert!(source_kinds.contains(&"approval"));
        assert!(source_kinds.contains(&"diff"));
        assert!(source_kinds.contains(&"local_file"));
        assert!(source_kinds.contains(&"model_call"));
        assert!(source_kinds.contains(&"review"));
        assert!(source_kinds.contains(&"terminal"));
        assert_eq!(outcome["testArtifactIds"].as_array().unwrap().len(), 1);
        assert!(outcome["evidenceRecordIds"].as_array().unwrap().len() >= 5);
    }

    #[test]
    fn final_answer_support_survives_thread_run_sqlite_reload() {
        let path = temp_db("final-answer-support");
        let mut store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut store, create_request()).unwrap();
        store
            .ledger
            .record_evidence_detail(&record.run.id, evidence_input())
            .unwrap();
        finalize_thread_record(
            &mut store,
            final_request(&record.thread.id),
            support(
                vec![test_artifact(&record.run.id, "test-artifact-1", "passed")],
                Vec::new(),
            ),
        )
        .unwrap();

        crate::thread_run_persistence::save_to_path(&store, &path).unwrap();
        let loaded = crate::thread_run_persistence::load_from_path(&path).unwrap();
        let snapshot = crate::thread_run_bridge::thread_run_snapshot_from_store(&loaded, "proj-1");
        let outcome = snapshot.runs[0].outcome.as_ref().unwrap();

        assert_eq!(snapshot.threads[0].status, "done");
        assert_eq!(outcome["summary"], "Final answer with real support.");
        assert_eq!(outcome["evidenceRecordIds"][0], "evidence-1");
        assert_eq!(outcome["testArtifactIds"][0], "test-artifact-1");
        assert!(snapshot.runs[0]
            .events
            .iter()
            .any(|event| event.kind == "final_answer.support_synthesized"));
        let _ = fs::remove_file(path);
    }

    fn create_request() -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-08T00:00:00.000Z".to_string(),
            goal: "Ship a supported final answer".to_string(),
            project_id: "proj-1".to_string(),
        }
    }

    fn final_request(thread_id: &str) -> ThreadFinalAnswerRequest {
        ThreadFinalAnswerRequest {
            summary: "Final answer with real support.".to_string(),
            thread_id: thread_id.to_string(),
            updated_at: "2026-06-08T00:01:00.000Z".to_string(),
        }
    }

    fn evidence_input() -> EvidenceRecordInput {
        EvidenceRecordInput {
            hash: Some("sha256:readme".to_string()),
            quote: Some("Delyx records support links.".to_string()),
            relevance: Some(EvidenceRelevance {
                reason: "README supports the claim.".to_string(),
                relationship: "doc".to_string(),
                score: 90,
            }),
            retrieved_at: "2026-06-08T00:00:30.000Z".to_string(),
            source_id: "file://README.md".to_string(),
            source_kind: "local_file".to_string(),
            title: "README.md".to_string(),
            uri: Some("file:///README.md".to_string()),
        }
    }

    fn approval_view(run_id: &str) -> ApprovalSupportRecord {
        let _ = run_id;
        ApprovalSupportRecord {
            action_type: "edit_file".to_string(),
            id: "approval-1".to_string(),
            scope: "Edit src/main.rs".to_string(),
            status: "approved".to_string(),
        }
    }

    fn support(
        test_artifacts: Vec<TestArtifactView>,
        approval_records: Vec<ApprovalSupportRecord>,
    ) -> FinalSupportInput {
        FinalSupportInput {
            approval_records,
            test_artifacts,
        }
    }

    fn test_artifact(run_id: &str, id: &str, status: &str) -> TestArtifactView {
        TestArtifactView {
            approval_id: Some("approval-1".to_string()),
            command: "cargo test --workspace".to_string(),
            completed_at: "2026-06-08T00:00:40.000Z".to_string(),
            cwd: "C:/workspace".to_string(),
            duration_ms: 12,
            exec_events: Vec::<CommandExecEventView>::new(),
            exit_code: Some(if status == "passed" { 0 } else { 1 }),
            failure_summary: None,
            id: id.to_string(),
            output_truncated: false,
            parsed_failures: None,
            run_id: run_id.to_string(),
            started_at: "2026-06-08T00:00:31.000Z".to_string(),
            status: status.to_string(),
            stderr: String::new(),
            stdout: String::new(),
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
