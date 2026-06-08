use crate::external_agent_run_bridge::{
    ExternalAgentEventView, ExternalAgentRunArtifactView, ExternalAgentRunBridgeStore,
};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &ExternalAgentRunBridgeStore, path: &Path) -> Result<(), String> {
    let mut connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let transaction = connection.transaction().map_err(sql_string)?;
    clear_tables(&transaction)?;
    for artifact in &store.artifacts {
        insert_artifact(&transaction, artifact)?;
        for (index, event) in artifact.transcript.iter().enumerate() {
            insert_event(&transaction, &artifact.id, index, event)?;
        }
        for (index, test_id) in artifact.test_artifact_ids.iter().enumerate() {
            insert_test_link(&transaction, &artifact.id, index, test_id)?;
        }
    }
    transaction.commit().map_err(sql_string)
}

pub fn load_from_path(path: &Path) -> Result<ExternalAgentRunBridgeStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut artifacts = load_artifacts(&connection)?;
    for artifact in &mut artifacts {
        artifact.transcript = load_events(&connection, &artifact.id)?;
        artifact.test_artifact_ids = load_test_links(&connection, &artifact.id)?;
    }
    Ok(ExternalAgentRunBridgeStore {
        next_id: next_artifact_id(&artifacts),
        artifacts,
    })
}

fn clear_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM external_agent_run_tests;
             DELETE FROM external_agent_run_events;
             DELETE FROM external_agent_run_records;",
        )
        .map_err(sql_string)
}

fn insert_artifact(
    connection: &Connection,
    artifact: &ExternalAgentRunArtifactView,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO external_agent_run_records
             (id, run_id, adapter_id, status, scope, terminal_output, diff_summary, review_required)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                artifact.id,
                artifact.run_id,
                artifact.adapter_id,
                artifact.status,
                artifact.scope,
                artifact.terminal_output,
                artifact.diff_summary,
                artifact.review_required as i32,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_event(
    connection: &Connection,
    artifact_id: &str,
    index: usize,
    event: &ExternalAgentEventView,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO external_agent_run_events
             (artifact_id, event_index, kind, message, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                artifact_id,
                index as i64,
                event.kind,
                event.message,
                event.timestamp
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_test_link(
    connection: &Connection,
    artifact_id: &str,
    index: usize,
    test_id: &str,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO external_agent_run_tests
             (artifact_id, test_index, test_artifact_id)
             VALUES (?1, ?2, ?3)",
            params![artifact_id, index as i64, test_id],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_artifacts(connection: &Connection) -> Result<Vec<ExternalAgentRunArtifactView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, run_id, adapter_id, status, scope, terminal_output, diff_summary,
                    review_required
             FROM external_agent_run_records ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map([], |row| {
            Ok(ExternalAgentRunArtifactView {
                id: row.get(0)?,
                run_id: row.get(1)?,
                adapter_id: row.get(2)?,
                status: row.get(3)?,
                scope: row.get(4)?,
                terminal_output: row.get(5)?,
                diff_summary: row.get(6)?,
                review_required: row.get::<_, i64>(7)? != 0,
                transcript: Vec::new(),
                test_artifact_ids: Vec::new(),
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn load_events(
    connection: &Connection,
    artifact_id: &str,
) -> Result<Vec<ExternalAgentEventView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT kind, message, timestamp FROM external_agent_run_events
             WHERE artifact_id = ?1 ORDER BY event_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![artifact_id], |row| {
            Ok(ExternalAgentEventView {
                kind: row.get(0)?,
                message: row.get(1)?,
                timestamp: row.get(2)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn load_test_links(connection: &Connection, artifact_id: &str) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT test_artifact_id FROM external_agent_run_tests
             WHERE artifact_id = ?1 ORDER BY test_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![artifact_id], |row| row.get(0))
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn next_artifact_id(artifacts: &[ExternalAgentRunArtifactView]) -> usize {
    artifacts
        .iter()
        .filter_map(|artifact| {
            artifact
                .id
                .strip_prefix("external-agent-run-")?
                .parse()
                .ok()
        })
        .max()
        .unwrap_or(artifacts.len())
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
