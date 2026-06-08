use rusqlite::{params, Connection};
use std::path::Path;

use crate::agent_run::{
    AgentEvent, AgentNode, AgentOutcome, AgentRun, AgentRunError, AgentRunLedger, AgentRunStatus,
    Artifact, RunMetrics,
};

pub fn save_to_path(ledger: &AgentRunLedger, path: &Path) -> Result<(), AgentRunError> {
    let mut connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_error)?;
    let transaction = connection.transaction().map_err(sql_error)?;
    save_to_connection(ledger, &transaction)?;
    transaction.commit().map_err(sql_error)
}

pub fn load_from_path(path: &Path) -> Result<AgentRunLedger, AgentRunError> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_error)?;
    load_from_connection(&connection)
}

pub fn save_to_connection(
    ledger: &AgentRunLedger,
    connection: &Connection,
) -> Result<(), AgentRunError> {
    clear_tables(connection)?;
    for run in &ledger.runs {
        insert_run(connection, run)?;
        for node in &run.nodes {
            insert_node(connection, &run.id, node)?;
        }
        for event in &run.events {
            insert_event(connection, &run.id, event)?;
        }
        for artifact in &run.artifacts {
            insert_artifact(connection, &run.id, artifact)?;
        }
        for evidence in &run.evidence {
            crate::agent_run_evidence_persistence::insert_evidence(connection, &run.id, evidence)?;
        }
    }
    Ok(())
}

pub fn load_from_connection(connection: &Connection) -> Result<AgentRunLedger, AgentRunError> {
    let mut ledger = AgentRunLedger::new();
    load_runs(connection, &mut ledger)?;
    load_nodes(connection, &mut ledger)?;
    load_events(connection, &mut ledger)?;
    load_artifacts(connection, &mut ledger)?;
    crate::agent_run_evidence_persistence::load_evidence(connection, &mut ledger)?;
    ledger.refresh_loaded_counters();
    Ok(ledger)
}

fn clear_tables(connection: &Connection) -> Result<(), AgentRunError> {
    connection
        .execute_batch(
            "DELETE FROM evidence_records;
             DELETE FROM artifacts;
             DELETE FROM agent_events;
             DELETE FROM agent_nodes;
             DELETE FROM agent_runs;",
        )
        .map_err(sql_error)
}

fn insert_run(connection: &Connection, run: &AgentRun) -> Result<(), AgentRunError> {
    let outcome = run.outcome.as_ref().map(|value| value.summary.as_str());
    let evidence_ids = outcome_list(run.outcome.as_ref().map(|value| &value.evidence_record_ids))?;
    let test_ids = outcome_list(run.outcome.as_ref().map(|value| &value.test_artifact_ids))?;
    connection
        .execute(
            "INSERT INTO agent_runs
             (id, thread_id, status, outcome_summary, outcome_evidence_record_ids, outcome_test_artifact_ids)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![run.id, run.thread_id, status_key(run.status), outcome, evidence_ids, test_ids],
        )
        .map(|_| ())
        .map_err(sql_error)
}

fn insert_node(
    connection: &Connection,
    run_id: &str,
    node: &AgentNode,
) -> Result<(), AgentRunError> {
    connection
        .execute(
            "INSERT INTO agent_nodes (id, run_id, kind, label, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                node.id,
                run_id,
                node.kind,
                node.label,
                status_key(node.status)
            ],
        )
        .map(|_| ())
        .map_err(sql_error)
}

fn insert_event(
    connection: &Connection,
    run_id: &str,
    event: &AgentEvent,
) -> Result<(), AgentRunError> {
    connection
        .execute(
            "INSERT INTO agent_events (id, run_id, kind, message) VALUES (?1, ?2, ?3, ?4)",
            params![event.id, run_id, event.kind, event.message],
        )
        .map(|_| ())
        .map_err(sql_error)
}

fn insert_artifact(
    connection: &Connection,
    run_id: &str,
    artifact: &Artifact,
) -> Result<(), AgentRunError> {
    connection
        .execute(
            "INSERT INTO artifacts (id, run_id, kind, label) VALUES (?1, ?2, ?3, ?4)",
            params![artifact.id, run_id, artifact.kind, artifact.label],
        )
        .map(|_| ())
        .map_err(sql_error)
}

fn load_runs(connection: &Connection, ledger: &mut AgentRunLedger) -> Result<(), AgentRunError> {
    let mut statement = connection
        .prepare(
            "SELECT id, thread_id, status, outcome_summary,
                    outcome_evidence_record_ids, outcome_test_artifact_ids
             FROM agent_runs ORDER BY rowid",
        )
        .map_err(sql_error)?;
    let mut rows = statement.query([]).map_err(sql_error)?;
    while let Some(row) = rows.next().map_err(sql_error)? {
        let status_value: String = row.get(2).map_err(sql_error)?;
        let status = parse_status(&status_value)?;
        let summary: Option<String> = row.get(3).map_err(sql_error)?;
        let evidence_ids: String = row.get(4).map_err(sql_error)?;
        let test_ids: String = row.get(5).map_err(sql_error)?;
        let outcome = if let Some(value) = summary {
            Some(AgentOutcome {
                evidence_record_ids: parse_list(&evidence_ids)?,
                status,
                summary: value,
                test_artifact_ids: parse_list(&test_ids)?,
            })
        } else {
            None
        };
        ledger.runs.push(AgentRun {
            id: row.get(0).map_err(sql_error)?,
            thread_id: row.get(1).map_err(sql_error)?,
            status,
            nodes: Vec::new(),
            events: Vec::new(),
            artifacts: Vec::new(),
            evidence: Vec::new(),
            metrics: RunMetrics::default(),
            outcome,
        });
    }
    Ok(())
}

fn load_nodes(connection: &Connection, ledger: &mut AgentRunLedger) -> Result<(), AgentRunError> {
    let mut statement = connection
        .prepare("SELECT run_id, id, kind, label, status FROM agent_nodes ORDER BY rowid")
        .map_err(sql_error)?;
    let mut rows = statement.query([]).map_err(sql_error)?;
    while let Some(row) = rows.next().map_err(sql_error)? {
        let status_value: String = row.get(4).map_err(sql_error)?;
        let status = parse_status(&status_value)?;
        let run_id: String = row.get(0).map_err(sql_error)?;
        let node = AgentNode {
            id: row.get(1).map_err(sql_error)?,
            kind: row.get(2).map_err(sql_error)?,
            label: row.get(3).map_err(sql_error)?,
            status,
        };
        ledger.run_mut(&run_id)?.nodes.push(node);
    }
    Ok(())
}

fn load_events(connection: &Connection, ledger: &mut AgentRunLedger) -> Result<(), AgentRunError> {
    let mut statement = connection
        .prepare("SELECT run_id, id, kind, message FROM agent_events ORDER BY rowid")
        .map_err(sql_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                AgentEvent {
                    id: row.get(1)?,
                    kind: row.get(2)?,
                    message: row.get(3)?,
                },
            ))
        })
        .map_err(sql_error)?;
    for row in rows {
        let (run_id, event) = row.map_err(sql_error)?;
        let run = ledger.run_mut(&run_id)?;
        run.events.push(event);
        run.metrics.event_count = run.events.len();
    }
    Ok(())
}

fn load_artifacts(
    connection: &Connection,
    ledger: &mut AgentRunLedger,
) -> Result<(), AgentRunError> {
    let mut statement = connection
        .prepare("SELECT run_id, id, kind, label FROM artifacts ORDER BY rowid")
        .map_err(sql_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                Artifact {
                    id: row.get(1)?,
                    kind: row.get(2)?,
                    label: row.get(3)?,
                },
            ))
        })
        .map_err(sql_error)?;
    for row in rows {
        let (run_id, artifact) = row.map_err(sql_error)?;
        let run = ledger.run_mut(&run_id)?;
        run.artifacts.push(artifact);
        run.metrics.artifact_count = run.artifacts.len();
    }
    Ok(())
}

fn status_key(status: AgentRunStatus) -> &'static str {
    match status {
        AgentRunStatus::Running => "running",
        AgentRunStatus::WaitingForApproval => "waiting_for_approval",
        AgentRunStatus::Completed => "completed",
        AgentRunStatus::Failed => "failed",
    }
}

fn parse_status(value: &str) -> Result<AgentRunStatus, AgentRunError> {
    match value {
        "running" => Ok(AgentRunStatus::Running),
        "waiting_for_approval" => Ok(AgentRunStatus::WaitingForApproval),
        "completed" => Ok(AgentRunStatus::Completed),
        "failed" => Ok(AgentRunStatus::Failed),
        _ => Err(AgentRunError::InvalidLedger(value.to_string())),
    }
}

fn outcome_list(value: Option<&Vec<String>>) -> Result<String, AgentRunError> {
    match value {
        Some(items) => serde_json::to_string(items)
            .map_err(|error| AgentRunError::InvalidLedger(error.to_string())),
        None => Ok("[]".to_string()),
    }
}

fn parse_list(value: &str) -> Result<Vec<String>, AgentRunError> {
    serde_json::from_str(value).map_err(|error| AgentRunError::InvalidLedger(error.to_string()))
}

fn sql_error(error: rusqlite::Error) -> AgentRunError {
    AgentRunError::Io(error.to_string())
}
