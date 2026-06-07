use crate::automation::{AutomationEngine, MissionContract, MissionStatus, ScheduledRun, ScheduledRunStatus};
use crate::local_store_bridge::LocalStoreBridgeState;
use serde::Serialize;
use std::path::Path;

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

#[tauri::command]
pub fn automation_snapshot(state: tauri::State<LocalStoreBridgeState>) -> Result<AutomationStateView, String> {
    automation_snapshot_from_path(state.database_path())
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

fn contract_view(contract: &MissionContract) -> MissionContractView {
    MissionContractView {
        id: contract.id.clone(),
        title: contract.title.clone(),
        status: status_key(contract.status).to_string(),
        scope: contract.scope.clone(),
        allowed_tools: contract.allowed_tools.clone(),
        active_hours: format!("{:02}:00-{:02}:00", contract.active_hours.start_hour, contract.active_hours.end_hour),
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
