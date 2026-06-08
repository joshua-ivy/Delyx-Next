use crate::approval::ApprovalEngine;
use crate::approval_bridge::ApprovalBridgeState;
use crate::memory::{MemoryCandidateInput, MemoryScope, MemoryStore, SourceRunStatus};
pub use crate::memory_bridge_views::{
    memory_snapshot_from_store, MemoryCandidateView, MemoryRecordView, MemoryStateView,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Default)]
pub struct MemoryBridgeState {
    store: Mutex<MemoryStore>,
    database_path: Option<PathBuf>,
}

impl MemoryBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        let store = crate::memory_persistence::load_from_path(&database_path)?;
        Ok(Self {
            store: Mutex::new(store),
            database_path: Some(database_path),
        })
    }

    fn save_if_persistent(&self, store: &MemoryStore) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::memory_persistence::save_to_path(store, path),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCandidateRequest {
    pub scope: String,
    pub key: String,
    pub value: String,
    pub source_run_id: String,
    pub source_thread_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryPromoteRequest {
    pub candidate_id: String,
    pub approval_id: String,
    pub approved_at_ms: u64,
    pub source_run_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCandidateActionRequest {
    pub candidate_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecordActionRequest {
    pub record_id: String,
}

#[tauri::command]
pub fn memory_snapshot(state: tauri::State<MemoryBridgeState>) -> Result<MemoryStateView, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "Memory bridge lock failed.".to_string())?;
    Ok(memory_snapshot_from_store(&store))
}

#[tauri::command]
pub fn memory_candidate_propose(
    state: tauri::State<MemoryBridgeState>,
    request: MemoryCandidateRequest,
) -> Result<MemoryStateView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Memory bridge lock failed.".to_string())?;
    let view = propose_memory_candidate_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(view)
}

#[tauri::command]
pub fn memory_promote_approved(
    state: tauri::State<MemoryBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: MemoryPromoteRequest,
) -> Result<MemoryStateView, String> {
    approvals.with_engine(|engine| {
        let mut store = state
            .store
            .lock()
            .map_err(|_| "Memory bridge lock failed.".to_string())?;
        let view = promote_memory_record(&mut store, engine, request)?;
        state.save_if_persistent(&store)?;
        Ok(view)
    })?
}

#[tauri::command]
pub fn memory_candidate_suppress(
    state: tauri::State<MemoryBridgeState>,
    request: MemoryCandidateActionRequest,
) -> Result<MemoryStateView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Memory bridge lock failed.".to_string())?;
    let view = suppress_memory_candidate_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(view)
}

#[tauri::command]
pub fn memory_record_suppress(
    state: tauri::State<MemoryBridgeState>,
    request: MemoryRecordActionRequest,
) -> Result<MemoryStateView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Memory bridge lock failed.".to_string())?;
    let view = suppress_memory_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(view)
}

pub fn memory_snapshot_from_path(path: &Path) -> Result<MemoryStateView, String> {
    let store = crate::memory_persistence::load_from_path(path)?;
    Ok(memory_snapshot_from_store(&store))
}

pub fn propose_memory_candidate_record(
    store: &mut MemoryStore,
    request: MemoryCandidateRequest,
) -> Result<MemoryStateView, String> {
    validate_candidate_request(&request)?;
    store.propose_candidate(MemoryCandidateInput {
        key: request.key,
        scope: parse_scope(&request.scope)?,
        source_run_id: request.source_run_id,
        source_thread_id: request.source_thread_id,
        value: request.value,
    });
    Ok(memory_snapshot_from_store(store))
}

pub fn promote_memory_record(
    store: &mut MemoryStore,
    approvals: &ApprovalEngine,
    request: MemoryPromoteRequest,
) -> Result<MemoryStateView, String> {
    if request.candidate_id.trim().is_empty() || request.approval_id.trim().is_empty() {
        return Err("Memory promotion requires candidate and approval IDs.".to_string());
    }
    store
        .promote_approved(
            &request.candidate_id,
            &request.approval_id,
            request.approved_at_ms,
            approvals,
            parse_source_status(&request.source_run_status)?,
        )
        .map_err(|error| format!("{error:?}"))?;
    Ok(memory_snapshot_from_store(store))
}

pub fn suppress_memory_candidate_record(
    store: &mut MemoryStore,
    request: MemoryCandidateActionRequest,
) -> Result<MemoryStateView, String> {
    if request.candidate_id.trim().is_empty() {
        return Err("Memory suppression requires a candidate ID.".to_string());
    }
    store
        .suppress_candidate(&request.candidate_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(memory_snapshot_from_store(store))
}

pub fn suppress_memory_record(
    store: &mut MemoryStore,
    request: MemoryRecordActionRequest,
) -> Result<MemoryStateView, String> {
    if request.record_id.trim().is_empty() {
        return Err("Memory suppression requires a record ID.".to_string());
    }
    store
        .suppress_memory(&request.record_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(memory_snapshot_from_store(store))
}

fn parse_scope(value: &str) -> Result<MemoryScope, String> {
    match value {
        "project" => Ok(MemoryScope::Project),
        "user" => Ok(MemoryScope::User),
        _ => Err("Unsupported memory scope.".to_string()),
    }
}

fn parse_source_status(value: &str) -> Result<SourceRunStatus, String> {
    match value {
        "completed" | "succeeded" => Ok(SourceRunStatus::Completed),
        "failed" => Ok(SourceRunStatus::Failed),
        "running" => Ok(SourceRunStatus::Running),
        _ => Err("Unsupported source run status for memory promotion.".to_string()),
    }
}

fn validate_candidate_request(request: &MemoryCandidateRequest) -> Result<(), String> {
    if request.key.trim().is_empty()
        || request.value.trim().is_empty()
        || request.source_run_id.trim().is_empty()
        || request.source_thread_id.trim().is_empty()
    {
        return Err("Memory candidate requires key, value, run, and thread IDs.".to_string());
    }
    Ok(())
}
