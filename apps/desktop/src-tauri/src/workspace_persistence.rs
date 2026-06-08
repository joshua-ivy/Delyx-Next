use crate::workspace_bridge::WorkspaceProjectView;
use rusqlite::{params, OptionalExtension};
use std::path::Path;

pub fn save_recent_project(path: &Path, project: &WorkspaceProjectView) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let project_json = serde_json::to_string(project).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO workspace_project_snapshots (id, path, project_json, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET path = excluded.path, project_json = excluded.project_json, updated_at = CURRENT_TIMESTAMP",
            params![project.id, project.path, project_json],
        )
        .map(|_| ())
        .map_err(sql_string)
}

pub fn load_recent_project(path: &Path) -> Result<Option<WorkspaceProjectView>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let project_json = connection
        .query_row(
            "SELECT project_json FROM workspace_project_snapshots ORDER BY updated_at DESC, rowid DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(sql_string)?;
    project_json
        .map(|value| serde_json::from_str(&value).map_err(|error| error.to_string()))
        .transpose()
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
