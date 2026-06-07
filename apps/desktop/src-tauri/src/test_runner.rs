use crate::approval::{ApprovalEngine, ApprovalError};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestArtifact {
    pub id: String,
    pub run_id: String,
    pub command: String,
    pub working_directory: PathBuf,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub status: TestStatus,
    pub failure_summary: Option<String>,
    pub created_at: u64,
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
        approvals.assert_can_execute(&input.approval_id, now).map_err(TestRunnerError::Approval)?;
        let working_directory = self.checked_directory(&input.working_directory)?;

        let started = Instant::now();
        let output = Command::new(&input.program)
            .args(&input.args)
            .current_dir(&working_directory)
            .output()
            .map_err(io_error)?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let status = if output.status.success() { TestStatus::Passed } else { TestStatus::Failed };

        self.next_artifact_id += 1;
        let artifact = TestArtifact {
            id: format!("test-artifact-{}", self.next_artifact_id),
            run_id: input.run_id,
            command: command_label(&input.program, &input.args),
            working_directory,
            exit_code: output.status.code(),
            duration_ms: started.elapsed().as_millis() as u64,
            failure_summary: failure_summary(status, &stdout, &stderr),
            stdout,
            stderr,
            status,
            created_at: now,
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
    pub working_directory: PathBuf,
}

pub fn is_test_command(program: &str, args: &[String]) -> bool {
    let command = command_label(program, args).to_lowercase();
    [" test", "test ", "cargo test", "npm test", "pytest", "vitest", "nextest"]
        .iter()
        .any(|needle| command.contains(needle))
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

fn command_label(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string()).chain(args.iter().cloned()).collect::<Vec<_>>().join(" ")
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
