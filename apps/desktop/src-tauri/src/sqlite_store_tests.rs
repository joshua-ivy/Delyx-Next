#[cfg(test)]
mod tests {
    use crate::sqlite_store::open_migrated_memory_database;

    #[test]
    fn migration_creates_agent_run_tables() {
        let connection = open_migrated_memory_database().unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('agent_runs', 'agent_events', 'artifacts', 'task_threads', 'thread_messages', 'thread_run_records', 'action_proposals', 'approval_bridge_records', 'workspace_project_snapshots', 'model_role_routes', 'memory_candidates', 'memory_records', 'skill_manifests', 'automation_contracts', 'scheduled_runs')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 15);
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
}
