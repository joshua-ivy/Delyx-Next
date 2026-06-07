use crate::approval::ApprovalError;
use crate::external_agent_terminal::ExternalAgentCommand;
use std::path::PathBuf;

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
    pub terminal_approval_id: Option<String>,
    pub adapter_id: String,
    pub task: String,
    pub scope: ExternalAgentScope,
    pub requires_isolation: bool,
    pub timeout_ms: u64,
    pub allowed_tools: Vec<String>,
    pub task_policy: ExternalAgentTaskPolicy,
    pub capture_plan: ExternalAgentCapturePlan,
    pub worker_command: Option<ExternalAgentCommand>,
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
    Failed,
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
    Failed,
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
    EmptyCommand,
    EmptyTask,
    Io(String),
    MissingTerminalApproval,
    MissingIsolation,
    OutsideApprovedRoot,
    Timeout,
    ToolNotAllowed(String),
}
