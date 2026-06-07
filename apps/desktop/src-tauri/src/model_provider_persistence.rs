use crate::model_provider::{ModelRole, RoleRoute};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_routes_to_path(path: &Path, routes: &[RoleRoute]) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    clear_routes(&connection)?;
    for route in routes {
        insert_route(&connection, route)?;
    }
    Ok(())
}

pub fn load_routes_from_path(path: &Path) -> Result<Vec<RoleRoute>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let mut statement = connection
        .prepare("SELECT role, provider_id, model_id FROM model_role_routes ORDER BY role")
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut routes = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let role: String = row.get(0).map_err(sql_string)?;
        routes.push(RoleRoute {
            role: parse_role(&role)?,
            provider_id: row.get(1).map_err(sql_string)?,
            model_id: row.get(2).map_err(sql_string)?,
        });
    }
    Ok(routes)
}

fn clear_routes(connection: &Connection) -> Result<(), String> {
    connection.execute("DELETE FROM model_role_routes", []).map(|_| ()).map_err(sql_string)
}

fn insert_route(connection: &Connection, route: &RoleRoute) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO model_role_routes (role, provider_id, model_id, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(role) DO UPDATE SET
               provider_id = excluded.provider_id,
               model_id = excluded.model_id,
               updated_at = CURRENT_TIMESTAMP",
            params![role_key(route.role), route.provider_id, route.model_id],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn role_key(role: ModelRole) -> &'static str {
    match role {
        ModelRole::Answer => "answer",
        ModelRole::Coding => "coding",
        ModelRole::DeepResearch => "deep_research",
        ModelRole::Embedding => "embedding",
        ModelRole::Helper => "helper",
        ModelRole::MaxReasoning => "max_reasoning",
        ModelRole::Scoring => "scoring",
    }
}

fn parse_role(value: &str) -> Result<ModelRole, String> {
    match value {
        "answer" => Ok(ModelRole::Answer),
        "coding" => Ok(ModelRole::Coding),
        "deep_research" => Ok(ModelRole::DeepResearch),
        "embedding" => Ok(ModelRole::Embedding),
        "helper" => Ok(ModelRole::Helper),
        "max_reasoning" => Ok(ModelRole::MaxReasoning),
        "scoring" => Ok(ModelRole::Scoring),
        _ => Err("Unsupported persisted model role.".to_string()),
    }
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
