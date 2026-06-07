use crate::approval::{ApprovalEngine, ApprovalError, RiskyAction};
use crate::command_exec::{
    run_command_exec, CommandExecArtifact, CommandExecError, CommandExecEvent, CommandExecRequest,
    CommandExecStatus,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestArtifact {
    pub id: String,
    pub run_id: String,
    pub approval_id: String,
    pub command: String,
    pub working_directory: PathBuf,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub status: TestStatus,
    pub failure_summary: Option<String>,
    pub created_at: u64,
    pub output_truncated: bool,
    pub exec_events: Vec<CommandExecEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestRunnerError {
    Approval(ApprovalError),
    EmptyCommand,
    Io(String),
    NotTestCommand,
    OutsideApprovedRoot,
    Timeout,
}

#[derive(Debug)]
pub struct TestRunner {
    approved_roots: Vec<PathBuf>,
    artifacts: Vec<TestArtifact>,
    next_artifact_id: usize,
}

impl TestRunner {
    pub fn new(approved_roots: Vec<PathBuf>) -> Result<Self, TestRunnerError> {
        let roots = approved_roots
            .iter()
            .map(|root| fs::canonicalize(root).map_err(io_error))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { approved_roots: roots, artifacts: Vec::new(), next_artifact_id: 0 })
    }

    pub fn run_approved_test(
        &mut self,
        input: TestCommandInput,
        now: u64,
        approvals: &ApprovalEngine,
    ) -> Result<TestArtifact, TestRunnerError> {
        if input.program.trim().is_empty() {
            return Err(TestRunnerError::EmptyCommand);
        }
        if !is_test_command(&input.program, &input.args) {
            return Err(TestRunnerError::NotTestCommand);
        }
        approvals
            .assert_can_execute_action_for_run(&input.approval_id, now, RiskyAction::TerminalCommand, &input.run_id)
            .map_err(TestRunnerError::Approval)?;
        let working_directory = self.checked_directory(&input.working_directory)?;
        if input.timeout_ms == 0 {
            return Err(TestRunnerError::Timeout);
        }

        let execution = run_command_exec(CommandExecRequest {
            approval_id: input.approval_id.clone(),
            args: input.args.clone(),
            prepare_terminal: false,
            program: input.program.clone(),
            run_id: input.run_id.clone(),
            started_at_ms: now,
            timeout_ms: input.timeout_ms,
            working_directory: working_directory.clone(),
        })
        .map_err(command_exec_error)?;
        let status = test_status(&execution);

        self.next_artifact_id += 1;
        let artifact = TestArtifact {
            id: format!("test-artifact-{}", self.next_artifact_id),
            run_id: input.run_id,
            approval_id: input.approval_id,
            command: execution.command,
            working_directory,
            exit_code: execution.exit_code,
            duration_ms: execution.duration_ms,
            failure_summary: failure_summary(status, &execution.stdout, &execution.stderr),
            stdout: execution.stdout,
            stderr: execution.stderr,
            status,
            created_at: now,
            output_truncated: execution.stdout_truncated || execution.stderr_truncated,
            exec_events: execution.events,
        };
        self.artifacts.push(artifact.clone());
        Ok(artifact)
    }

    pub fn list_artifacts(&self, run_id: &str) -> Vec<&TestArtifact> {
        self.artifacts.iter().filter(|artifact| artifact.run_id == run_id).collect()
    }

    pub fn has_execution_artifact(&self, run_id: &str) -> bool {
        self.artifacts.iter().any(|artifact| artifact.run_id == run_id)
    }

    fn checked_directory(&self, directory: &Path) -> Result<PathBuf, TestRunnerError> {
        let normalized = fs::canonicalize(directory).map_err(io_error)?;
        self.approved_roots
            .iter()
            .any(|root| normalized.starts_with(root))
            .then_some(normalized)
            .ok_or(TestRunnerError::OutsideApprovedRoot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCommandInput {
    pub run_id: String,
    pub approval_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub working_directory: PathBuf,
}

pub fn is_test_command(program: &str, args: &[String]) -> bool {
    let program = normalized_program_name(program);
    if is_shell_program(&program) {
        return false;
    }
    match program.as_str() {
        "cargo" => matches!(arg(args, 0), Some("test" | "nextest")),
        "npm" | "pnpm" | "yarn" => is_package_test_command(args),
        "pytest" | "vitest" | "cargo-nextest" => true,
        _ => false,
    }
}

fn failure_summary(status: TestStatus, stdout: &str, stderr: &str) -> Option<String> {
    if status == TestStatus::Passed {
        return None;
    }
    stderr
        .lines()
        .chain(stdout.lines())
        .find(|line| {
            let lower = line.to_lowercase();
            lower.contains("fail") || lower.contains("error") || lower.contains("panic")
        })
        .map(shorten)
        .or_else(|| Some("Command exited with non-zero status.".to_string()))
}

fn normalized_program_name(program: &str) -> String {
    let mut name = Path::new(program)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();
    for suffix in [".exe", ".cmd", ".bat"] {
        if name.ends_with(suffix) {
            name.truncate(name.len() - suffix.len());
        }
    }
    name
}

fn is_shell_program(program: &str) -> bool {
    matches!(program, "cmd" | "powershell" | "pwsh" | "sh" | "bash" | "zsh")
}

fn is_package_test_command(args: &[String]) -> bool {
    matches!(arg(args, 0), Some("test")) || matches!((arg(args, 0), arg(args, 1)), (Some("run"), Some("test")))
}

fn arg(args: &[String], index: usize) -> Option<&str> {
    args.get(index).map(String::as_str)
}

fn shorten(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.len() > 240 {
        format!("{}...", &trimmed[..240])
    } else {
        trimmed.to_string()
    }
}

fn io_error(error: std::io::Error) -> TestRunnerError {
    TestRunnerError::Io(error.to_string())
}

fn command_exec_error(error: CommandExecError) -> TestRunnerError {
    match error {
        CommandExecError::EmptyCommand => TestRunnerError::EmptyCommand,
        CommandExecError::Io(message) => TestRunnerError::Io(message),
        CommandExecError::Timeout => TestRunnerError::Timeout,
    }
}

fn test_status(execution: &CommandExecArtifact) -> TestStatus {
    match execution.status {
        CommandExecStatus::Succeeded => TestStatus::Passed,
        CommandExecStatus::Failed => TestStatus::Failed,
    }
}
