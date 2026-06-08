use crate::agent_executor::{
    execute_patch_apply_node, execute_patch_proposal_node, AgentExecutionStatus,
};
use crate::agent_patch_restore_executor::execute_patch_restore_node;
use crate::approval_bridge::ApprovalBridgeState;
use crate::patch_apply_bridge::PatchApplyRequest;
use crate::patch_bridge::{PatchBridgeState, PatchFileRequest, PatchProposalRequest};
use crate::patch_restore_bridge::PatchRestoreRequest;
use crate::thread_run_bridge::{ThreadRunBridgeState, ThreadRunStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPatchProposalExecuteRequest {
    pub client_id: String,
    pub run_id: String,
    pub approval_id: String,
    pub approved_roots: Vec<String>,
    pub files: Vec<PatchFileRequest>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentExecutionBridgeView {
    pub status: String,
    pub run_id: String,
    pub patch_id: Option<String>,
    pub message: String,
}

#[tauri::command]
pub fn agent_execute_patch_proposal(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: AgentPatchProposalExecuteRequest,
) -> Result<AgentExecutionBridgeView, String> {
    approvals.with_engine(|engine| {
        let mut thread_store = threads
            .store
            .lock()
            .map_err(|_| "Thread bridge lock failed.".to_string())?;
        let mut patch_store = patches
            .store
            .lock()
            .map_err(|_| "Patch bridge lock failed.".to_string())?;
        let view =
            execute_patch_proposal_record(&mut thread_store, &mut patch_store, engine, request)?;
        threads.persist(&thread_store)?;
        patches.save_if_persistent(&patch_store)?;
        Ok(view)
    })?
}

#[tauri::command]
pub fn agent_execute_patch_apply(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: PatchApplyRequest,
) -> Result<AgentExecutionBridgeView, String> {
    approvals.with_engine(|engine| {
        let mut thread_store = threads
            .store
            .lock()
            .map_err(|_| "Thread bridge lock failed.".to_string())?;
        let mut patch_store = patches
            .store
            .lock()
            .map_err(|_| "Patch bridge lock failed.".to_string())?;
        let view =
            execute_patch_apply_record(&mut thread_store, &mut patch_store, engine, request)?;
        threads.persist(&thread_store)?;
        patches.save_if_persistent(&patch_store)?;
        Ok(view)
    })?
}

#[tauri::command]
pub fn agent_execute_patch_restore(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: PatchRestoreRequest,
) -> Result<AgentExecutionBridgeView, String> {
    approvals.with_engine(|engine| {
        let mut thread_store = threads
            .store
            .lock()
            .map_err(|_| "Thread bridge lock failed.".to_string())?;
        let mut patch_store = patches
            .store
            .lock()
            .map_err(|_| "Patch bridge lock failed.".to_string())?;
        let view =
            execute_patch_restore_record(&mut thread_store, &mut patch_store, engine, request)?;
        threads.persist(&thread_store)?;
        patches.save_if_persistent(&patch_store)?;
        Ok(view)
    })?
}

pub fn execute_patch_proposal_record(
    thread_store: &mut ThreadRunStore,
    patch_store: &mut crate::patch_bridge::PatchBridgeStore,
    approvals: &crate::approval::ApprovalEngine,
    request: AgentPatchProposalExecuteRequest,
) -> Result<AgentExecutionBridgeView, String> {
    let result = execute_patch_proposal_node(
        &mut thread_store.ledger,
        patch_store,
        approvals,
        patch_request(&request),
        request.created_at_ms,
    )?;
    Ok(AgentExecutionBridgeView {
        message: result.message,
        patch_id: result.patch_id,
        run_id: result.run_id,
        status: status_key(result.status).to_string(),
    })
}

pub fn execute_patch_apply_record(
    thread_store: &mut ThreadRunStore,
    patch_store: &mut crate::patch_bridge::PatchBridgeStore,
    approvals: &crate::approval::ApprovalEngine,
    request: PatchApplyRequest,
) -> Result<AgentExecutionBridgeView, String> {
    let result =
        execute_patch_apply_node(&mut thread_store.ledger, patch_store, approvals, request)?;
    Ok(AgentExecutionBridgeView {
        message: result.message,
        patch_id: result.patch_id,
        run_id: result.run_id,
        status: status_key(result.status).to_string(),
    })
}

pub fn execute_patch_restore_record(
    thread_store: &mut ThreadRunStore,
    patch_store: &mut crate::patch_bridge::PatchBridgeStore,
    approvals: &crate::approval::ApprovalEngine,
    request: PatchRestoreRequest,
) -> Result<AgentExecutionBridgeView, String> {
    let result =
        execute_patch_restore_node(&mut thread_store.ledger, patch_store, approvals, request)?;
    Ok(AgentExecutionBridgeView {
        message: result.message,
        patch_id: result.patch_id,
        run_id: result.run_id,
        status: status_key(result.status).to_string(),
    })
}

fn patch_request(request: &AgentPatchProposalExecuteRequest) -> PatchProposalRequest {
    PatchProposalRequest {
        approval_id: request.approval_id.clone(),
        approved_roots: request.approved_roots.clone(),
        client_id: request.client_id.clone(),
        files: request.files.clone(),
        run_id: request.run_id.clone(),
    }
}

fn status_key(status: AgentExecutionStatus) -> &'static str {
    match status {
        AgentExecutionStatus::Completed => "completed",
        AgentExecutionStatus::Failed => "failed",
        AgentExecutionStatus::WaitingForApproval => "waiting_for_approval",
    }
}
