use crate::automation::{
    ActiveHours, AutomationEngine, MissionContract, MissionStatus, ScheduledRun, ScheduledRunStatus,
};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn save_to_path(engine: &AutomationEngine, path: &Path) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    clear_tables(&connection)?;
    for contract in engine.contracts() {
        insert_contract(&connection, contract)?;
    }
    for run in engine.scheduled_runs() {
        insert_run(&connection, run)?;
    }
    Ok(())
}

pub fn load_from_path(path: &Path) -> Result<AutomationEngine, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let contracts = load_contracts(&connection)?;
    let runs = load_runs(&connection)?;
    Ok(AutomationEngine::from_loaded(contracts, runs))
}

fn clear_tables(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "DELETE FROM scheduled_runs;
             DELETE FROM automation_contracts;",
        )
        .map_err(sql_string)
}

fn insert_contract(connection: &Connection, contract: &MissionContract) -> Result<(), String> {
    let allowed_tools = serde_json::to_string(&contract.allowed_tools).map_err(|error| error.to_string())?;
    let delivery_targets = serde_json::to_string(&contract.delivery_targets).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO automation_contracts
             (id, title, status, scope, allowed_tools_json, active_start_hour, active_end_hour, timezone, delivery_targets_json, stop_condition, workspace_fingerprint)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                contract.id,
                contract.title,
                status_key(contract.status),
                contract.scope,
                allowed_tools,
                contract.active_hours.start_hour as i64,
                contract.active_hours.end_hour as i64,
                contract.timezone,
                delivery_targets,
                contract.stop_condition,
                contract.workspace_fingerprint,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn insert_run(connection: &Connection, run: &ScheduledRun) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO scheduled_runs (id, contract_id, status, reason, approval_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![run.id, run.contract_id, run_status_key(run.status), run.reason, run.approval_id],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn load_contracts(connection: &Connection) -> Result<Vec<MissionContract>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, title, status, scope, allowed_tools_json, active_start_hour, active_end_hour, timezone,
                    delivery_targets_json, stop_condition, workspace_fingerprint
             FROM automation_contracts ORDER BY rowid",
        )
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut contracts = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let status: String = row.get(2).map_err(sql_string)?;
        let allowed_tools: String = row.get(4).map_err(sql_string)?;
        let delivery_targets: String = row.get(8).map_err(sql_string)?;
        contracts.push(MissionContract {
            id: row.get(0).map_err(sql_string)?,
            title: row.get(1).map_err(sql_string)?,
            status: parse_status(&status)?,
            scope: row.get(3).map_err(sql_string)?,
            allowed_tools: serde_json::from_str(&allowed_tools).map_err(|error| error.to_string())?,
            active_hours: ActiveHours {
                start_hour: row.get::<_, i64>(5).map_err(sql_string)? as u8,
                end_hour: row.get::<_, i64>(6).map_err(sql_string)? as u8,
            },
            timezone: row.get(7).map_err(sql_string)?,
            delivery_targets: serde_json::from_str(&delivery_targets).map_err(|error| error.to_string())?,
            stop_condition: row.get(9).map_err(sql_string)?,
            workspace_fingerprint: row.get(10).map_err(sql_string)?,
        });
    }
    Ok(contracts)
}

fn load_runs(connection: &Connection) -> Result<Vec<ScheduledRun>, String> {
    let mut statement = connection
        .prepare("SELECT id, contract_id, status, reason, approval_id FROM scheduled_runs ORDER BY rowid")
        .map_err(sql_string)?;
    let mut rows = statement.query([]).map_err(sql_string)?;
    let mut runs = Vec::new();
    while let Some(row) = rows.next().map_err(sql_string)? {
        let status: String = row.get(2).map_err(sql_string)?;
        runs.push(ScheduledRun {
            id: row.get(0).map_err(sql_string)?,
            contract_id: row.get(1).map_err(sql_string)?,
            status: parse_run_status(&status)?,
            reason: row.get(3).map_err(sql_string)?,
            approval_id: row.get(4).map_err(sql_string)?,
        });
    }
    Ok(runs)
}

fn status_key(status: MissionStatus) -> &'static str {
    match status {
        MissionStatus::Active => "active",
        MissionStatus::Blocked => "blocked",
        MissionStatus::Paused => "paused",
    }
}

fn parse_status(value: &str) -> Result<MissionStatus, String> {
    match value {
        "active" => Ok(MissionStatus::Active),
        "blocked" => Ok(MissionStatus::Blocked),
        "paused" => Ok(MissionStatus::Paused),
        _ => Err("Unsupported persisted mission status.".to_string()),
    }
}

fn run_status_key(status: ScheduledRunStatus) -> &'static str {
    match status {
        ScheduledRunStatus::Blocked => "blocked",
        ScheduledRunStatus::Created => "created",
        ScheduledRunStatus::WaitingForApproval => "waiting_for_approval",
    }
}

fn parse_run_status(value: &str) -> Result<ScheduledRunStatus, String> {
    match value {
        "blocked" => Ok(ScheduledRunStatus::Blocked),
        "created" => Ok(ScheduledRunStatus::Created),
        "waiting_for_approval" => Ok(ScheduledRunStatus::WaitingForApproval),
        _ => Err("Unsupported persisted scheduled run status.".to_string()),
    }
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
