use crate::agent_run::AgentRun;
use crate::threads::{MessageRole, TaskThread, ThreadStatus};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadRunViewContext {
    pub created_at: String,
    pub project_id: String,
    pub run_id: String,
    pub updated_at: String,
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

pub fn record_view(
    thread: &TaskThread,
    run: &AgentRun,
    context: &ThreadRunViewContext,
) -> ThreadRunRecordView {
    ThreadRunRecordView {
        run: run_view(run, thread, context),
        thread: thread_view(thread, context),
    }
}

pub fn thread_view(thread: &TaskThread, context: &ThreadRunViewContext) -> TaskThreadView {
    TaskThreadView {
        active_run_id: Some(context.run_id.clone()),
        archived: thread.archived,
        created_at: context.created_at.clone(),
        created_label: "Now".to_string(),
        goal: thread.goal.clone(),
        id: thread.id.clone(),
        messages: thread.messages.iter().map(message_view).collect(),
        mode: mode_key(thread.status).to_string(),
        project_id: context.project_id.clone(),
        run_ids: vec![context.run_id.clone()],
        status: status_key(thread.status).to_string(),
        title: thread.title.clone(),
        updated_at: context.updated_at.clone(),
    }
}

pub fn run_view(
    run: &AgentRun,
    thread: &TaskThread,
    context: &ThreadRunViewContext,
) -> AgentRunView {
    AgentRunView {
        artifacts: Vec::new(),
        created_at: context.created_at.clone(),
        events: run
            .events
            .iter()
            .map(|event| AgentEventView {
                created_at: context.created_at.clone(),
                id: event.id.clone(),
                kind: event.kind.clone(),
                message: event.message.clone(),
                run_id: run.id.clone(),
            })
            .collect(),
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
        project_id: context.project_id.clone(),
        status: run_status_key(thread.status).to_string(),
        thread_id: thread.id.clone(),
        updated_at: context.updated_at.clone(),
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
        ThreadStatus::Blocked
        | ThreadStatus::Done
        | ThreadStatus::Failed
        | ThreadStatus::Reviewing => "review",
        ThreadStatus::Building => "build",
        ThreadStatus::Idle | ThreadStatus::Exploring => "explore",
        ThreadStatus::Planning | ThreadStatus::WaitingForApproval => "plan",
        ThreadStatus::Testing => "test",
    }
}

fn run_status_key(status: ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::Blocked => "blocked",
        ThreadStatus::Done => "succeeded",
        ThreadStatus::Failed => "failed",
        ThreadStatus::Idle => "created",
        ThreadStatus::WaitingForApproval => "waiting_for_approval",
        _ => "running",
    }
}
