#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ProposalInput, RiskLevel, RiskyAction};
    use crate::external_agent::{
        ExternalAgentBridge, ExternalAgentCapturePlan, ExternalAgentError, ExternalAgentEventKind,
        ExternalAgentRunRequest, ExternalAgentRunStatus, ExternalAgentScope, ExternalAgentTaskPolicy,
    };
    use crate::external_agent_terminal::ExternalAgentCommand;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn approved_generic_worker_runs_terminal_command_and_captures_output() {
        let root = temp_workspace("generic-terminal-command");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();

        let artifact = bridge.run_approved_worker(command_request(&approval.id, &root, passing_command()), 10, &approvals).unwrap();

        assert_eq!(artifact.status, ExternalAgentRunStatus::Completed);
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

        assert_eq!(artifact.status, ExternalAgentRunStatus::Failed);
        assert!(artifact.terminal_output.contains("worker failed"));
        assert!(artifact.transcript.iter().any(|event| event.kind == ExternalAgentEventKind::Failed));
    }

    #[test]
    fn worker_command_requires_terminal_command_tool() {
        let root = temp_workspace("generic-terminal-denied");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(external_agent_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut bridge = ExternalAgentBridge::new(vec![root.clone()]).unwrap();
        let mut request = run_request(&approval.id, &root);
        let (program, args) = passing_command();
        request.worker_command = Some(ExternalAgentCommand { program, args, working_directory: root.clone() });

        let result = bridge.run_approved_worker(request, 10, &approvals);

        assert_eq!(result.unwrap_err(), ExternalAgentError::ToolNotAllowed("terminal_command".to_string()));
    }

    fn command_request(
        approval_id: &str,
        root: &std::path::Path,
        command: (String, Vec<String>),
    ) -> ExternalAgentRunRequest {
        let mut request = run_request(approval_id, root);
        request.allowed_tools.push("terminal_command".to_string());
        request.task_policy.allowed_tools.push("terminal_command".to_string());
        request.worker_command = Some(ExternalAgentCommand {
            args: command.1,
            program: command.0,
            working_directory: root.to_path_buf(),
        });
        request
    }

    fn run_request(approval_id: &str, root: &std::path::Path) -> ExternalAgentRunRequest {
        ExternalAgentRunRequest {
            adapter_id: "generic-terminal".to_string(),
            allowed_tools: vec!["read".to_string(), "patch_proposal".to_string()],
            approval_id: approval_id.to_string(),
            capture_plan: ExternalAgentCapturePlan::default(),
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

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
