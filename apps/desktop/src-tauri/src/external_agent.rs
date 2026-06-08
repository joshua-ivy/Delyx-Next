use crate::approval::{ApprovalEngine, RiskyAction};
use crate::external_agent_adapters::default_adapters;
use crate::external_agent_scope::{checked_approved_path, checked_scoped_path};
use crate::external_agent_terminal::run_worker_command;
pub use crate::external_agent_types::{
    AdapterStatus, ExternalAgentAvailability, ExternalAgentCapturePlan, ExternalAgentError,
    ExternalAgentEvent, ExternalAgentEventKind, ExternalAgentKind, ExternalAgentReviewDecision,
    ExternalAgentRunArtifact, ExternalAgentRunRequest, ExternalAgentRunStatus, ExternalAgentScope,
    ExternalAgentTaskPolicy,
};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ExternalAgentBridge {
    adapters: Vec<ExternalAgentAvailability>,
    approved_roots: Vec<PathBuf>,
    artifacts: Vec<ExternalAgentRunArtifact>,
    next_artifact_id: usize,
}

impl ExternalAgentBridge {
    pub fn new(approved_roots: Vec<PathBuf>) -> Result<Self, ExternalAgentError> {
        Self::with_adapters(approved_roots, default_adapters())
    }

    pub fn with_adapters(
        approved_roots: Vec<PathBuf>,
        adapters: Vec<ExternalAgentAvailability>,
    ) -> Result<Self, ExternalAgentError> {
        let roots = approved_roots
            .iter()
            .map(|root| fs::canonicalize(root).map_err(io_error))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            adapters,
            approved_roots: roots,
            artifacts: Vec::new(),
            next_artifact_id: 0,
        })
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
        approvals
            .assert_can_execute_action_for_run(
                &request.approval_id,
                now,
                RiskyAction::ExternalAgentExecution,
                &request.run_id,
            )
            .map_err(ExternalAgentError::Approval)?;
        if request.worker_command.is_some() {
            let terminal_approval_id = request
                .terminal_approval_id
                .as_deref()
                .filter(|id| !id.trim().is_empty())
                .ok_or(ExternalAgentError::MissingTerminalApproval)?;
            approvals
                .assert_can_execute_action_for_run(
                    terminal_approval_id,
                    now,
                    RiskyAction::TerminalCommand,
                    &request.run_id,
                )
                .map_err(ExternalAgentError::Approval)?;
        }
        self.ensure_available(&request.adapter_id)?;
        self.ensure_task_authority(&request)?;
        let scope = self.checked_scope(request.scope.clone())?;
        if request.requires_isolation
            && scope.checkpoint_id.is_none()
            && scope.worktree_id.is_none()
        {
            return Err(ExternalAgentError::MissingIsolation);
        }
        let changed_files = request
            .capture_plan
            .changed_files
            .iter()
            .map(|path| checked_scoped_path(path, &scope))
            .collect::<Result<Vec<_>, _>>()?;
        let worker = run_worker_command(&request, &scope, now)?;
        let mut transcript = vec![
            event(
                ExternalAgentEventKind::Started,
                "External worker started inside approved scope.",
                now,
            ),
            event(
                ExternalAgentEventKind::Stdout,
                &format!("Task: {}", request.task),
                now,
            ),
        ];
        if request.worker_command.is_none() {
            transcript.push(event(
                ExternalAgentEventKind::Command,
                "prototype worker command captured",
                now,
            ));
        }
        transcript.extend(worker.transcript);
        for command in &request.capture_plan.commands {
            transcript.push(event(ExternalAgentEventKind::Command, command, now));
        }
        for path in &changed_files {
            transcript.push(event(
                ExternalAgentEventKind::FileChanged,
                &path.display().to_string(),
                now,
            ));
        }
        let diff_summary = request.capture_plan.capture_diff.then(|| {
            if changed_files.is_empty() {
                "Diff capture requested for Delyx review; no changed files were reported by the worker.".to_string()
            } else {
                format!("Diff capture requested for Delyx review across {} changed file(s).", changed_files.len())
            }
        });
        if diff_summary.is_some() {
            transcript.push(event(
                ExternalAgentEventKind::DiffCaptured,
                "Diff artifact must be reviewed by Delyx UI.",
                now,
            ));
        }
        for artifact_id in &request.capture_plan.test_artifact_ids {
            transcript.push(event(ExternalAgentEventKind::TestResult, artifact_id, now));
        }
        let final_event = if worker.status == ExternalAgentRunStatus::Failed {
            event(
                ExternalAgentEventKind::Failed,
                "External worker command failed.",
                now,
            )
        } else {
            event(
                ExternalAgentEventKind::Completed,
                "External worker completed.",
                now,
            )
        };
        transcript.push(final_event);

        self.next_artifact_id += 1;
        let artifact = ExternalAgentRunArtifact {
            id: format!("external-agent-{}", self.next_artifact_id),
            run_id: request.run_id,
            adapter_id: request.adapter_id,
            status: worker.status,
            scope,
            transcript,
            terminal_output: worker.terminal_output.unwrap_or_else(|| {
                "prototype external agent bridge completed without spawning a worker".to_string()
            }),
            review_required: diff_summary.is_some(),
            diff_summary,
            test_artifact_ids: request.capture_plan.test_artifact_ids,
        };
        self.artifacts.push(artifact.clone());
        Ok(artifact)
    }

    pub fn list_artifacts(&self, run_id: &str) -> Vec<&ExternalAgentRunArtifact> {
        self.artifacts
            .iter()
            .filter(|artifact| artifact.run_id == run_id)
            .collect()
    }

    pub fn record_review_decision(
        &mut self,
        artifact_id: &str,
        decision: ExternalAgentReviewDecision,
        timestamp: u64,
    ) -> Result<ExternalAgentRunArtifact, ExternalAgentError> {
        let artifact = self
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.id == artifact_id)
            .ok_or(ExternalAgentError::ArtifactNotFound)?;
        let (status, message) = match decision {
            ExternalAgentReviewDecision::Accept => (
                ExternalAgentRunStatus::Accepted,
                "External worker diff accepted.",
            ),
            ExternalAgentReviewDecision::Revert => (
                ExternalAgentRunStatus::Reverted,
                "External worker diff reverted.",
            ),
        };
        artifact.status = status;
        artifact.transcript.push(event(
            ExternalAgentEventKind::ReviewDecision,
            message,
            timestamp,
        ));
        Ok(artifact.clone())
    }

    fn ensure_available(&self, adapter_id: &str) -> Result<(), ExternalAgentError> {
        self.adapters
            .iter()
            .find(|adapter| {
                adapter.adapter_id == adapter_id && adapter.status == AdapterStatus::Available
            })
            .map(|_| ())
            .ok_or(ExternalAgentError::AdapterUnavailable)
    }

    fn ensure_task_authority(
        &self,
        request: &ExternalAgentRunRequest,
    ) -> Result<(), ExternalAgentError> {
        for tool in &request.allowed_tools {
            if !request.task_policy.allowed_tools.contains(tool) {
                return Err(ExternalAgentError::ToolNotAllowed(tool.clone()));
            }
        }
        Ok(())
    }

    fn checked_scope(
        &self,
        scope: ExternalAgentScope,
    ) -> Result<ExternalAgentScope, ExternalAgentError> {
        let root = checked_approved_path(&scope.project_root, &self.approved_roots)?;
        let allowed_paths = scope
            .allowed_paths
            .iter()
            .map(|path| checked_approved_path(path, &self.approved_roots))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ExternalAgentScope {
            project_root: root,
            checkpoint_id: scope.checkpoint_id,
            worktree_id: scope.worktree_id,
            allowed_paths,
        })
    }
}

pub fn tests_are_trusted(artifact: &ExternalAgentRunArtifact) -> bool {
    !artifact.test_artifact_ids.is_empty()
}

fn event(kind: ExternalAgentEventKind, message: &str, timestamp: u64) -> ExternalAgentEvent {
    ExternalAgentEvent {
        kind,
        message: message.to_string(),
        timestamp,
    }
}

fn io_error(error: std::io::Error) -> ExternalAgentError {
    ExternalAgentError::Io(error.to_string())
}
