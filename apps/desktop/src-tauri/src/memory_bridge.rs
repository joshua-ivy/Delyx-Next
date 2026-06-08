use crate::local_store_bridge::LocalStoreBridgeState;
use crate::memory::{
    MemoryCandidate, MemoryCandidateStatus, MemoryRecord, MemoryScope, MemoryStore,
};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStateView {
    pub candidates: Vec<MemoryCandidateView>,
    pub records: Vec<MemoryRecordView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCandidateView {
    pub id: String,
    pub scope: String,
    pub key: String,
    pub value: String,
    pub source_run_id: String,
    pub source_thread_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecordView {
    pub id: String,
    pub scope: String,
    pub key: String,
    pub value: String,
    pub source_run_id: String,
    pub source_thread_id: String,
    pub supersedes: Option<String>,
    pub suppressed: bool,
}

#[tauri::command]
pub fn memory_snapshot(
    state: tauri::State<LocalStoreBridgeState>,
) -> Result<MemoryStateView, String> {
    memory_snapshot_from_path(state.database_path())
}

pub fn memory_snapshot_from_path(path: &Path) -> Result<MemoryStateView, String> {
    let store = crate::memory_persistence::load_from_path(path)?;
    Ok(memory_snapshot_from_store(&store))
}

pub fn memory_snapshot_from_store(store: &MemoryStore) -> MemoryStateView {
    MemoryStateView {
        candidates: store.candidates().iter().map(candidate_view).collect(),
        records: store.records().iter().map(record_view).collect(),
    }
}

fn candidate_view(candidate: &MemoryCandidate) -> MemoryCandidateView {
    MemoryCandidateView {
        id: candidate.id.clone(),
        scope: scope_key(candidate.scope).to_string(),
        key: candidate.key.clone(),
        value: candidate.value.clone(),
        source_run_id: candidate.source_run_id.clone(),
        source_thread_id: candidate.source_thread_id.clone(),
        status: status_key(candidate.status).to_string(),
    }
}

fn record_view(record: &MemoryRecord) -> MemoryRecordView {
    MemoryRecordView {
        id: record.id.clone(),
        scope: scope_key(record.scope).to_string(),
        key: record.key.clone(),
        value: record.value.clone(),
        source_run_id: record.source_run_id.clone(),
        source_thread_id: record.source_thread_id.clone(),
        supersedes: record.supersedes.clone(),
        suppressed: record.suppressed,
    }
}

fn scope_key(scope: MemoryScope) -> &'static str {
    match scope {
        MemoryScope::Project => "project",
        MemoryScope::User => "user",
    }
}

fn status_key(status: MemoryCandidateStatus) -> &'static str {
    match status {
        MemoryCandidateStatus::Pending => "pending",
        MemoryCandidateStatus::Promoted => "promoted",
        MemoryCandidateStatus::Suppressed => "suppressed",
    }
}
