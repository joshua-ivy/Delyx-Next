use crate::test_runner_bridge::{
    CommandExecEventView, ParsedFailureView, TestArtifactView, TestRunnerBridgeStore,
};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &TestRunnerBridgeStore, path: &Path) -> Result<(), String> {
    let mut connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let transaction = connection.transaction().map_err(sql_string)?;
    clear_tables(&transaction)?;
    for artifact in &store.artifacts {
        insert_artifact(&transaction, artifact)?;
        for (index, failure) in artifact.parsed_failures.iter().flatten().enumerate() {
            insert_failure(&transaction, &artifact.id, index, failure)?;
        }
        for (index, event) in artifact.exec_events.iter().enumerate() {
            insert_exec_event(&transaction, &artifact.id, index, event)?;
        }
    }
    transaction.commit().map_err(sql_string)
}

pub fn load_from_path(path: &Path) -> Result<TestRunnerBridgeStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut artifacts = load_artifacts(&connection)?;
    for artifact in &mut artifacts {
        artifact.parsed_failures = load_failures(&connection, &artifact.id)?;
        artifact.exec_events = load_exec_events(&connection, &artifact.id)?;
    }
    Ok(TestRunnerBridgeStore { artifacts })
}

fn clear_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM test_exec_events;
             DELETE FROM test_parsed_failures;
             DELETE FROM test_artifact_records;",
        )
        .map_err(sql_string)
}

fn insert_artifact(connection: &Connection, artifact: &TestArtifactView) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO test_artifact_records
             (id, run_id, command, cwd, exit_code, duration_ms, stdout, stderr, started_at,
              completed_at, approval_id, status, failure_summary, output_truncated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                artifact.id,
                artifact.run_id,
                artifact.command,
                artifact.cwd,
                artifact.exit_code,
                artifact.duration_ms as i64,
                artifact.stdout,
                artifact.stderr,
                artifact.started_at,
                artifact.completed_at,
                artifact.approval_id,
                artifact.status,
                artifact.failure_summary,
                artifact.output_truncated as i32,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_failure(
    connection: &Connection,
    artifact_id: &str,
    index: usize,
    failure: &ParsedFailureView,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO test_parsed_failures (artifact_id, failure_index, id, message)
             VALUES (?1, ?2, ?3, ?4)",
            params![artifact_id, index as i64, failure.id, failure.message],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_exec_event(
    connection: &Connection,
    artifact_id: &str,
    index: usize,
    event: &CommandExecEventView,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO test_exec_events (artifact_id, event_index, kind, message, timestamp_ms)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                artifact_id,
                index as i64,
                event.kind,
                event.message,
                event.timestamp_ms as i64,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_artifacts(connection: &Connection) -> Result<Vec<TestArtifactView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, run_id, command, cwd, exit_code, duration_ms, stdout, stderr, started_at,
                    completed_at, approval_id, status, failure_summary, output_truncated
             FROM test_artifact_records ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map([], |row| {
            Ok(TestArtifactView {
                id: row.get(0)?,
                run_id: row.get(1)?,
                command: row.get(2)?,
                cwd: row.get(3)?,
                exit_code: row.get(4)?,
                duration_ms: row.get::<_, i64>(5)? as u64,
                stdout: row.get(6)?,
                stderr: row.get(7)?,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
                approval_id: row.get(10)?,
                status: row.get(11)?,
                failure_summary: row.get(12)?,
                output_truncated: row.get::<_, i64>(13)? != 0,
                parsed_failures: None,
                exec_events: Vec::new(),
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn load_failures(
    connection: &Connection,
    artifact_id: &str,
) -> Result<Option<Vec<ParsedFailureView>>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, message FROM test_parsed_failures
             WHERE artifact_id = ?1 ORDER BY failure_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![artifact_id], |row| {
            Ok(ParsedFailureView {
                id: row.get(0)?,
                message: row.get(1)?,
            })
        })
        .map_err(sql_string)?;
    let failures = rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)?;
    Ok((!failures.is_empty()).then_some(failures))
}

fn load_exec_events(
    connection: &Connection,
    artifact_id: &str,
) -> Result<Vec<CommandExecEventView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT kind, message, timestamp_ms FROM test_exec_events
             WHERE artifact_id = ?1 ORDER BY event_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![artifact_id], |row| {
            Ok(CommandExecEventView {
                kind: row.get(0)?,
                message: row.get(1)?,
                timestamp_ms: row.get::<_, i64>(2)? as u64,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
