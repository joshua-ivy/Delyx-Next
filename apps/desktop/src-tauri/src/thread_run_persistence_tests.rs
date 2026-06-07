#[cfg(test)]
mod tests {
    use crate::thread_run_bridge::{
        append_thread_message_record, create_thread_run_record, thread_run_snapshot_from_store,
        update_thread_status_record, ThreadMessageAppendRequest, ThreadRunCreateRequest,
        ThreadStatusUpdateRequest,
    };
    use crate::thread_run_persistence::{load_from_path, save_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn thread_run_store_survives_sqlite_reload() {
        let path = temp_path("thread-run");
        let mut store = crate::thread_run_bridge::ThreadRunStore::default();
        let record = create_thread_run_record(&mut store, create_request("proj-1", "Persist this thread")).unwrap();
        update_thread_status_record(&mut store, status_request(&record.thread.id, "exploring")).unwrap();
        append_thread_message_record(
            &mut store,
            message_request(&record.thread.id, "assistant", "Reloadable local answer.", Some("idle")),
        )
        .unwrap();

        save_to_path(&store, &path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let mut loaded = load_from_path(&path).unwrap();
        let snapshot = thread_run_snapshot_from_store(&loaded, "proj-1");
        assert_eq!(snapshot.threads[0].messages[1].body, "Reloadable local answer.");
        assert_eq!(snapshot.runs[0].events[0].kind, "thread.created");

        let next = create_thread_run_record(&mut loaded, create_request("proj-1", "Next thread")).unwrap();
        assert_eq!(next.thread.id, "proj-1-thread-2");
        assert_eq!(next.run.id, "run-2");
        let _ = fs::remove_file(path);
    }

    fn create_request(project_id: &str, goal: &str) -> ThreadRunCreateRequest {
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
            updated_at: "2026-06-07T00:02:00.000Z".to_string(),
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
