#[cfg(test)]
mod tests {
    use crate::sqlite_store::{open_migrated_database, open_migrated_memory_database};
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn migration_creates_agent_run_tables() {
        let connection = open_migrated_memory_database().unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('agent_runs', 'agent_events', 'artifacts', 'task_threads', 'thread_messages', 'thread_run_records', 'action_proposals', 'approval_bridge_records', 'workspace_project_snapshots', 'model_role_routes', 'memory_candidates', 'memory_records', 'skill_manifests', 'automation_contracts', 'scheduled_runs', 'release_profiles', 'support_bundles', 'release_smoke_records', 'test_artifact_records', 'test_parsed_failures', 'test_exec_events', 'patch_proposal_records', 'patch_proposal_files', 'patch_diff_lines', 'review_report_records', 'review_findings', 'external_agent_run_records', 'external_agent_run_events', 'external_agent_run_tests', 'research_evidence_records')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 30);
    }

    #[test]
    fn migration_enforces_foreign_keys() {
        let connection = open_migrated_memory_database().unwrap();
        let result = connection.execute(
            "INSERT INTO agent_events (id, run_id, kind, message) VALUES ('event-1', 'missing-run', 'test', 'blocked')",
            [],
        );

        assert!(result.unwrap_err().to_string().contains("FOREIGN KEY"));
    }

    #[test]
    fn migration_upgrades_legacy_evidence_records() {
        let path = temp_path("legacy-evidence");
        let legacy = Connection::open(&path).unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE agent_runs (
                   id TEXT PRIMARY KEY NOT NULL,
                   thread_id TEXT NOT NULL,
                   status TEXT NOT NULL,
                   outcome_summary TEXT
                 );
                 CREATE TABLE evidence_records (
                   id TEXT NOT NULL,
                   run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
                   source_kind TEXT NOT NULL,
                   title TEXT NOT NULL,
                   PRIMARY KEY (run_id, id)
                 );",
            )
            .unwrap();
        drop(legacy);

        let connection = open_migrated_database(&path).unwrap();
        let agent_columns = table_columns(&connection, "agent_runs");
        let columns = table_columns(&connection, "evidence_records");

        assert!(agent_columns.contains(&"outcome_evidence_record_ids".to_string()));
        assert!(agent_columns.contains(&"outcome_test_artifact_ids".to_string()));
        assert!(columns.contains(&"source_id".to_string()));
        assert!(columns.contains(&"retrieved_at".to_string()));
        assert!(columns.contains(&"relevance_reason".to_string()));
        let _ = fs::remove_file(path);
    }

    fn table_columns(connection: &Connection, table: &str) -> Vec<String> {
        let mut statement = connection
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
