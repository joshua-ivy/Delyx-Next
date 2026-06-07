#[cfg(test)]
mod tests {
    use crate::approval::{ApprovalEngine, ApprovalError, ProposalInput, RiskLevel, RiskyAction};
    use crate::test_runner::{is_test_command, TestCommandInput, TestRunner, TestRunnerError, TestStatus};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_common_test_commands() {
        assert!(is_test_command("cargo", &["test".to_string()]));
        assert!(is_test_command("npm", &["run".to_string(), "test".to_string()]));
        assert!(is_test_command("pytest", &[]));
        assert!(!is_test_command("npm", &["install".to_string()]));
    }

    #[test]
    fn test_command_requires_approval() {
        let root = temp_workspace("pending-command");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(command_input());
        let mut runner = TestRunner::new(vec![root.clone()]).unwrap();

        let result = runner.run_approved_test(test_input(&approval.id, &root, passing_command()), 10, &approvals);

        assert_eq!(result.unwrap_err(), TestRunnerError::Approval(ApprovalError::NotApproved));
        assert!(!runner.has_execution_artifact("run-1"));
    }

    #[test]
    fn approved_test_captures_output_exit_code_and_duration() {
        let root = temp_workspace("passing-command");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(command_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut runner = TestRunner::new(vec![root.clone()]).unwrap();

        let artifact = runner.run_approved_test(test_input(&approval.id, &root, passing_command()), 10, &approvals).unwrap();

        assert_eq!(artifact.status, TestStatus::Passed);
        assert_eq!(artifact.exit_code, Some(0));
        assert!(artifact.stdout.to_lowercase().contains("test passed"));
        assert!(artifact.duration_ms <= 60_000);
        assert!(runner.has_execution_artifact("run-1"));
    }

    #[test]
    fn failed_test_captures_failure_summary() {
        let root = temp_workspace("failing-command");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(command_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut runner = TestRunner::new(vec![root.clone()]).unwrap();

        let artifact = runner.run_approved_test(test_input(&approval.id, &root, failing_command()), 10, &approvals).unwrap();

        assert_eq!(artifact.status, TestStatus::Failed);
        assert_ne!(artifact.exit_code, Some(0));
        assert!(artifact.stderr.to_lowercase().contains("test failure"));
        assert_eq!(artifact.failure_summary.as_deref(), Some("test failure"));
    }

    #[test]
    fn working_directory_must_be_inside_approved_root() {
        let root = temp_workspace("approved-test-root");
        let outside = temp_workspace("outside-test-root");
        let mut approvals = ApprovalEngine::new();
        let approval = approvals.propose(command_input());
        approvals.approve(&approval.id, 10, "approved in test").unwrap();
        let mut runner = TestRunner::new(vec![root]).unwrap();

        let result = runner.run_approved_test(test_input(&approval.id, &outside, passing_command()), 10, &approvals);

        assert_eq!(result.unwrap_err(), TestRunnerError::OutsideApprovedRoot);
    }

    fn command_input() -> ProposalInput {
        ProposalInput {
            action: RiskyAction::TerminalCommand,
            expires_at: 30,
            expected_result: "Capture test command output.".to_string(),
            node_id: "node-test-runner".to_string(),
            reason: "Deterministic test artifact execution.".to_string(),
            risk: RiskLevel::Medium,
            rollback_plan: "No durable mutation; artifact can be discarded.".to_string(),
            run_id: "run-1".to_string(),
            scope: "Run one test command in an approved root.".to_string(),
        }
    }

    fn test_input(approval_id: &str, cwd: &std::path::Path, command: (String, Vec<String>)) -> TestCommandInput {
        TestCommandInput {
            run_id: "run-1".to_string(),
            approval_id: approval_id.to_string(),
            program: command.0,
            args: command.1,
            working_directory: cwd.to_path_buf(),
        }
    }

    fn passing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            ("cmd".to_string(), vec!["/C".to_string(), "echo test passed".to_string()])
        } else {
            ("sh".to_string(), vec!["-c".to_string(), "echo test passed".to_string()])
        }
    }

    fn failing_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            ("cmd".to_string(), vec!["/C".to_string(), "echo test failure 1>&2 & exit /B 2".to_string()])
        } else {
            ("sh".to_string(), vec!["-c".to_string(), "echo test failure >&2; exit 2".to_string()])
        }
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("delyx-next-{label}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
