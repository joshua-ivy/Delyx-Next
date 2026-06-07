#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ApprovalError, ProposalInput, RiskLevel, RiskyAction};
    use crate::external_agent::{
        tests_are_trusted, AdapterStatus, ExternalAgentBridge, ExternalAgentCapturePlan, ExternalAgentError,
        ExternalAgentEventKind, ExternalAgentReviewDecision, ExternalAgentRunRequest, ExternalAgentScope,
        ExternalAgentTaskPolicy,
    };
    use crate::external_agent_terminal::ExternalAgentCommand;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_placeholder_adapters() {
        let root = temp_workspace("adapter-detect");
        let bridge = ExternalAgentBridge::new(vec![root]).unwrap();
        let adapters = bridge.detect_adapters();

        assert!(adapters.iter().any(|adapter| adapter.adapter_id == "codex-cli" && adapter.status == AdapterStatus::Missing));
        assert!(adapters.iter().any(|adapter| adapter.adapter_id == "claude-code" && adapter.status == AdapterStatus::Missing));
        assert!(adapters.iter().any(|adapter| adapter.adapter_id == "generic-terminal" && adapter.status == AdapterStatus::Available));
    }

    #[test]
    fn external_agent_cannot_run_without_approval() {
        let root = temp_workspace("pending-external");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let result = bridge.run_approved_worker(run_request(&approval.id, &root, true, vec![]), 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::Approval(ApprovalError::NotApproved));
    }

    #[test]
    fn external_agent_requires_external_agent_approval_action() {
        let root = temp_workspace("wrong-external-approval");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(ProposalInput { action: RiskyAction::TerminalCommand, ..external_agent_input() });
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let result = bridge.run_approved_worker(run_request(&approval.id, &root, true, vec![]), 10, &approvals);

        assert_eq!(
            result.unwrap_err(),
            ExternalAgentError::Approval(ApprovalError::ActionMismatch {
                expected: RiskyAction::ExternalAgentExecution,
                actual: RiskyAction::TerminalCommand,
            })
        );
    }

    #[test]
    fn external_agent_requires_same_run_approval() {
        let root = temp_workspace("wrong-external-run");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(ProposalInput { run_id: "run-2".to_string(), ..external_agent_input() });
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let result = bridge.run_approved_worker(run_request(&approval.id, &root, true, vec![]), 10, &approvals);

        assert_eq!(
            result.unwrap_err(),
            ExternalAgentError::Approval(ApprovalError::RunMismatch {
                expected: "run-1".to_string(),
                actual: "run-2".to_string(),
            })
        );
    }

    #[test]
    fn external_agent_scope_must_be_inside_approved_root() {
        let root = temp_workspace("approved-external");
        let outside = temp_workspace("outside-external");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root]).unwrap();

        let result = bridge.run_approved_worker(run_request(&approval.id, &outside, true, vec![]), 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::OutsideApprovedRoot);
    }

    #[test]
    fn external_agent_changed_files_must_stay_inside_approved_scope() {
        let root = temp_workspace("changed-approved");
        let outside = temp_workspace("changed-outside");
        let outside_file = outside.join("leak.txt");
        fs::write(&outside_file, "outside").unwrap();
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let mut request = run_request(&approval.id, &root, true, vec![]);
        request.capture_plan.changed_files = vec![outside_file];

        let result = bridge.run_approved_worker(request, 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::OutsideApprovedRoot);
    }

    #[test]
    fn approved_generic_worker_captures_transcript_output_and_diff() {
        let root = temp_workspace("approved-worker");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let artifact = bridge.run_approved_worker(run_request(&approval.id, &root, true, vec![]), 10, &approvals).unwrap();

        assert_eq!(artifact.adapter_id, "generic-terminal");
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Stdout));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Command));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::FileChanged));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Completed));
        assert!(artifact.transcript.iter().all(|event| event.timestamp == 10));
        assert!(artifact.terminal_output.contains("prototype external agent bridge"));
        assert!(artifact.diff_summary.unwrap().contains("Delyx review"));
        assert!(artifact.review_required);
    }

    #[test]
    fn approved_generic_worker_runs_terminal_command_and_captures_output() {
        let root = temp_workspace("generic-terminal-command");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let artifact = bridge.run_approved_worker(command_request(&approval.id, &root, passing_command()), 10, &approvals).unwrap();

        assert_eq!(artifact.status, crate::external_agent::ExternalAgentRunStatus::Completed);
        assert!(artifact.terminal_output.contains("worker passed"));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Stdout));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Completed));
    }

    #[test]
    fn failed_generic_worker_command_is_captured_as_failed_artifact() {
        let root = temp_workspace("generic-terminal-failed");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let artifact = bridge.run_approved_worker(command_request(&approval.id, &root, failing_command()), 10, &approvals).unwrap();

        assert_eq!(artifact.status, crate::external_agent::ExternalAgentRunStatus::Failed);
        assert!(artifact.terminal_output.contains("worker failed"));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Failed));
    }

    #[test]
    fn external_agent_cannot_request_tools_outside_task_policy() {
        let root = temp_workspace("tool-authority");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let mut request = run_request(&approval.id, &root, true, vec![]);
        request.allowed_tools.push("terminal".to_string());

        let result = bridge.run_approved_worker(request, 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::ToolNotAllowed("terminal".to_string()));
    }

    #[test]
    fn worker_command_requires_terminal_command_tool() {
        let root = temp_workspace("generic-terminal-denied");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let mut request = run_request(&approval.id, &root, false, vec![]);
        let (program, args) = passing_command();
        request.worker_command = Some(ExternalAgentCommand { program, args, working_directory: root.clone() });

        let result = bridge.run_approved_worker(request, 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::ToolNotAllowed("terminal_command".to_string()));
    }

    #[test]
    fn external_agent_requires_checkpoint_or_worktree_isolation() {
        let root = temp_workspace("missing-isolation");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let mut request = run_request(&approval.id, &root, true, vec![]);
        request.scope.checkpoint_id = None;
        request.scope.worktree_id = None;

        let result = bridge.run_approved_worker(request, 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::MissingIsolation);
    }

    #[test]
    fn user_can_accept_or_revert_external_agent_artifact() {
        let root = temp_workspace("review-decision");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let artifact = bridge.run_approved_worker(run_request(&approval.id, &root, true, vec![]), 10, &approvals).unwrap();

        let accepted = bridge.record_review_decision(&artifact.id, ExternalAgentReviewDecision::Accept, 20).unwrap();
        let reverted = bridge.record_review_decision(&artifact.id, ExternalAgentReviewDecision::Revert, 30).unwrap();

        assert_eq!(accepted.status, crate::external_agent::ExternalAgentRunStatus::Accepted);
        assert_eq!(reverted.status, crate::external_agent::ExternalAgentRunStatus::Reverted);
        assert!(reverted.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::ReviewDecision && event.timestamp == 30));
    }

    #[test]
    fn tests_are_not_trusted_without_captured_artifacts() {
        let root = temp_workspace("test-trust");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let no_tests = bridge.run_approved_worker(run_request(&approval.id, &root, false, vec![]), 10, &approvals).unwrap();
        let with_tests = bridge
            .run_approved_worker(run_request(&approval.id, &root, false, vec!["test-artifact-1".to_string()]), 10, &approvals)
            .unwrap();

        assert!(!tests_are_trusted(&no_tests));
        assert!(tests_are_trusted(&with_tests));
    }

    fn external_agent_input() -> ProposalInput {
        ProposalInput {
            action: RiskyAction::ExternalAgentExecution,
            expires_at: 30,
            expected_result: "External worker runs inside approved scope.".to_string(),
            node_id: "node-external-agent".to_string(),
            reason: "Deterministic external bridge test.".to_string(),
            risk: RiskLevel::High,
            rollback_plan: "Discard transcript and restore checkpoint if changes are rejected.".to_string(),
            run_id: "run-1".to_string(),
            scope: "Launch generic terminal agent in one approved project root.".to_string(),
        }
    }

    fn run_request(approval_id: &str, root: &std::path::Path, capture_diff: bool, test_artifact_ids: Vec<String>) -> ExternalAgentRunRequest {
        ExternalAgentRunRequest {
            adapter_id: "generic-terminal".to_string(),
            allowed_tools: vec!["read".to_string(), "patch_proposal".to_string()],
            approval_id: approval_id.to_string(),
            capture_plan: ExternalAgentCapturePlan {
                capture_diff,
                changed_files: vec![root.join("src").join("main.rs")],
                commands: vec!["echo prototype".to_string()],
                test_artifact_ids,
            },
            run_id: "run-1".to_string(),
            scope: ExternalAgentScope {
                allowed_paths: vec![root.to_path_buf()],
                checkpoint_id: Some("checkpoint-1".to_string()),
                project_root: root.to_path_buf(),
                worktree_id: None,
            },
            task: "Inspect and propose a patch.".to_string(),
            task_policy: ExternalAgentTaskPolicy {
                allowed_tools: vec!["read".to_string(), "patch_proposal".to_string()],
                approval_scope: "Launch generic terminal agent in one approved project root.".to_string(),
            },
            timeout_ms: 60_000,
            worker_command: None,
        }
    }

    fn command_request(
        approval_id: &str,
        root: &std::path::Path,
        command: (String, Vec<String>),
    ) -> ExternalAgentRunRequest {
        let mut request = run_request(approval_id, root, false, vec![]);
        request.allowed_tools.push("terminal_command".to_string());
        request.task_policy.allowed_tools.push("terminal_command".to_string());
        request.worker_command = Some(ExternalAgentCommand {
            args: command.1,
            program: command.0,
            working_directory: root.to_path_buf(),
        });
        request
    }

    fn passing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            ("cmd".to_string(), vec!["/C".to_string(), "echo worker passed".to_string()])
        } else {
            ("sh".to_string(), vec!["-c".to_string(), "echo worker passed".to_string()])
        }
    }

    fn failing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            ("cmd".to_string(), vec!["/C".to_string(), "echo worker failed 1>&2 & exit /B 7".to_string()])
        } else {
            ("sh".to_string(), vec!["-c".to_string(), "echo worker failed >&2; exit 7".to_string()])
        }
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        path
    }
}
