use crate::plan_bridge::PlanView;
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_plan_to_path(path: &Path, project_id: &str, plan: &PlanView) -> Result<(), String> {
    validate_plan(project_id, plan)?;
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    insert_plan(&connection, project_id, plan)
}

pub fn load_plans_from_path(path: &Path, project_id: &str) -> Result<Vec<PlanView>, String> {
    if project_id.trim().is_empty() {
        return Err("Plan snapshot requires a project ID.".to_string());
    }
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    load_plans_from_connection(&connection, project_id)
}

fn insert_plan(connection: &Connection, project_id: &str, plan: &PlanView) -> Result<(), String> {
    let plan_json = serde_json::to_string(plan).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO plan_records (thread_id, project_id, plan_json, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(thread_id) DO UPDATE SET
               project_id = excluded.project_id,
               plan_json = excluded.plan_json,
               updated_at = CURRENT_TIMESTAMP",
            params![plan.thread_id, project_id, plan_json],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_plans_from_connection(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<PlanView>, String> {
    let mut statement = connection
        .prepare(
            "SELECT plan_json FROM plan_records
             WHERE project_id = ?1 ORDER BY updated_at DESC, rowid DESC",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map([project_id], |row| row.get::<_, String>(0))
        .map_err(sql_string)?;
    rows.map(|row| {
        row.map_err(sql_string)
            .and_then(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
    })
    .collect()
}

fn validate_plan(project_id: &str, plan: &PlanView) -> Result<(), String> {
    if project_id.trim().is_empty() || plan.thread_id.trim().is_empty() {
        return Err("Plan persistence requires project and thread IDs.".to_string());
    }
    if plan.goal_understanding.trim().is_empty() || plan.steps.is_empty() {
        return Err("Plan persistence requires a goal and at least one step.".to_string());
    }
    Ok(())
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
