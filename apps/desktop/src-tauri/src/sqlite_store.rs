use rusqlite::Connection;
use std::path::Path;

const AGENT_RUN_MIGRATION: &str = include_str!("../migrations/0001_agent_run_ledger.sql");

pub fn open_migrated_database(path: &Path) -> rusqlite::Result<Connection> {
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
