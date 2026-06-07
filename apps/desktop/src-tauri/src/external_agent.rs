use crate::approval::{ApprovalEngine, ApprovalError};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentAvailability {
    pub adapter_id: String,
    pub kind: ExternalAgentKind,
    pub display_name: String,
    pub status: AdapterStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalAgentKind {
    CodexCli,
    ClaudeCode,
    GenericTerminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterStatus {
    Available,
    Missing,
    NotChecked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentScope {
    pub project_root: PathBuf,
    pub checkpoint_id: Option<String>,
    pub worktree_id: Option<String>,
    pub allowed_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentTaskPolicy {
    pub allowed_tools: Vec<String>,
    pub approval_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentRunRequest {
    pub run_id: String,
    pub approval_id: String,
    pub adapter_id: String,
    pub task: String,
    pub scope: ExternalAgentScope,
    pub timeout_ms: u64,
    pub allowed_tools: Vec<String>,
    pub task_policy: ExternalAgentTaskPolicy,
    pub capture_plan: ExternalAgentCapturePlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExternalAgentCapturePlan {
    pub capture_diff: bool,
    pub changed_files: Vec<PathBuf>,
    pub commands: Vec<String>,
    pub test_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentRunArtifact {
    pub id: String,
    pub run_id: String,
    pub adapter_id: String,
    pub status: ExternalAgentRunStatus,
    pub scope: ExternalAgentScope,
    pub transcript: Vec<ExternalAgentEvent>,
    pub terminal_output: String,
    pub diff_summary: Option<String>,
    pub test_artifact_ids: Vec<String>,
    pub review_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalAgentRunStatus {
    Accepted,
    Completed,
    Blocked,
    Reverted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentEvent {
    pub kind: ExternalAgentEventKind,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalAgentEventKind {
    Command,
    Completed,
    Started,
    Stderr,
    Stdout,
    FileChanged,
    DiffCaptured,
    TestResult,
    ReviewDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalAgentReviewDecision {
    Accept,
    Revert,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalAgentError {
    AdapterUnavailable,
    Approval(ApprovalError),
    ArtifactNotFound,
    EmptyTask,
    Io(String),
    MissingIsolation,
    OutsideApprovedRoot,
    ToolNotAllowed(String),
}

#[derive(Debug)]
pub struct ExternalAgentBridge {
    adapters: Vec<ExternalAgentAvailability>,
    approved_roots: Vec<PathBuf>,
    artifacts: Vec<ExternalAgentRunArtifact>,
    next_artifact_id: usize,
}

impl ExternalAgentBridge {
    pub fn new(approved_roots: Vec<PathBuf>) -> Result<Self, ExternalAgentError> {
        let roots = approved_roots
            .iter()
            .map(|root| fs::canonicalize(root).map_err(io_error))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { adapters: default_adapters(), approved_roots: roots, artifacts: Vec::new(), next_artifact_id: 0 })
    }

    pub fn detect_adapters(&self) -> &[ExternalAgentAvailability] {
        &self.adapters
    }

    pub fn run_approved_worker(
        &mut self,
        request: ExternalAgentRunRequest,
        now: u64,
        approvals: &ApprovalEngine,
    ) -> Result<ExternalAgentRunArtifact, ExternalAgentError> {
        if request.task.trim().is_empty() {
            return Err(ExternalAgentError::EmptyTask);
        }
        approvals.assert_can_execute(&request.approval_id, now).map_err(ExternalAgentError::Approval)?;
        self.ensure_available(&request.adapter_id)?;
        self.ensure_task_authority(&request)?;
        let scope = self.checked_scope(request.scope)?;
        if scope.checkpoint_id.is_none() && scope.worktree_id.is_none() {
            return Err(ExternalAgentError::MissingIsolation);
        }
        let mut transcript = vec![
            event(ExternalAgentEventKind::Started, "External worker started inside approved scope.", now),
            event(ExternalAgentEventKind::Stdout, &format!("Task: {}", request.task), now),
            event(ExternalAgentEventKind::Command, "prototype worker command captured", now),
        ];
        for command in &request.capture_plan.commands {
            transcript.push(event(ExternalAgentEventKind::Command, command, now));
        }
        for path in &request.capture_plan.changed_files {
            transcript.push(event(ExternalAgentEventKind::FileChanged, &path.display().to_string(), now));
        }
        let diff_summary = request.capture_plan.capture_diff.then(|| "Diff capture requested for Delyx review.".to_string());
        if diff_summary.is_some() {
            transcript.push(event(ExternalAgentEventKind::DiffCaptured, "Diff artifact must be reviewed by Delyx UI.", now));
        }
        for artifact_id in &request.capture_plan.test_artifact_ids {
            transcript.push(event(ExternalAgentEventKind::TestResult, artifact_id, now));
        }
        transcript.push(event(ExternalAgentEventKind::Completed, "External worker completed.", now));

        self.next_artifact_id += 1;
        let artifact = ExternalAgentRunArtifact {
            id: format!("external-agent-{}", self.next_artifact_id),
            run_id: request.run_id,
            adapter_id: request.adapter_id,
            status: ExternalAgentRunStatus::Completed,
            scope,
            transcript,
            terminal_output: "prototype external agent bridge completed without spawning a worker".to_string(),
            review_required: diff_summary.is_some(),
            diff_summary,
            test_artifact_ids: request.capture_plan.test_artifact_ids,
        };
        self.artifacts.push(artifact.clone());
        Ok(artifact)
    }

    pub fn list_artifacts(&self, run_id: &str) -> Vec<&ExternalAgentRunArtifact> {
        self.artifacts.iter().filter(|artifact| artifact.run_id == run_id).collect()
    }

    pub fn record_review_decision(
        &mut self,
        artifact_id: &str,
        decision: ExternalAgentReviewDecision,
        timestamp: u64,
    ) -> Result<ExternalAgentRunArtifact, ExternalAgentError> {
        let artifact = self.artifacts.iter_mut().find(|artifact| artifact.id == artifact_id).ok_or(ExternalAgentError::ArtifactNotFound)?;
        let (status, message) = match decision {
            ExternalAgentReviewDecision::Accept => (ExternalAgentRunStatus::Accepted, "External worker diff accepted."),
            ExternalAgentReviewDecision::Revert => (ExternalAgentRunStatus::Reverted, "External worker diff reverted."),
        };
        artifact.status = status;
        artifact.transcript.push(event(ExternalAgentEventKind::ReviewDecision, message, timestamp));
        Ok(artifact.clone())
    }

    fn ensure_available(&self, adapter_id: &str) -> Result<(), ExternalAgentError> {
        self.adapters
            .iter()
            .find(|adapter| adapter.adapter_id == adapter_id && adapter.status == AdapterStatus::Available)
            .map(|_| ())
            .ok_or(ExternalAgentError::AdapterUnavailable)
    }

    fn ensure_task_authority(&self, request: &ExternalAgentRunRequest) -> Result<(), ExternalAgentError> {
        for tool in &request.allowed_tools {
            if !request.task_policy.allowed_tools.contains(tool) {
                return Err(ExternalAgentError::ToolNotAllowed(tool.clone()));
            }
        }
        Ok(())
    }

    fn checked_scope(&self, scope: ExternalAgentScope) -> Result<ExternalAgentScope, ExternalAgentError> {
        let root = checked_path(&scope.project_root, &self.approved_roots)?;
        let allowed_paths = scope
            .allowed_paths
            .iter()
            .map(|path| checked_path(path, &self.approved_roots))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ExternalAgentScope { project_root: root, checkpoint_id: scope.checkpoint_id, worktree_id: scope.worktree_id, allowed_paths })
    }
}

pub fn tests_are_trusted(artifact: &ExternalAgentRunArtifact) -> bool {
    !artifact.test_artifact_ids.is_empty()
}

fn default_adapters() -> Vec<ExternalAgentAvailability> {
    vec![
        adapter("codex-cli", ExternalAgentKind::CodexCli, "Codex CLI", AdapterStatus::Missing, "Adapter placeholder; executable not detected."),
        adapter("claude-code", ExternalAgentKind::ClaudeCode, "Claude Code", AdapterStatus::Missing, "Adapter placeholder; executable not detected."),
        adapter("generic-terminal", ExternalAgentKind::GenericTerminal, "Generic terminal agent", AdapterStatus::Available, "Prototype adapter available; no process spawn yet."),
    ]
}

fn adapter(id: &str, kind: ExternalAgentKind, display_name: &str, status: AdapterStatus, detail: &str) -> ExternalAgentAvailability {
    ExternalAgentAvailability { adapter_id: id.to_string(), kind, display_name: display_name.to_string(), status, detail: detail.to_string() }
}

fn checked_path(path: &Path, approved_roots: &[PathBuf]) -> Result<PathBuf, ExternalAgentError> {
    let normalized = fs::canonicalize(path).map_err(io_error)?;
    approved_roots
        .iter()
        .any(|root| normalized.starts_with(root))
        .then_some(normalized)
        .ok_or(ExternalAgentError::OutsideApprovedRoot)
}

fn event(kind: ExternalAgentEventKind, message: &str, timestamp: u64) -> ExternalAgentEvent {
    ExternalAgentEvent { kind, message: message.to_string(), timestamp }
}

fn io_error(error: std::io::Error) -> ExternalAgentError {
    ExternalAgentError::Io(error.to_string())
}
