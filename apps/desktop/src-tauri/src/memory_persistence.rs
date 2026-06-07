use crate::memory::{MemoryCandidate, MemoryCandidateStatus, MemoryRecord, MemoryScope, MemoryStore};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(store: &MemoryStore, path: &Path) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    clear_tables(&connection)?;
    for candidate in store.candidates() {
        insert_candidate(&connection, candidate)?;
    }
    for record in store.records() {
        insert_record(&connection, record)?;
    }
    Ok(())
}

pub fn load_from_path(path: &Path) -> Result<MemoryStore, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let candidates = load_candidates(&connection)?;
    let records = load_records(&connection)?;
    Ok(MemoryStore::from_loaded(candidates, records))
}

fn clear_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM memory_records;
             DELETE FROM memory_candidates;",
        )
        .map_err(sql_string)
}

fn insert_candidate(connection: &Connection, candidate: &MemoryCandidate) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO memory_candidates
             (id, scope, key, value, source_run_id, source_thread_id, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                candidate.id,
                scope_key(candidate.scope),
                candidate.key,
                candidate.value,
                candidate.source_run_id,
                candidate.source_thread_id,
                status_key(candidate.status),
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_record(connection: &Connection, record: &MemoryRecord) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO memory_records
             (id, scope, key, value, source_run_id, source_thread_id, supersedes, suppressed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.id,
                scope_key(record.scope),
                record.key,
                record.value,
                record.source_run_id,
                record.source_thread_id,
                record.supersedes,
                record.suppressed as i64,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_candidates(connection: &Connection) -> Result<Vec<MemoryCandidate>, String> {
    let mut statement = connection
        .prepare("SELECT id, scope, key, value, source_run_id, source_thread_id, status FROM memory_candidates ORDER BY rowid")
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut candidates = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let scope: String = row.get(1).map_err(sql_string)?;
        let status: String = row.get(6).map_err(sql_string)?;
        candidates.push(MemoryCandidate {
            id: row.get(0).map_err(sql_string)?,
            scope: parse_scope(&scope)?,
            key: row.get(2).map_err(sql_string)?,
            value: row.get(3).map_err(sql_string)?,
            source_run_id: row.get(4).map_err(sql_string)?,
            source_thread_id: row.get(5).map_err(sql_string)?,
            status: parse_status(&status)?,
        });
    }
    Ok(candidates)
}

fn load_records(connection: &Connection) -> Result<Vec<MemoryRecord>, String> {
    let mut statement = connection
        .prepare("SELECT id, scope, key, value, source_run_id, source_thread_id, supersedes, suppressed FROM memory_records ORDER BY rowid")
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut records = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let scope: String = row.get(1).map_err(sql_string)?;
        records.push(MemoryRecord {
            id: row.get(0).map_err(sql_string)?,
            scope: parse_scope(&scope)?,
            key: row.get(2).map_err(sql_string)?,
            value: row.get(3).map_err(sql_string)?,
            source_run_id: row.get(4).map_err(sql_string)?,
            source_thread_id: row.get(5).map_err(sql_string)?,
            supersedes: row.get(6).map_err(sql_string)?,
            suppressed: row.get::<_, i64>(7).map_err(sql_string)? != 0,
        });
    }
    Ok(records)
}

fn scope_key(scope: MemoryScope) -> &'static str {
    match scope {
        MemoryScope::Project => "project",
        MemoryScope::User => "user",
    }
}

fn parse_scope(value: &str) -> Result<MemoryScope, String> {
    match value {
        "project" => Ok(MemoryScope::Project),
        "user" => Ok(MemoryScope::User),
        _ => Err("Unsupported persisted memory scope.".to_string()),
    }
}

fn status_key(status: MemoryCandidateStatus) -> &'static str {
    match status {
        MemoryCandidateStatus::Pending => "pending",
        MemoryCandidateStatus::Promoted => "promoted",
        MemoryCandidateStatus::Suppressed => "suppressed",
    }
}

fn parse_status(value: &str) -> Result<MemoryCandidateStatus, String> {
    match value {
        "pending" => Ok(MemoryCandidateStatus::Pending),
        "promoted" => Ok(MemoryCandidateStatus::Promoted),
        "suppressed" => Ok(MemoryCandidateStatus::Suppressed),
        _ => Err("Unsupported persisted memory status.".to_string()),
    }
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
