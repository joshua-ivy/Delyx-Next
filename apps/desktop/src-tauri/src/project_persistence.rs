//! SQLite persistence for native project records and their file scopes.
//!
//! Projects live in normalized tables (`projects` + `project_file_scopes`) so
//! scopes and policy are first-class, not an opaque JSON blob.

use crate::project::{FileScopeRecord, ProjectRecord, ProjectTrustLevel};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub fn save_project_to_path(path: &Path, project: &ProjectRecord) -> Result<ProjectRecord, String> {
    project.validate()?;
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    upsert_project(&connection, project)?;
    load_project(&connection, &project.id)?
        .ok_or_else(|| "Project disappeared immediately after saving.".to_string())
}

pub fn load_project_from_path(path: &Path, id: &str) -> Result<Option<ProjectRecord>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    load_project(&connection, id)
}

/// Load the project for `root_path`, creating one with safe defaults if none
/// exists yet. Never clobbers an existing project's scopes or policy.
pub fn ensure_project_to_path(
    path: &Path,
    name: &str,
    root_path: &str,
) -> Result<ProjectRecord, String> {
    let id = crate::project::stable_project_id(root_path);
    if let Some(existing) = load_project_from_path(path, &id)? {
        return Ok(existing);
    }
    save_project_to_path(path, &ProjectRecord::new(name, root_path))
}

pub fn list_projects_from_path(path: &Path) -> Result<Vec<ProjectRecord>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    list_projects(&connection)
}

pub fn delete_project_from_path(path: &Path, id: &str) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute("DELETE FROM projects WHERE id = ?1", params![id.trim()])
        .map(|_| ())
        .map_err(sql_string)
}

pub(crate) fn upsert_project(
    connection: &Connection,
    project: &ProjectRecord,
) -> Result<(), String> {
    let approval = json(&project.approval_policy)?;
    let models = json(&project.model_permissions)?;
    let tools = json(&project.tool_permissions)?;
    let memory = json(&project.memory_scope)?;
    connection
        .execute(
            "INSERT INTO projects (
                id, name, root_path, trust_level, approval_policy_json,
                model_permissions_json, tool_permissions_json, memory_scope_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                root_path = excluded.root_path,
                trust_level = excluded.trust_level,
                approval_policy_json = excluded.approval_policy_json,
                model_permissions_json = excluded.model_permissions_json,
                tool_permissions_json = excluded.tool_permissions_json,
                memory_scope_json = excluded.memory_scope_json,
                updated_at = CURRENT_TIMESTAMP",
            params![
                project.id,
                project.name,
                project.root_path,
                project.trust_level.as_str(),
                approval,
                models,
                tools,
                memory,
            ],
        )
        .map_err(sql_string)?;
    // Replace the scope set wholesale so removed scopes don't linger.
    connection
        .execute(
            "DELETE FROM project_file_scopes WHERE project_id = ?1",
            params![project.id],
        )
        .map_err(sql_string)?;
    for (index, scope) in project.allowed_file_scopes.iter().enumerate() {
        connection
            .execute(
                "INSERT INTO project_file_scopes (
                    project_id, scope_index, path, recursive, can_read, can_write, reason
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    project.id,
                    index as i64,
                    scope.path,
                    scope.recursive as i64,
                    scope.can_read as i64,
                    scope.can_write as i64,
                    scope.reason,
                ],
            )
            .map_err(sql_string)?;
    }
    Ok(())
}

fn load_project(connection: &Connection, id: &str) -> Result<Option<ProjectRecord>, String> {
    let row = connection
        .query_row(
            "SELECT id, name, root_path, trust_level, approval_policy_json,
                    model_permissions_json, tool_permissions_json, memory_scope_json,
                    created_at, updated_at
             FROM projects WHERE id = ?1",
            params![id.trim()],
            project_from_row,
        )
        .optional()
        .map_err(sql_string)?;
    match row {
        Some(mut project) => {
            project.allowed_file_scopes = load_scopes(connection, &project.id)?;
            Ok(Some(project))
        }
        None => Ok(None),
    }
}

fn list_projects(connection: &Connection) -> Result<Vec<ProjectRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, name, root_path, trust_level, approval_policy_json,
                    model_permissions_json, tool_permissions_json, memory_scope_json,
                    created_at, updated_at
             FROM projects ORDER BY updated_at DESC, name",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map([], project_from_row)
        .map_err(sql_string)?;
    let mut projects = rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)?;
    for project in &mut projects {
        project.allowed_file_scopes = load_scopes(connection, &project.id)?;
    }
    Ok(projects)
}

fn load_scopes(connection: &Connection, project_id: &str) -> Result<Vec<FileScopeRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT path, recursive, can_read, can_write, reason
             FROM project_file_scopes WHERE project_id = ?1 ORDER BY scope_index",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map(params![project_id], |row| {
            Ok(FileScopeRecord {
                path: row.get(0)?,
                recursive: row.get::<_, i64>(1)? != 0,
                can_read: row.get::<_, i64>(2)? != 0,
                can_write: row.get::<_, i64>(3)? != 0,
                reason: row.get(4)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn project_from_row(row: &rusqlite::Row) -> rusqlite::Result<ProjectRecord> {
    Ok(ProjectRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        root_path: row.get(2)?,
        trust_level: ProjectTrustLevel::from_str(&row.get::<_, String>(3)?),
        allowed_file_scopes: Vec::new(),
        approval_policy: parse_json(&row.get::<_, String>(4)?).unwrap_or_default(),
        model_permissions: parse_json(&row.get::<_, String>(5)?).unwrap_or_default(),
        tool_permissions: parse_json(&row.get::<_, String>(6)?).unwrap_or_default(),
        memory_scope: parse_json(&row.get::<_, String>(7)?).unwrap_or_default(),
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn parse_json<T: serde::de::DeserializeOwned + Default>(value: &str) -> Option<T> {
    serde_json::from_str(value).ok()
}

fn json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| error.to_string())
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
