use std::path::Path;

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

#[derive(Debug, Default)]
pub struct AgentRunLedger {
    pub(crate) runs: Vec<AgentRun>,
    pub(crate) next_run: usize,
    next_event: usize,
    next_node: usize,
}

impl AgentRunLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_run(&mut self, thread_id: &str) -> Result<AgentRun, AgentRunError> {
        if thread_id.trim().is_empty() {
            return Err(AgentRunError::EmptyThread);
        }

        self.next_run += 1;
        let run = AgentRun {
            id: format!("run-{}", self.next_run),
            thread_id: thread_id.to_string(),
            status: AgentRunStatus::Running,
            nodes: Vec::new(),
            events: Vec::new(),
            artifacts: Vec::new(),
            evidence: Vec::new(),
            metrics: RunMetrics::default(),
            outcome: None,
        };
        self.runs.push(run.clone());
        Ok(run)
    }

    pub fn list_runs(&self, thread_id: &str) -> Vec<&AgentRun> {
        self.runs.iter().filter(|run| run.thread_id == thread_id).collect()
    }

    pub fn get_run(&self, run_id: &str) -> Result<&AgentRun, AgentRunError> {
        self.runs.iter().find(|run| run.id == run_id).ok_or(AgentRunError::RunNotFound)
    }

    pub fn append_event(&mut self, run_id: &str, kind: &str, message: &str) -> Result<AgentEvent, AgentRunError> {
        let event_id = self.allocate_event_id();
        let run = self.run_mut(run_id)?;
        ensure_running(run)?;
        let event = AgentEvent { id: event_id, kind: kind.to_string(), message: message.to_string() };
        run.events.push(event.clone());
        run.metrics.event_count = run.events.len();
        Ok(event)
    }

    pub fn append_node(&mut self, run_id: &str, kind: &str, label: &str) -> Result<AgentNode, AgentRunError> {
        let node_id = self.allocate_node_id();
        let run = self.run_mut(run_id)?;
        ensure_running(run)?;
        let node = AgentNode {
            id: node_id,
            kind: kind.to_string(),
            label: label.to_string(),
            status: AgentRunStatus::Running,
        };
        run.nodes.push(node.clone());
        Ok(node)
    }

    pub fn record_artifact(&mut self, run_id: &str, kind: &str, label: &str) -> Result<Artifact, AgentRunError> {
        let run = self.run_mut(run_id)?;
        ensure_running(run)?;
        let artifact = Artifact {
            id: format!("artifact-{}", run.artifacts.len() + 1),
            kind: kind.to_string(),
            label: label.to_string(),
        };
        run.artifacts.push(artifact.clone());
        run.metrics.artifact_count = run.artifacts.len();
        Ok(artifact)
    }

    pub fn record_evidence(&mut self, run_id: &str, source_kind: &str, title: &str) -> Result<EvidenceRecord, AgentRunError> {
        let run = self.run_mut(run_id)?;
        ensure_running(run)?;
        let evidence = EvidenceRecord {
            id: format!("evidence-{}", run.evidence.len() + 1),
            source_kind: source_kind.to_string(),
            title: title.to_string(),
        };
        run.evidence.push(evidence.clone());
        run.metrics.evidence_count = run.evidence.len();
        Ok(evidence)
    }

    pub fn wait_for_approval(&mut self, run_id: &str, approval_id: &str) -> Result<AgentEvent, AgentRunError> {
        let event_id = self.allocate_event_id();
        let run = self.run_mut(run_id)?;
        ensure_running(run)?;
        run.status = AgentRunStatus::WaitingForApproval;
        let event = AgentEvent {
            id: event_id,
            kind: "approval.waiting".to_string(),
            message: format!("Waiting for approval {approval_id}."),
        };
        run.events.push(event.clone());
        run.metrics.event_count = run.events.len();
        Ok(event)
    }

    pub fn resume_after_approval(&mut self, run_id: &str, approval_id: &str) -> Result<AgentEvent, AgentRunError> {
        let event_id = self.allocate_event_id();
        let run = self.run_mut(run_id)?;
        if run.status != AgentRunStatus::WaitingForApproval {
            return Err(AgentRunError::InvalidTransition);
        }
        run.status = AgentRunStatus::Running;
        let event = AgentEvent {
            id: event_id,
            kind: "approval.resumed".to_string(),
            message: format!("Resumed after approval {approval_id}."),
        };
        run.events.push(event.clone());
        run.metrics.event_count = run.events.len();
        Ok(event)
    }

    pub fn complete_run(&mut self, run_id: &str, summary: &str) -> Result<(), AgentRunError> {
        self.finish_run(run_id, AgentRunStatus::Completed, summary)
    }

    pub fn fail_run(&mut self, run_id: &str, summary: &str) -> Result<(), AgentRunError> {
        self.finish_run(run_id, AgentRunStatus::Failed, summary)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), AgentRunError> {
        crate::agent_run_persistence::save_to_path(self, path)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, AgentRunError> {
        crate::agent_run_persistence::load_from_path(path)
    }

    fn finish_run(&mut self, run_id: &str, status: AgentRunStatus, summary: &str) -> Result<(), AgentRunError> {
        let run = self.run_mut(run_id)?;
        ensure_running(run)?;
        run.status = status;
        run.outcome = Some(AgentOutcome { status, summary: summary.to_string() });
        Ok(())
    }

    pub(crate) fn run_mut(&mut self, run_id: &str) -> Result<&mut AgentRun, AgentRunError> {
        self.runs.iter_mut().find(|run| run.id == run_id).ok_or(AgentRunError::RunNotFound)
    }

    pub(crate) fn refresh_loaded_counters(&mut self) {
        self.next_run = self.runs.iter().filter_map(|run| numeric_suffix(&run.id, "run-")).max().unwrap_or(self.runs.len());
        self.next_event = self
            .runs
            .iter()
            .flat_map(|run| run.events.iter())
            .filter_map(|event| numeric_suffix(&event.id, "event-"))
            .max()
            .unwrap_or(0);
        self.next_node = self
            .runs
            .iter()
            .flat_map(|run| run.nodes.iter())
            .filter_map(|node| numeric_suffix(&node.id, "node-"))
            .max()
            .unwrap_or(0);
    }

    fn allocate_event_id(&mut self) -> String {
        self.next_event += 1;
        format!("event-{}", self.next_event)
    }

    fn allocate_node_id(&mut self) -> String {
        self.next_node += 1;
        format!("node-{}", self.next_node)
    }

}

pub fn create_agent_run(ledger: &mut AgentRunLedger, thread_id: &str) -> Result<AgentRun, AgentRunError> { ledger.create_run(thread_id) }

pub fn list_agent_runs<'a>(ledger: &'a AgentRunLedger, thread_id: &str) -> Vec<&'a AgentRun> { ledger.list_runs(thread_id) }

pub fn get_agent_run<'a>(ledger: &'a AgentRunLedger, run_id: &str) -> Result<&'a AgentRun, AgentRunError> { ledger.get_run(run_id) }

pub fn append_agent_event(ledger: &mut AgentRunLedger, run_id: &str, kind: &str, message: &str) -> Result<AgentEvent, AgentRunError> { ledger.append_event(run_id, kind, message) }

fn ensure_running(run: &AgentRun) -> Result<(), AgentRunError> {
    (run.status == AgentRunStatus::Running).then_some(()).ok_or(AgentRunError::TerminalRun)
}

fn numeric_suffix(value: &str, prefix: &str) -> Option<usize> {
    value.strip_prefix(prefix)?.parse().ok()
}
