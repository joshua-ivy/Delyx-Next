#[cfg(test)]
mod tests {
    use crate::thread_run_bridge::{
        create_thread_run_record, thread_run_snapshot_from_store, ThreadRunCreateRequest, ThreadRunStore,
    };

    #[test]
    fn create_thread_run_returns_ui_ready_record_without_fake_execution() {
        let mut store = ThreadRunStore::default();
        let record = create_thread_run_record(&mut store, request("proj-1", "Use Ollama locally")).unwrap();

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
    fn empty_goal_is_rejected_before_recording_run() {
        let mut store = ThreadRunStore::default();
        let result = create_thread_run_record(&mut store, request("proj-1", "   "));

        assert!(result.is_err());
        assert!(thread_run_snapshot_from_store(&store, "proj-1").threads.is_empty());
    }

    fn request(project_id: &str, goal: &str) -> ThreadRunCreateRequest {
        ThreadRunCreateRequest {
            created_at: "2026-06-07T00:00:00.000Z".to_string(),
            goal: goal.to_string(),
            project_id: project_id.to_string(),
        }
    }
}
