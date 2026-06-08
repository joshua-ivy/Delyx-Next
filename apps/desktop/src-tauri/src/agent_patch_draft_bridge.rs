use crate::agent_executor_bridge::execute_patch_proposal_record;
use crate::agent_patch_draft_parser::{
    draft_messages, patch_request_from_draft_text, read_draft_files, validate_request,
};
use crate::agent_patch_draft_receipts::{
    draft_view, lock_thread_store, record_model_completed, record_model_failed,
    record_model_started,
};
use crate::approval_bridge::ApprovalBridgeState;
use crate::model_ollama::send_ollama_chat;
use crate::patch_bridge::PatchBridgeState;
use crate::thread_run_bridge::ThreadRunBridgeState;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(test)]
use crate::agent_patch_draft_receipts::record_model_started_in_store;
#[cfg(test)]
use crate::thread_run_bridge::ThreadRunStore;
#[cfg(test)]
use crate::workspace_bridge::WorkspaceFileReadView;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPatchDraftExecuteRequest {
    pub client_id: String,
    pub run_id: String,
    pub approval_id: String,
    pub approved_roots: Vec<String>,
    pub project_path: String,
    pub model: String,
    pub goal: String,
    pub plan_steps: Vec<String>,
    pub files_likely_involved: Vec<String>,
    pub scope_paths: Vec<String>,
    pub created_at_ms: u64,
    pub max_bytes_per_file: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPatchDraftBridgeView {
    pub status: String,
    pub run_id: String,
    pub patch_id: Option<String>,
    pub message: String,
    pub model: String,
    pub provider_id: String,
}

#[tauri::command]
pub fn agent_execute_patch_draft(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: AgentPatchDraftExecuteRequest,
) -> Result<AgentPatchDraftBridgeView, String> {
    validate_request(&request)?;
    let files = read_draft_files(&request)?;
    let node_id = record_model_started(&threads, &request)?;
    let messages = draft_messages(&request, &files);
    match send_ollama_chat(request.model.clone(), messages, Duration::from_secs(120)) {
        Ok(response) => {
            let mut thread_store = lock_thread_store(&threads)?;
            record_model_completed(&mut thread_store, &request, &node_id, &response.text)?;
            threads.persist(&thread_store)?;
            drop(thread_store);

            let patch_request = patch_request_from_draft_text(&request, &files, &response.text)?;
            approvals.with_engine(|engine| {
                let mut thread_store = lock_thread_store(&threads)?;
                let mut patch_store = patches
                    .store
                    .lock()
                    .map_err(|_| "Patch bridge lock failed.".to_string())?;
                let view = execute_patch_proposal_record(
                    &mut thread_store,
                    &mut patch_store,
                    engine,
                    patch_request,
                )?;
                threads.persist(&thread_store)?;
                patches.save_if_persistent(&patch_store)?;
                Ok(draft_view(view, &response.model, &response.provider_id))
            })?
        }
        Err(error) => {
            record_model_failed(&threads, &request, &node_id, &error)?;
            Err(error)
        }
    }
}

#[cfg(test)]
pub(crate) fn execute_patch_draft_from_model_text(
    thread_store: &mut ThreadRunStore,
    patch_store: &mut crate::patch_bridge::PatchBridgeStore,
    approvals: &crate::approval::ApprovalEngine,
    request: &AgentPatchDraftExecuteRequest,
    files: &[WorkspaceFileReadView],
    model_text: &str,
) -> Result<AgentPatchDraftBridgeView, String> {
    let node = record_model_started_in_store(thread_store, request)?;
    record_model_completed(thread_store, request, &node.id, model_text)?;
    let patch_request = patch_request_from_draft_text(request, files, model_text)?;
    let view = execute_patch_proposal_record(thread_store, patch_store, approvals, patch_request)?;
    Ok(draft_view(view, &request.model, "ollama-local"))
}
