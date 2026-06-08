#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::external_agent_run_bridge::{
        external_agent_run_snapshot_from_store, ExternalAgentEventView,
        ExternalAgentRunArtifactView, ExternalAgentRunBridgeStore,
    };
    use crate::external_agent_run_persistence::{load_from_path, save_to_path};

    #[test]
    fn external_agent_run_artifacts_survive_sqlite_reload() {
        let path = temp_db_path("external-agent-run-reload");
        let expected = artifact("external-agent-run-7", "run-1");
        let store = ExternalAgentRunBridgeStore {
            artifacts: vec![expected.clone(), artifact("external-agent-run-3", "run-2")],
            next_id: 7,
        };

        save_to_path(&store, &path).unwrap();
        let loaded = load_from_path(&path).unwrap();
        let snapshot = external_agent_run_snapshot_from_store(&loaded, "run-1");

        assert_eq!(snapshot, vec![expected]);
        assert_eq!(snapshot[0].transcript[0].kind, "started");
        assert_eq!(snapshot[0].transcript[1].message, "codex jsonl");
        assert_eq!(
            snapshot[0].test_artifact_ids,
            vec!["test-artifact-1", "test-artifact-2"]
        );
        assert!(snapshot[0].review_required);
        assert_eq!(loaded.next_id, 7);

        std::fs::remove_file(path).ok();
    }

    fn artifact(id: &str, run_id: &str) -> ExternalAgentRunArtifactView {
        ExternalAgentRunArtifactView {
            adapter_id: "codex-cli".to_string(),
            diff_summary: Some("1 file changed; Delyx review required".to_string()),
            id: id.to_string(),
            review_required: true,
            run_id: run_id.to_string(),
            scope: "root: C:/workspace; isolation: checkpoint checkpoint-1".to_string(),
            status: "completed".to_string(),
            terminal_output: "codex jsonl\n".to_string(),
            test_artifact_ids: vec!["test-artifact-1".to_string(), "test-artifact-2".to_string()],
            transcript: vec![
                event("started", "Codex CLI started.", "10"),
                event("stdout", "codex jsonl", "11"),
            ],
        }
    }

    fn event(kind: &str, message: &str, timestamp: &str) -> ExternalAgentEventView {
        ExternalAgentEventView {
            kind: kind.to_string(),
            message: message.to_string(),
            timestamp: timestamp.to_string(),
        }
    }

    fn temp_db_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{label}-{nanos}.sqlite"))
    }
}
