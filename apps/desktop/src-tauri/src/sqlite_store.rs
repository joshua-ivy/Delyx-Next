use rusqlite::Connection;
use std::path::{Path, PathBuf};

const AGENT_RUN_MIGRATION: &str = include_str!("../migrations/0001_agent_run_ledger.sql");

pub fn default_database_path() -> PathBuf {
    if let Some(path) = std::env::var_os("DELYX_NEXT_DB_PATH") {
        return PathBuf::from(path);
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data)
            .join("Delyx Next")
            .join("delyx-next.sqlite3");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("delyx-next.sqlite3")
}

pub fn open_migrated_database(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
    }
    let connection = Connection::open(path)?;
    migrate(&connection)?;
    Ok(connection)
}

pub fn open_migrated_memory_database() -> rusqlite::Result<Connection> {
    let connection = Connection::open_in_memory()?;
    migrate(&connection)?;
    Ok(connection)
}

fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.execute_batch(AGENT_RUN_MIGRATION)?;
    ensure_agent_run_columns(connection)?;
    ensure_evidence_columns(connection)?;
    ensure_patch_file_columns(connection)?;
    Ok(())
}

fn ensure_agent_run_columns(connection: &Connection) -> rusqlite::Result<()> {
    let columns = table_columns(connection, "agent_runs")?;
    for (name, definition) in [
        ("outcome_evidence_record_ids", "TEXT NOT NULL DEFAULT '[]'"),
        ("outcome_test_artifact_ids", "TEXT NOT NULL DEFAULT '[]'"),
    ] {
        if !columns.iter().any(|column| column == name) {
            connection.execute(
                &format!("ALTER TABLE agent_runs ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    Ok(())
}

fn ensure_evidence_columns(connection: &Connection) -> rusqlite::Result<()> {
    let columns = table_columns(connection, "evidence_records")?;
    for (name, definition) in [
        ("source_id", "TEXT NOT NULL DEFAULT ''"),
        ("uri", "TEXT"),
        ("quote", "TEXT"),
        ("hash", "TEXT"),
        ("retrieved_at", "TEXT NOT NULL DEFAULT ''"),
        ("relevance_relationship", "TEXT"),
        ("relevance_score", "INTEGER"),
        ("relevance_reason", "TEXT"),
    ] {
        if !columns.iter().any(|column| column == name) {
            connection.execute(
                &format!("ALTER TABLE evidence_records ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    Ok(())
}

fn ensure_patch_file_columns(connection: &Connection) -> rusqlite::Result<()> {
    let columns = table_columns(connection, "patch_proposal_files")?;
    for (name, definition) in [
        ("before_text", "TEXT NOT NULL DEFAULT ''"),
        ("after_text", "TEXT NOT NULL DEFAULT ''"),
    ] {
        if !columns.iter().any(|column| column == name) {
            connection.execute(
                &format!("ALTER TABLE patch_proposal_files ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    Ok(())
}

fn table_columns(connection: &Connection, table: &str) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    rows.collect()
}
