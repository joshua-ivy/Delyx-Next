use crate::automation::{
    AutomationEngine, MissionContract, MissionStatus, ScheduledRun, ScheduledRunStatus,
};
use serde::Serialize;

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

pub fn automation_snapshot_from_engine(engine: &AutomationEngine) -> AutomationStateView {
    AutomationStateView {
        contracts: engine.contracts().iter().map(contract_view).collect(),
        scheduled_runs: engine.scheduled_runs().iter().map(run_view).collect(),
    }
}

fn contract_view(contract: &MissionContract) -> MissionContractView {
    MissionContractView {
        active_hours: format!(
            "{:02}:00-{:02}:00",
            contract.active_hours.start_hour, contract.active_hours.end_hour
        ),
        allowed_tools: contract.allowed_tools.clone(),
        delivery_targets: contract.delivery_targets.clone(),
        id: contract.id.clone(),
        scope: contract.scope.clone(),
        status: status_key(contract.status).to_string(),
        stop_condition: contract.stop_condition.clone(),
        timezone: contract.timezone.clone(),
        title: contract.title.clone(),
        workspace_fingerprint: contract.workspace_fingerprint.clone(),
    }
}

fn run_view(run: &ScheduledRun) -> ScheduledRunView {
    ScheduledRunView {
        approval_id: run.approval_id.clone(),
        contract_id: run.contract_id.clone(),
        id: run.id.clone(),
        reason: run.reason.clone(),
        status: run_status_key(run.status).to_string(),
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
