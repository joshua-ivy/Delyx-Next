#[cfg(test)]
mod tests {
    use crate::agent_run::{EvidenceRecordInput, EvidenceRelevance};
    use crate::thread_run_bridge::{
        append_thread_message_record, archive_thread_record, create_thread_run_record,
        thread_run_snapshot_from_store, update_thread_status_record, ThreadArchiveRequest,
        ThreadMessageAppendRequest, ThreadRunCreateRequest, ThreadRunStore,
        ThreadStatusUpdateRequest,
    };

    #[test]
    fn create_thread_run_returns_ui_ready_record_without_fake_execution() {
        let mut store = ThreadRunStore::default();
        let record =
            create_thread_run_record(&mut store, request("proj-1", "Use Ollama locally")).unwrap();

        assert_eq!(record.thread.project_id, "proj-1");
        assert_eq!(record.thread.status, "idle");
        assert_eq!(record.thread.mode, "explore");
        assert_eq!(record.thread.active_run_id, Some(record.run.id.clone()));
        assert_eq!(record.run.status, "created");
        assert_eq!(record.run.metrics.event_count, 1);
        assert_eq!(record.run.events[0].kind, "thread.created");
    }

    #[test]
    fn snapshot_lists_only_requested_project_threads() {
        let mut store = ThreadRunStore::default();
        create_thread_run_record(&mut store, request("proj-1", "One")).unwrap();
        create_thread_run_record(&mut store, request("proj-2", "Two")).unwrap();

        let snapshot = thread_run_snapshot_from_store(&store, "proj-1");

        assert_eq!(snapshot.threads.len(), 1);
        assert_eq!(snapshot.runs.len(), 1);
        assert_eq!(snapshot.threads[0].goal, "One");
        assert_eq!(snapshot.runs[0].thread_id, snapshot.threads[0].id);
    }

    #[test]
    fn snapshot_exposes_run_evidence_records() {
        let mut store = ThreadRunStore::default();
        let record =
            create_thread_run_record(&mut store, request("proj-1", "Use evidence")).unwrap();
        store
            .ledger
            .record_evidence_detail(&record.run.id, evidence_input())
            .unwrap();
        store
            .ledger
            .complete_run_with_support(
                &record.run.id,
                "Supported final answer.",
                vec!["evidence-1".to_string()],
                vec!["test-artifact-1".to_string()],
            )
            .unwrap();

        let snapshot = thread_run_snapshot_from_store(&store, "proj-1");
        let evidence = &snapshot.runs[0].evidence[0];

        assert_eq!(evidence["runId"], record.run.id);
        assert_eq!(evidence["sourceKind"], "local_file");
        assert_eq!(evidence["sourceId"], "file://README.md");
        assert_eq!(evidence["quote"], "Delyx Next is local-first.");
        assert_eq!(evidence["relevance"]["relationship"], "doc");
        let outcome = snapshot.runs[0].outcome.as_ref().unwrap();
        assert_eq!(outcome["status"], "succeeded");
        assert_eq!(outcome["evidenceRecordIds"][0], "evidence-1");
        assert_eq!(outcome["testArtifactIds"][0], "test-artifact-1");
    }

    #[test]
    fn empty_goal_is_rejected_before_recording_run() {
        let mut store = ThreadRunStore::default();
        let result = create_thread_run_record(&mut store, request("proj-1", "   "));

        assert!(result.is_err());
        assert!(thread_run_snapshot_from_store(&store, "proj-1")
            .threads
            .is_empty());
    }

    #[test]
    fn status_update_changes_thread_and_snapshot_run_status() {
        let mut store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut store, request("proj-1", "Plan")).unwrap();

        let updated =
            update_thread_status_record(&mut store, status_request(&record.thread.id, "planning"))
                .unwrap();
        let snapshot = thread_run_snapshot_from_store(&store, "proj-1");

        assert_eq!(updated.status, "planning");
        assert_eq!(updated.mode, "plan");
        assert_eq!(snapshot.runs[0].status, "running");
    }

    #[test]
    fn archived_threads_are_hidden_from_active_snapshot() {
        let mut store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut store, request("proj-1", "Archive me")).unwrap();

        let archived =
            archive_thread_record(&mut store, archive_request(&record.thread.id)).unwrap();
        let snapshot = thread_run_snapshot_from_store(&store, "proj-1");

        assert!(archived.archived);
        assert!(snapshot.threads.is_empty());
        assert!(snapshot.runs.is_empty());
    }

    #[test]
    fn message_append_records_ollama_reply_and_settles_status() {
        let mut store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut store, request("proj-1", "Ask Ollama")).unwrap();

        update_thread_status_record(&mut store, status_request(&record.thread.id, "exploring"))
            .unwrap();
        let updated = append_thread_message_record(
            &mut store,
            message_request(
                &record.thread.id,
                "assistant",
                "Real local reply.",
                Some("idle"),
            ),
        )
        .unwrap();
        let snapshot = thread_run_snapshot_from_store(&store, "proj-1");

        assert_eq!(updated.status, "idle");
        assert_eq!(updated.messages.len(), 2);
        assert_eq!(updated.messages[1].role, "assistant");
        assert_eq!(snapshot.threads[0].messages[1].body, "Real local reply.");
    }

    #[test]
    fn message_append_rejects_unknown_roles_without_changing_thread() {
        let mut store = ThreadRunStore::default();
        let record =
            create_thread_run_record(&mut store, request("proj-1", "Keep roles typed")).unwrap();

        let result = append_thread_message_record(
            &mut store,
            message_request(&record.thread.id, "tool", "Nope", None),
        );
        let snapshot = thread_run_snapshot_from_store(&store, "proj-1");

        assert!(result.is_err());
        assert_eq!(snapshot.threads[0].messages.len(), 1);
    }

    fn request(project_id: &str, goal: &str) -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-07T00:00:00.000Z".to_string(),
            goal: goal.to_string(),
            project_id: project_id.to_string(),
        }
    }

    fn status_request(thread_id: &str, status: &str) -> ThreadStatusUpdateRequest {
        ThreadStatusUpdateRequest {
            status: status.to_string(),
            thread_id: thread_id.to_string(),
            updated_at: "2026-06-07T00:01:00.000Z".to_string(),
        }
    }

    fn archive_request(thread_id: &str) -> ThreadArchiveRequest {
        ThreadArchiveRequest {
            thread_id: thread_id.to_string(),
            updated_at: "2026-06-07T00:02:00.000Z".to_string(),
        }
    }

    fn message_request(
        thread_id: &str,
        role: &str,
        body: &str,
        status: Option<&str>,
    ) -> ThreadMessageAppendRequest {
        ThreadMessageAppendRequest {
            body: body.to_string(),
            role: role.to_string(),
            status: status.map(str::to_string),
            thread_id: thread_id.to_string(),
            updated_at: "2026-06-07T00:03:00.000Z".to_string(),
        }
    }

    fn evidence_input() -> EvidenceRecordInput {
        EvidenceRecordInput {
            hash: Some("sha256:readme".to_string()),
            quote: Some("Delyx Next is local-first.".to_string()),
            relevance: Some(EvidenceRelevance {
                reason: "README supports the local-first claim.".to_string(),
                relationship: "doc".to_string(),
                score: 95,
            }),
            retrieved_at: "2026-06-07T00:04:00.000Z".to_string(),
            source_id: "file://README.md".to_string(),
            source_kind: "local_file".to_string(),
            title: "README.md".to_string(),
            uri: Some("file:///README.md".to_string()),
        }
    }
}
