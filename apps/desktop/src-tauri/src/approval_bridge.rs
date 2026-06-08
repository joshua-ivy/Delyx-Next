use crate::approval::{ActionProposal, ApprovalEngine, ApprovalError, ProposalInput};
use crate::approval_bridge_keys::{parse_action, parse_risk, risk_key, status_key};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct ApprovalBridgeState {
    pub(crate) store: Mutex<ApprovalBridgeStore>,
    database_path: Option<PathBuf>,
}

impl ApprovalBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        let store = crate::approval_persistence::load_from_path(&database_path)?;
        Ok(Self {
            store: Mutex::new(store),
            database_path: Some(database_path),
        })
    }

    pub fn with_engine<R>(&self, read: impl FnOnce(&ApprovalEngine) -> R) -> Result<R, String> {
        let store = self
            .store
            .lock()
            .map_err(|_| "Approval bridge lock failed.".to_string())?;
        Ok(read(&store.engine))
    }

    pub fn with_store_mut<R>(
        &self,
        write: impl FnOnce(&mut ApprovalBridgeStore) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| "Approval bridge lock failed.".to_string())?;
        let result = write(&mut store)?;
        self.persist(&store)?;
        Ok(result)
    }

    fn persist(&self, store: &ApprovalBridgeStore) -> Result<(), String> {
        if let Some(path) = &self.database_path {
            crate::approval_persistence::save_to_path(store, path)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct ApprovalBridgeStore {
    pub(crate) engine: ApprovalEngine,
    pub(crate) records: Vec<ApprovalBridgeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApprovalBridgeRecord {
    pub(crate) action_type: String,
    pub(crate) client_id: String,
    pub(crate) expires_at: String,
    pub(crate) proposal_id: String,
    pub(crate) required_permission: String,
    pub(crate) run_id: String,
    pub(crate) scope: PermissionScopeView,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalProposalRequest {
    pub client_id: String,
    pub run_id: String,
    pub node_id: String,
    pub action_type: String,
    pub risk_label: String,
    pub required_permission: String,
    pub rationale: String,
    pub expected_result: String,
    pub rollback_plan: Option<String>,
    pub scope: PermissionScopeView,
    pub expires_at: String,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDecisionRequest {
    pub proposal_id: String,
    pub decision: String,
    pub decided_at_ms: u64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionScopeView {
    pub kind: String,
    pub summary: String,
    pub project_id: Option<String>,
    pub root: Option<String>,
    pub paths: Option<Vec<String>>,
    pub commands: Option<Vec<String>>,
    pub connector_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionProposalBridgeView {
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub action_type: String,
    pub risk_label: String,
    pub required_permission: String,
    pub rationale: String,
    pub expected_result: String,
    pub rollback_plan: Option<String>,
    pub scope: PermissionScopeView,
    pub expires_at: String,
    pub status: String,
}

#[tauri::command]
pub fn approval_propose(
    state: tauri::State<ApprovalBridgeState>,
    request: ApprovalProposalRequest,
) -> Result<ActionProposalBridgeView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Approval bridge lock failed.".to_string())?;
    let view = propose_approval_record(&mut store, request)?;
    state.persist(&store)?;
    Ok(view)
}

#[tauri::command]
pub fn approval_decide(
    state: tauri::State<ApprovalBridgeState>,
    request: ApprovalDecisionRequest,
) -> Result<ActionProposalBridgeView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Approval bridge lock failed.".to_string())?;
    let view = decide_approval_record(&mut store, request)?;
    state.persist(&store)?;
    Ok(view)
}

#[tauri::command]
pub fn approval_snapshot(
    state: tauri::State<ApprovalBridgeState>,
    run_id: String,
) -> Result<Vec<ActionProposalBridgeView>, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "Approval bridge lock failed.".to_string())?;
    Ok(approval_snapshot_from_store(&store, &run_id))
}

pub fn propose_approval_record(
    store: &mut ApprovalBridgeStore,
    request: ApprovalProposalRequest,
) -> Result<ActionProposalBridgeView, String> {
    validate_request(&request)?;
    if let Some(existing) = store
        .records
        .iter()
        .find(|record| record.client_id == request.client_id)
    {
        return view_for_record(store, existing);
    }
    let action = parse_action(&request.action_type)?;
    let proposal = store.engine.propose(ProposalInput {
        action,
        expected_result: request.expected_result,
        expires_at: request.expires_at_ms,
        node_id: request.node_id,
        reason: request.rationale,
        risk: parse_risk(&request.risk_label)?,
        rollback_plan: request.rollback_plan.unwrap_or_default(),
        run_id: request.run_id.clone(),
        scope: request.scope.summary.clone(),
    });
    let record = ApprovalBridgeRecord {
        action_type: request.action_type,
        client_id: request.client_id,
        expires_at: request.expires_at,
        proposal_id: proposal.id.clone(),
        required_permission: request.required_permission,
        run_id: request.run_id,
        scope: request.scope,
    };
    store.records.push(record);
    view_for_proposal_id(store, &proposal.id)
}

pub fn decide_approval_record(
    store: &mut ApprovalBridgeStore,
    request: ApprovalDecisionRequest,
) -> Result<ActionProposalBridgeView, String> {
    store.engine.expire_due(request.decided_at_ms);
    let note = request
        .note
        .as_deref()
        .unwrap_or("decision recorded from Delyx UI");
    let result = match request.decision.as_str() {
        "approved" => store
            .engine
            .approve(&request.proposal_id, request.decided_at_ms, note),
        "denied" => store
            .engine
            .deny(&request.proposal_id, request.decided_at_ms, note),
        _ => return Err("Unsupported approval decision.".to_string()),
    };
    match result {
        Ok(()) | Err(ApprovalError::AlreadyDecided) | Err(ApprovalError::Expired) => {
            view_for_proposal_id(store, &request.proposal_id)
        }
        Err(error) => Err(format!("{error:?}")),
    }
}

pub fn approval_snapshot_from_store(
    store: &ApprovalBridgeStore,
    run_id: &str,
) -> Vec<ActionProposalBridgeView> {
    store
        .records
        .iter()
        .filter(|record| record.run_id == run_id)
        .filter_map(|record| view_for_record(store, record).ok())
        .collect()
}

fn view_for_proposal_id(
    store: &ApprovalBridgeStore,
    proposal_id: &str,
) -> Result<ActionProposalBridgeView, String> {
    let record = store
        .records
        .iter()
        .find(|record| record.proposal_id == proposal_id)
        .ok_or_else(|| "Approval proposal not found.".to_string())?;
    view_for_record(store, record)
}

fn view_for_record(
    store: &ApprovalBridgeStore,
    record: &ApprovalBridgeRecord,
) -> Result<ActionProposalBridgeView, String> {
    let proposal = proposal_for_record(store, record)?;
    Ok(ActionProposalBridgeView {
        action_type: record.action_type.clone(),
        expected_result: proposal.expected_result.clone(),
        expires_at: record.expires_at.clone(),
        id: proposal.id.clone(),
        node_id: proposal.node_id.clone(),
        rationale: proposal.reason.clone(),
        required_permission: record.required_permission.clone(),
        risk_label: risk_key(proposal.risk).to_string(),
        rollback_plan: (!proposal.rollback_plan.is_empty()).then(|| proposal.rollback_plan.clone()),
        run_id: proposal.run_id.clone(),
        scope: record.scope.clone(),
        status: status_key(proposal.status).to_string(),
    })
}

fn proposal_for_record<'a>(
    store: &'a ApprovalBridgeStore,
    record: &ApprovalBridgeRecord,
) -> Result<&'a ActionProposal, String> {
    store
        .engine
        .list_proposals(&record.run_id)
        .into_iter()
        .find(|proposal| proposal.id == record.proposal_id)
        .ok_or_else(|| "Approval proposal not found.".to_string())
}

fn validate_request(request: &ApprovalProposalRequest) -> Result<(), String> {
    if request.client_id.trim().is_empty()
        || request.run_id.trim().is_empty()
        || request.node_id.trim().is_empty()
    {
        return Err("Approval proposal requires client, run, and node IDs.".to_string());
    }
    if request.expires_at_ms == 0 || request.expires_at.trim().is_empty() {
        return Err("Approval proposal requires an expiration.".to_string());
    }
    if request.scope.summary.trim().is_empty() {
        return Err("Approval proposal requires visible scope.".to_string());
    }
    Ok(())
}
