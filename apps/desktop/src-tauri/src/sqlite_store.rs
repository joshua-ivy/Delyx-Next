use rusqlite::Connection;
use std::path::{Path, PathBuf};

const AGENT_RUN_MIGRATION: &str = include_str!("../migrations/0001_agent_run_ledger.sql");

pub fn default_database_path() -> PathBuf {
    if let Some(path) = std::env::var_os("DELYX_NEXT_DB_PATH") {
        return PathBuf::from(path);
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("Delyx Next").join("delyx-next.sqlite3");
    }
    std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir()).join("delyx-next.sqlite3")
}

pub fn open_migrated_database(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
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
    Ok(())
}
