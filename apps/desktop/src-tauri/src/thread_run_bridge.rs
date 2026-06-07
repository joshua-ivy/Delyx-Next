use crate::agent_run::{AgentRun, AgentRunLedger};
use crate::threads::{MessageRole, TaskThread, ThreadManager, ThreadStatus};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThreadRunRecord {
    thread_id: String,
    run_id: String,
    project_id: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadRunSnapshotView {
    pub threads: Vec<TaskThreadView>,
    pub runs: Vec<AgentRunView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadRunRecordView {
    pub thread: TaskThreadView,
    pub run: AgentRunView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskThreadView {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub goal: String,
    pub status: String,
    pub mode: String,
    pub active_run_id: Option<String>,
    pub run_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_label: String,
    pub messages: Vec<ThreadMessageView>,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMessageView {
    pub role: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunView {
    pub id: String,
    pub project_id: String,
    pub thread_id: String,
    pub goal: String,
    pub mode: String,
    pub status: String,
    pub nodes: Vec<serde_json::Value>,
    pub events: Vec<AgentEventView>,
    pub artifacts: Vec<serde_json::Value>,
    pub evidence: Vec<serde_json::Value>,
    pub metrics: RunMetricsView,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEventView {
    pub id: String,
    pub run_id: String,
    pub kind: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunMetricsView {
    pub event_count: usize,
    pub node_count: usize,
    pub artifact_count: usize,
    pub evidence_count: usize,
    pub command_count: usize,
    pub approval_count: usize,
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
    Ok(record_view(&thread, &run, &record))
}

pub fn thread_run_snapshot_from_store(store: &ThreadRunStore, project_id: &str) -> ThreadRunSnapshotView {
    let mut runs = Vec::new();
    let threads = store.manager
        .list_threads(project_id, false)
        .into_iter()
        .filter_map(|thread| {
            let record = store.records.iter().find(|item| item.thread_id == thread.id)?;
            let run = store.ledger.get_run(&record.run_id).ok()?;
            runs.push(run_view(run, thread, record));
            Some(thread_view(thread, record))
        })
        .collect();
    ThreadRunSnapshotView { runs, threads }
}

fn record_view(thread: &TaskThread, run: &AgentRun, record: &ThreadRunRecord) -> ThreadRunRecordView {
    ThreadRunRecordView {
        run: run_view(run, thread, record),
        thread: thread_view(thread, record),
    }
}

fn thread_view(thread: &TaskThread, record: &ThreadRunRecord) -> TaskThreadView {
    TaskThreadView {
        active_run_id: Some(record.run_id.clone()),
        archived: thread.archived,
        created_at: record.created_at.clone(),
        created_label: "Now".to_string(),
        goal: thread.goal.clone(),
        id: thread.id.clone(),
        messages: thread.messages.iter().map(message_view).collect(),
        mode: mode_key(thread.status).to_string(),
        project_id: record.project_id.clone(),
        run_ids: vec![record.run_id.clone()],
        status: status_key(thread.status).to_string(),
        title: thread.title.clone(),
        updated_at: record.updated_at.clone(),
    }
}

fn run_view(run: &AgentRun, thread: &TaskThread, record: &ThreadRunRecord) -> AgentRunView {
    AgentRunView {
        artifacts: Vec::new(),
        created_at: record.created_at.clone(),
        events: run.events.iter().map(|event| AgentEventView {
            created_at: record.created_at.clone(),
            id: event.id.clone(),
            kind: event.kind.clone(),
            message: event.message.clone(),
            run_id: run.id.clone(),
        }).collect(),
        evidence: Vec::new(),
        goal: thread.goal.clone(),
        id: run.id.clone(),
        metrics: RunMetricsView {
            approval_count: 0,
            artifact_count: run.metrics.artifact_count,
            command_count: 0,
            event_count: run.metrics.event_count,
            evidence_count: run.metrics.evidence_count,
            node_count: run.nodes.len(),
        },
        mode: mode_key(thread.status).to_string(),
        nodes: Vec::new(),
        project_id: record.project_id.clone(),
        status: "created".to_string(),
        thread_id: thread.id.clone(),
        updated_at: record.updated_at.clone(),
    }
}

fn message_view(message: &crate::threads::ThreadMessage) -> ThreadMessageView {
    ThreadMessageView {
        body: message.body.clone(),
        role: role_key(message.role).to_string(),
    }
}

fn role_key(role: MessageRole) -> &'static str {
    match role {
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::User => "user",
    }
}

fn status_key(status: ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::Blocked => "blocked",
        ThreadStatus::Building => "building",
        ThreadStatus::Done => "done",
        ThreadStatus::Exploring => "exploring",
        ThreadStatus::Failed => "failed",
        ThreadStatus::Idle => "idle",
        ThreadStatus::Planning => "planning",
        ThreadStatus::Reviewing => "reviewing",
        ThreadStatus::Testing => "testing",
        ThreadStatus::WaitingForApproval => "waiting_for_approval",
    }
}

fn mode_key(status: ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::Blocked | ThreadStatus::Done | ThreadStatus::Failed | ThreadStatus::Reviewing => "review",
        ThreadStatus::Building => "build",
        ThreadStatus::Idle | ThreadStatus::Exploring => "explore",
        ThreadStatus::Planning | ThreadStatus::WaitingForApproval => "plan",
        ThreadStatus::Testing => "test",
    }
}
