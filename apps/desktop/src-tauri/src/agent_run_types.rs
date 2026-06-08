#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRun {
    pub id: String,
    pub thread_id: String,
    pub status: AgentRunStatus,
    pub nodes: Vec<AgentNode>,
    pub events: Vec<AgentEvent>,
    pub artifacts: Vec<Artifact>,
    pub evidence: Vec<EvidenceRecord>,
    pub metrics: RunMetrics,
    pub outcome: Option<AgentOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunStatus {
    Running,
    WaitingForApproval,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub status: AgentRunStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentEvent {
    pub id: String,
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub id: String,
    pub source_kind: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunMetrics {
    pub event_count: usize,
    pub artifact_count: usize,
    pub evidence_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentOutcome {
    pub status: AgentRunStatus,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentRunError {
    EmptyThread,
    InvalidTransition,
    RunNotFound,
    TerminalRun,
    Io(String),
    InvalidLedger(String),
}
