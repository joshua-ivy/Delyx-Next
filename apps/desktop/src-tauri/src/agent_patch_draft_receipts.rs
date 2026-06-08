use crate::agent_executor_bridge::AgentExecutionBridgeView;
use crate::agent_patch_draft_bridge::{AgentPatchDraftBridgeView, AgentPatchDraftExecuteRequest};
use crate::agent_run::{
    AgentNode, AgentRunError, AgentRunLedger, AgentRunStatus, EvidenceRecordInput,
    EvidenceRelevance,
};
use crate::thread_run_bridge::{ThreadRunBridgeState, ThreadRunStore};
use std::sync::MutexGuard;

pub(crate) fn record_model_started(
    threads: &ThreadRunBridgeState,
    request: &AgentPatchDraftExecuteRequest,
) -> Result<String, String> {
    let mut store = lock_thread_store(threads)?;
    let node = record_model_started_in_store(&mut store, request)?;
    threads.persist(&store)?;
    Ok(node.id)
}

pub(crate) fn record_model_started_in_store(
    store: &mut ThreadRunStore,
    request: &AgentPatchDraftExecuteRequest,
) -> Result<AgentNode, String> {
    let node = store
        .ledger
        .append_node(
            &request.run_id,
            "model_call",
            &format!("Ollama PatchDraftAgent: {}", request.model),
        )
        .map_err(agent_error)?;
    store
        .ledger
        .append_event(
            &request.run_id,
            "model_call.started",
            &format!("Ollama patch draft request sent to {}.", request.model),
        )
        .map_err(agent_error)?;
    Ok(node)
}

pub(crate) fn record_model_completed(
    store: &mut ThreadRunStore,
    request: &AgentPatchDraftExecuteRequest,
    node_id: &str,
    text: &str,
) -> Result<(), String> {
    mark_node(
        &mut store.ledger,
        &request.run_id,
        node_id,
        AgentRunStatus::Completed,
    )?;
    store
        .ledger
        .append_event(
            &request.run_id,
            "model_call.completed",
            &format!("Ollama patch draft replied with {}.", request.model),
        )
        .map_err(agent_error)?;
    store
        .ledger
        .record_artifact(
            &request.run_id,
            "model_response",
            &format!("PatchDraftAgent response from {}", request.model),
        )
        .map_err(agent_error)?;
    store
        .ledger
        .record_evidence_detail(&request.run_id, model_evidence(request, text))
        .map_err(agent_error)?;
    Ok(())
}

pub(crate) fn record_model_failed(
    threads: &ThreadRunBridgeState,
    request: &AgentPatchDraftExecuteRequest,
    node_id: &str,
    error: &str,
) -> Result<(), String> {
    let mut store = lock_thread_store(threads)?;
    mark_node(
        &mut store.ledger,
        &request.run_id,
        node_id,
        AgentRunStatus::Failed,
    )?;
    let _ = store
        .ledger
        .append_event(&request.run_id, "model_call.failed", error);
    store
        .ledger
        .fail_run(&request.run_id, error)
        .map_err(agent_error)?;
    threads.persist(&store)?;
    Ok(())
}

pub(crate) fn draft_view(
    view: AgentExecutionBridgeView,
    model: &str,
    provider_id: &str,
) -> AgentPatchDraftBridgeView {
    AgentPatchDraftBridgeView {
        message: view.message,
        model: model.to_string(),
        patch_id: view.patch_id,
        provider_id: provider_id.to_string(),
        run_id: view.run_id,
        status: view.status,
    }
}

pub(crate) fn lock_thread_store(
    threads: &ThreadRunBridgeState,
) -> Result<MutexGuard<'_, ThreadRunStore>, String> {
    threads
        .store
        .lock()
        .map_err(|_| "Thread bridge lock failed.".to_string())
}

fn mark_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    node_id: &str,
    status: AgentRunStatus,
) -> Result<(), String> {
    let run = ledger.run_mut(run_id).map_err(agent_error)?;
    let node = run
        .nodes
        .iter_mut()
        .find(|item| item.id == node_id)
        .ok_or_else(|| "PatchDraft model node was not found.".to_string())?;
    node.status = status;
    Ok(())
}

fn model_evidence(request: &AgentPatchDraftExecuteRequest, text: &str) -> EvidenceRecordInput {
    EvidenceRecordInput {
        hash: None,
        quote: Some(text.chars().take(280).collect()),
        relevance: Some(EvidenceRelevance {
            reason: "PatchDraftAgent generated a local patch proposal candidate.".to_string(),
            relationship: "model-generated".to_string(),
            score: 60,
        }),
        retrieved_at: request.created_at_ms.to_string(),
        source_id: format!("ollama:{}", request.model),
        source_kind: "model".to_string(),
        title: format!("PatchDraftAgent response from {}", request.model),
        uri: None,
    }
}

fn agent_error(error: AgentRunError) -> String {
    format!("{error:?}")
}
