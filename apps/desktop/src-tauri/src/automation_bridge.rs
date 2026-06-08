use crate::approval::ApprovalEngine;
use crate::approval_bridge::ApprovalBridgeState;
use crate::automation::{
    ActiveHours, AutomationEngine, MissionContract, MissionContractInput, MissionStatus,
    ScheduledRun, ScheduledRunStatus,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Default)]
pub struct AutomationBridgeState {
    engine: Mutex<AutomationEngine>,
    database_path: Option<PathBuf>,
}

impl AutomationBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        let engine = crate::automation_persistence::load_from_path(&database_path)?;
        Ok(Self {
            engine: Mutex::new(engine),
            database_path: Some(database_path),
        })
    }

    fn save_if_persistent(&self, engine: &AutomationEngine) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::automation_persistence::save_to_path(engine, path),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationStateView {
    pub contracts: Vec<MissionContractView>,
    pub scheduled_runs: Vec<ScheduledRunView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionContractView {
    pub id: String,
    pub title: String,
    pub status: String,
    pub scope: String,
    pub allowed_tools: Vec<String>,
    pub active_hours: String,
    pub timezone: String,
    pub delivery_targets: Vec<String>,
    pub stop_condition: String,
    pub workspace_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledRunView {
    pub id: String,
    pub contract_id: String,
    pub status: String,
    pub reason: String,
    pub approval_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionContractRequest {
    pub title: String,
    pub scope: String,
    pub allowed_tools: Vec<String>,
    pub active_start_hour: u8,
    pub active_end_hour: u8,
    pub timezone: String,
    pub delivery_targets: Vec<String>,
    pub stop_condition: String,
    pub workspace_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionApproveRequest {
    pub contract_id: String,
    pub approval_id: String,
    pub approved_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionActionRequest {
    pub contract_id: String,
}

#[tauri::command]
pub fn automation_snapshot(
    state: tauri::State<AutomationBridgeState>,
) -> Result<AutomationStateView, String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "Automation bridge lock failed.".to_string())?;
    Ok(automation_snapshot_from_engine(&engine))
}

#[tauri::command]
pub fn automation_contract_create(
    state: tauri::State<AutomationBridgeState>,
    request: MissionContractRequest,
) -> Result<AutomationStateView, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Automation bridge lock failed.".to_string())?;
    let view = create_contract_record(&mut engine, request)?;
    state.save_if_persistent(&engine)?;
    Ok(view)
}

#[tauri::command]
pub fn automation_contract_approve(
    state: tauri::State<AutomationBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: MissionApproveRequest,
) -> Result<AutomationStateView, String> {
    approvals.with_engine(|approval_engine| {
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| "Automation bridge lock failed.".to_string())?;
        let view = approve_contract_record(&mut engine, approval_engine, request)?;
        state.save_if_persistent(&engine)?;
        Ok(view)
    })?
}

#[tauri::command]
pub fn automation_contract_pause(
    state: tauri::State<AutomationBridgeState>,
    request: MissionActionRequest,
) -> Result<AutomationStateView, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Automation bridge lock failed.".to_string())?;
    let view = pause_contract_record(&mut engine, request)?;
    state.save_if_persistent(&engine)?;
    Ok(view)
}

pub fn automation_snapshot_from_path(path: &Path) -> Result<AutomationStateView, String> {
    let engine = crate::automation_persistence::load_from_path(path)?;
    Ok(automation_snapshot_from_engine(&engine))
}

pub fn automation_snapshot_from_engine(engine: &AutomationEngine) -> AutomationStateView {
    AutomationStateView {
        contracts: engine.contracts().iter().map(contract_view).collect(),
        scheduled_runs: engine.scheduled_runs().iter().map(run_view).collect(),
    }
}

pub fn create_contract_record(
    engine: &mut AutomationEngine,
    request: MissionContractRequest,
) -> Result<AutomationStateView, String> {
    validate_contract_request(&request)?;
    engine.create_contract(MissionContractInput {
        active_hours: ActiveHours {
            start_hour: request.active_start_hour,
            end_hour: request.active_end_hour,
        },
        allowed_tools: request.allowed_tools,
        delivery_targets: request.delivery_targets,
        scope: request.scope,
        stop_condition: request.stop_condition,
        timezone: request.timezone,
        title: request.title,
        workspace_fingerprint: request.workspace_fingerprint,
    });
    Ok(automation_snapshot_from_engine(engine))
}

pub fn approve_contract_record(
    engine: &mut AutomationEngine,
    approvals: &ApprovalEngine,
    request: MissionApproveRequest,
) -> Result<AutomationStateView, String> {
    if request.contract_id.trim().is_empty() || request.approval_id.trim().is_empty() {
        return Err("Mission approval requires contract and approval IDs.".to_string());
    }
    engine
        .approve_contract(
            &request.contract_id,
            &request.approval_id,
            request.approved_at_ms,
            approvals,
        )
        .map_err(|error| format!("{error:?}"))?;
    Ok(automation_snapshot_from_engine(engine))
}

pub fn pause_contract_record(
    engine: &mut AutomationEngine,
    request: MissionActionRequest,
) -> Result<AutomationStateView, String> {
    if request.contract_id.trim().is_empty() {
        return Err("Mission pause requires a contract ID.".to_string());
    }
    engine
        .pause_contract(&request.contract_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(automation_snapshot_from_engine(engine))
}

fn contract_view(contract: &MissionContract) -> MissionContractView {
    MissionContractView {
        id: contract.id.clone(),
        title: contract.title.clone(),
        status: status_key(contract.status).to_string(),
        scope: contract.scope.clone(),
        allowed_tools: contract.allowed_tools.clone(),
        active_hours: format!(
            "{:02}:00-{:02}:00",
            contract.active_hours.start_hour, contract.active_hours.end_hour
        ),
        timezone: contract.timezone.clone(),
        delivery_targets: contract.delivery_targets.clone(),
        stop_condition: contract.stop_condition.clone(),
        workspace_fingerprint: contract.workspace_fingerprint.clone(),
    }
}

fn run_view(run: &ScheduledRun) -> ScheduledRunView {
    ScheduledRunView {
        id: run.id.clone(),
        contract_id: run.contract_id.clone(),
        status: run_status_key(run.status).to_string(),
        reason: run.reason.clone(),
        approval_id: run.approval_id.clone(),
    }
}

fn status_key(status: MissionStatus) -> &'static str {
    match status {
        MissionStatus::Active => "active",
        MissionStatus::Blocked => "blocked",
        MissionStatus::Paused => "paused",
    }
}

fn run_status_key(status: ScheduledRunStatus) -> &'static str {
    match status {
        ScheduledRunStatus::Blocked => "blocked",
        ScheduledRunStatus::Created => "created",
        ScheduledRunStatus::WaitingForApproval => "waiting_for_approval",
    }
}

fn validate_contract_request(request: &MissionContractRequest) -> Result<(), String> {
    if request.title.trim().is_empty()
        || request.scope.trim().is_empty()
        || request.timezone.trim().is_empty()
        || request.stop_condition.trim().is_empty()
        || request.workspace_fingerprint.trim().is_empty()
    {
        return Err("Mission contract requires title, scope, timezone, stop condition, and workspace fingerprint.".to_string());
    }
    if request.allowed_tools.is_empty() || request.delivery_targets.is_empty() {
        return Err("Mission contract requires allowed tools and delivery targets.".to_string());
    }
    if request.active_start_hour >= request.active_end_hour || request.active_end_hour > 24 {
        return Err("Mission active hours must be an increasing 0-24 range.".to_string());
    }
    Ok(())
}
