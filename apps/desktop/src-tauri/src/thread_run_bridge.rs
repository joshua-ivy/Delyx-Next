use crate::agent_run::AgentRunLedger;
use crate::thread_run_bridge_views::{
    record_view, run_view, thread_view, TaskThreadView, ThreadRunRecordView,
    ThreadRunSnapshotView, ThreadRunViewContext,
};
use crate::threads::{MessageRole, ThreadManager, ThreadStatus};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Default)]
pub struct ThreadRunBridgeState {
    store: Mutex<ThreadRunStore>,
}

#[derive(Default)]
pub struct ThreadRunStore {
    manager: ThreadManager,
    ledger: AgentRunLedger,
    records: Vec<ThreadRunRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadRunCreateRequest {
    pub project_id: String,
    pub goal: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadStatusUpdateRequest {
    pub thread_id: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadArchiveRequest {
    pub thread_id: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMessageAppendRequest {
    pub thread_id: String,
    pub role: String,
    pub body: String,
    pub status: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThreadRunRecord {
    thread_id: String,
    run_id: String,
    project_id: String,
    created_at: String,
    updated_at: String,
}

#[tauri::command]
pub fn thread_run_create(
    state: tauri::State<ThreadRunBridgeState>,
    request: ThreadRunCreateRequest,
) -> Result<ThreadRunRecordView, String> {
    let mut store = state.store.lock().map_err(|_| "Thread bridge lock failed.".to_string())?;
    create_thread_run_record(&mut store, request).map_err(|error| format!("{error:?}"))
}

#[tauri::command]
pub fn thread_run_snapshot(
    state: tauri::State<ThreadRunBridgeState>,
    project_id: String,
) -> Result<ThreadRunSnapshotView, String> {
    let store = state.store.lock().map_err(|_| "Thread bridge lock failed.".to_string())?;
    Ok(thread_run_snapshot_from_store(&store, &project_id))
}

#[tauri::command]
pub fn thread_status_update(
    state: tauri::State<ThreadRunBridgeState>,
    request: ThreadStatusUpdateRequest,
) -> Result<TaskThreadView, String> {
    let mut store = state.store.lock().map_err(|_| "Thread bridge lock failed.".to_string())?;
    update_thread_status_record(&mut store, request).map_err(|error| format!("{error:?}"))
}

#[tauri::command]
pub fn thread_archive(
    state: tauri::State<ThreadRunBridgeState>,
    request: ThreadArchiveRequest,
) -> Result<TaskThreadView, String> {
    let mut store = state.store.lock().map_err(|_| "Thread bridge lock failed.".to_string())?;
    archive_thread_record(&mut store, request).map_err(|error| format!("{error:?}"))
}

#[tauri::command]
pub fn thread_message_append(
    state: tauri::State<ThreadRunBridgeState>,
    request: ThreadMessageAppendRequest,
) -> Result<TaskThreadView, String> {
    let mut store = state.store.lock().map_err(|_| "Thread bridge lock failed.".to_string())?;
    append_thread_message_record(&mut store, request).map_err(|error| format!("{error:?}"))
}

pub fn create_thread_run_record(
    store: &mut ThreadRunStore,
    request: ThreadRunCreateRequest,
) -> Result<ThreadRunRecordView, crate::threads::ThreadError> {
    store.manager.link_project(request.project_id.clone());
    let thread = store.manager.create_thread(&request.project_id, &request.goal)?;
    let run = store.ledger.create_run(&thread.id).map_err(|_| crate::threads::ThreadError::InvalidTransition)?;
    let run_id = run.id.clone();
    let _ = store.ledger.append_event(&run_id, "thread.created", "Thread created from user goal.");
    let record = ThreadRunRecord {
        created_at: request.created_at.clone(),
        project_id: request.project_id,
        run_id,
        thread_id: thread.id.clone(),
        updated_at: request.created_at,
    };
    store.records.push(record.clone());
    let run = store.ledger.get_run(&record.run_id).map_err(|_| crate::threads::ThreadError::InvalidTransition)?;
    Ok(record_view(&thread, &run, &view_context(&record)))
}

pub fn update_thread_status_record(
    store: &mut ThreadRunStore,
    request: ThreadStatusUpdateRequest,
) -> Result<TaskThreadView, crate::threads::ThreadError> {
    let status = parse_thread_status(&request.status)?;
    store.manager.set_status(&request.thread_id, status)?;
    let context = view_context(update_record_timestamp(store, &request.thread_id, &request.updated_at)?);
    Ok(thread_view(store.manager.get_thread(&request.thread_id)?, &context))
}

pub fn archive_thread_record(
    store: &mut ThreadRunStore,
    request: ThreadArchiveRequest,
) -> Result<TaskThreadView, crate::threads::ThreadError> {
    store.manager.archive_thread(&request.thread_id)?;
    let context = view_context(update_record_timestamp(store, &request.thread_id, &request.updated_at)?);
    Ok(thread_view(store.manager.get_thread(&request.thread_id)?, &context))
}

pub fn append_thread_message_record(
    store: &mut ThreadRunStore,
    request: ThreadMessageAppendRequest,
) -> Result<TaskThreadView, crate::threads::ThreadError> {
    let role = parse_message_role(&request.role)?;
    let body = request.body.trim().to_string();
    if body.is_empty() {
        return Err(crate::threads::ThreadError::InvalidTransition);
    }
    let status = request.status.as_deref().map(parse_thread_status).transpose()?;
    if !store.records.iter().any(|item| item.thread_id == request.thread_id) {
        return Err(crate::threads::ThreadError::ThreadNotFound);
    }
    if let Some(status) = status {
        store.manager.set_status(&request.thread_id, status)?;
    }
    store.manager.append_message(&request.thread_id, role, &body)?;
    let context = view_context(update_record_timestamp(store, &request.thread_id, &request.updated_at)?);
    Ok(thread_view(store.manager.get_thread(&request.thread_id)?, &context))
}

pub fn thread_run_snapshot_from_store(store: &ThreadRunStore, project_id: &str) -> ThreadRunSnapshotView {
    let mut runs = Vec::new();
    let threads = store.manager
        .list_threads(project_id, false)
        .into_iter()
        .filter_map(|thread| {
            let record = store.records.iter().find(|item| item.thread_id == thread.id)?;
            let run = store.ledger.get_run(&record.run_id).ok()?;
            let context = view_context(record);
            runs.push(run_view(run, thread, &context));
            Some(thread_view(thread, &context))
        })
        .collect();
    ThreadRunSnapshotView { runs, threads }
}

fn view_context(record: &ThreadRunRecord) -> ThreadRunViewContext {
    ThreadRunViewContext {
        created_at: record.created_at.clone(),
        project_id: record.project_id.clone(),
        run_id: record.run_id.clone(),
        updated_at: record.updated_at.clone(),
    }
}

fn update_record_timestamp<'a>(
    store: &'a mut ThreadRunStore,
    thread_id: &str,
    updated_at: &str,
) -> Result<&'a ThreadRunRecord, crate::threads::ThreadError> {
    let record = store
        .records
        .iter_mut()
        .find(|item| item.thread_id == thread_id)
        .ok_or(crate::threads::ThreadError::ThreadNotFound)?;
    record.updated_at = updated_at.to_string();
    Ok(record)
}

fn parse_thread_status(status: &str) -> Result<ThreadStatus, crate::threads::ThreadError> {
    match status {
        "blocked" => Ok(ThreadStatus::Blocked),
        "building" => Ok(ThreadStatus::Building),
        "done" => Ok(ThreadStatus::Done),
        "exploring" => Ok(ThreadStatus::Exploring),
        "failed" => Ok(ThreadStatus::Failed),
        "idle" => Ok(ThreadStatus::Idle),
        "planning" => Ok(ThreadStatus::Planning),
        "reviewing" => Ok(ThreadStatus::Reviewing),
        "testing" => Ok(ThreadStatus::Testing),
        "waiting_for_approval" => Ok(ThreadStatus::WaitingForApproval),
        _ => Err(crate::threads::ThreadError::InvalidTransition),
    }
}

fn parse_message_role(role: &str) -> Result<MessageRole, crate::threads::ThreadError> {
    match role {
        "assistant" => Ok(MessageRole::Assistant),
        "system" => Ok(MessageRole::System),
        "user" => Ok(MessageRole::User),
        _ => Err(crate::threads::ThreadError::InvalidTransition),
    }
}
