use crate::approval::ApprovalEngine;
use crate::approval_bridge::{
    ApprovalBridgeRecord, ApprovalBridgeState, ApprovalBridgeStore, PermissionScopeView,
};
use crate::automation::{ActiveHours, AutomationEngine, MissionContractInput, ScheduledRun};
pub use crate::automation_bridge_views::{
    automation_snapshot_from_engine, AutomationStateView, MissionContractView, ScheduledRunView,
};
use serde::Deserialize;
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledRunRequest {
    pub contract_id: String,
    pub workspace_fingerprint: String,
    pub requested_at_ms: u64,
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

#[tauri::command]
pub fn automation_schedule_due_run(
    state: tauri::State<AutomationBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: ScheduledRunRequest,
) -> Result<AutomationStateView, String> {
    approvals.with_store_mut(|approval_store| {
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| "Automation bridge lock failed.".to_string())?;
        let view = schedule_due_run_record(&mut engine, approval_store, request)?;
        state.save_if_persistent(&engine)?;
        Ok(view)
    })
}

pub fn automation_snapshot_from_path(path: &Path) -> Result<AutomationStateView, String> {
    let engine = crate::automation_persistence::load_from_path(path)?;
    Ok(automation_snapshot_from_engine(&engine))
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

pub fn schedule_due_run_record(
    engine: &mut AutomationEngine,
    approval_store: &mut ApprovalBridgeStore,
    request: ScheduledRunRequest,
) -> Result<AutomationStateView, String> {
    validate_scheduled_run_request(&request)?;
    let run = engine
        .schedule_due_run(
            &request.contract_id,
            &request.workspace_fingerprint,
            request.requested_at_ms,
            &mut approval_store.engine,
        )
        .map_err(|error| format!("{error:?}"))?;
    if let Some(approval_id) = run.approval_id.as_deref() {
        record_generated_schedule_approval(engine, approval_store, &run, approval_id)?;
    }
    Ok(automation_snapshot_from_engine(engine))
}

fn record_generated_schedule_approval(
    engine: &AutomationEngine,
    approval_store: &mut ApprovalBridgeStore,
    run: &ScheduledRun,
    approval_id: &str,
) -> Result<(), String> {
    if approval_store
        .records
        .iter()
        .any(|record| record.proposal_id == approval_id)
    {
        return Ok(());
    }
    let proposal = approval_store
        .engine
        .all_proposals()
        .iter()
        .find(|proposal| proposal.id == approval_id)
        .ok_or_else(|| "Generated scheduled-run approval not found.".to_string())?;
    let tools = engine
        .contracts()
        .iter()
        .find(|contract| contract.id == run.contract_id)
        .map(|contract| contract.allowed_tools.clone());
    approval_store.records.push(ApprovalBridgeRecord {
        action_type: "schedule_work".to_string(),
        client_id: format!("automation-scheduled-run-{}", run.id),
        expires_at: format!("epoch_ms:{}", proposal.expires_at),
        proposal_id: approval_id.to_string(),
        required_permission: "schedule_work".to_string(),
        run_id: proposal.run_id.clone(),
        scope: PermissionScopeView {
            commands: tools,
            connector_id: None,
            kind: "automation".to_string(),
            paths: None,
            project_id: None,
            root: None,
            summary: proposal.scope.clone(),
        },
    });
    Ok(())
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

fn validate_scheduled_run_request(request: &ScheduledRunRequest) -> Result<(), String> {
    if request.contract_id.trim().is_empty() || request.workspace_fingerprint.trim().is_empty() {
        return Err("Scheduled run requires contract ID and workspace fingerprint.".to_string());
    }
    if request.requested_at_ms == 0 {
        return Err("Scheduled run requires a request timestamp.".to_string());
    }
    Ok(())
}
