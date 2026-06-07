use crate::external_agent::{
    ExternalAgentError, ExternalAgentEvent, ExternalAgentEventKind, ExternalAgentRunRequest,
    ExternalAgentRunStatus, ExternalAgentScope,
};
use crate::external_agent_scope::checked_scoped_path;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const TERMINAL_TOOL: &str = "terminal_command";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentCommand {
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalWorkerExecution {
    pub status: ExternalAgentRunStatus,
    pub terminal_output: Option<String>,
    pub transcript: Vec<ExternalAgentEvent>,
}

pub fn run_worker_command(
    request: &ExternalAgentRunRequest,
    scope: &ExternalAgentScope,
    now: u64,
) -> Result<ExternalWorkerExecution, ExternalAgentError> {
    let Some(command) = &request.worker_command else {
        return Ok(ExternalWorkerExecution { status: ExternalAgentRunStatus::Completed, terminal_output: None, transcript: Vec::new() });
    };
    if !request.allowed_tools.iter().any(|tool| tool == TERMINAL_TOOL) {
        return Err(ExternalAgentError::ToolNotAllowed(TERMINAL_TOOL.to_string()));
    }
    let working_directory = checked_working_directory(&command.working_directory, scope)?;
    let output = run_command(command, &working_directory, request.timeout_ms)?;
    let mut transcript = vec![event(ExternalAgentEventKind::Command, &output.command, now)];
    push_stream(&mut transcript, ExternalAgentEventKind::Stdout, &output.stdout, now);
    push_stream(&mut transcript, ExternalAgentEventKind::Stderr, &output.stderr, now);
    let status = if output.exit_code == Some(0) { ExternalAgentRunStatus::Completed } else { ExternalAgentRunStatus::Failed };
    Ok(ExternalWorkerExecution {
        status,
        terminal_output: Some(output.terminal_output()),
        transcript,
    })
}

fn checked_working_directory(path: &Path, scope: &ExternalAgentScope) -> Result<PathBuf, ExternalAgentError> {
    checked_scoped_path(path, scope)
}

fn run_command(
    command: &ExternalAgentCommand,
    working_directory: &Path,
    timeout_ms: u64,
) -> Result<TerminalCommandOutput, ExternalAgentError> {
    if command.program.trim().is_empty() {
        return Err(ExternalAgentError::EmptyCommand);
    }
    if timeout_ms == 0 {
        return Err(ExternalAgentError::Timeout);
    }
    let started = Instant::now();
    let mut child = Command::new(&command.program)
        .args(&command.args)
        .current_dir(working_directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(io_error)?;

    loop {
        if child.try_wait().map_err(io_error)?.is_some() {
            let output = child.wait_with_output().map_err(io_error)?;
            return Ok(TerminalCommandOutput {
                command: command_label(&command.program, &command.args),
                duration_ms: started.elapsed().as_millis() as u64,
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            });
        }
        if started.elapsed() >= Duration::from_millis(timeout_ms) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExternalAgentError::Timeout);
        }
        sleep(Duration::from_millis(10));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TerminalCommandOutput {
    command: String,
    duration_ms: u64,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl TerminalCommandOutput {
    fn terminal_output(&self) -> String {
        format!(
            "command: {}\nexit: {:?}\nduration_ms: {}\nstdout:\n{}\nstderr:\n{}",
            self.command, self.exit_code, self.duration_ms, self.stdout, self.stderr
        )
    }
}

fn push_stream(transcript: &mut Vec<ExternalAgentEvent>, kind: ExternalAgentEventKind, text: &str, now: u64) {
    if !text.trim().is_empty() {
        transcript.push(event(kind, text.trim(), now));
    }
}

fn command_label(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string()).chain(args.iter().cloned()).collect::<Vec<_>>().join(" ")
}

fn event(kind: ExternalAgentEventKind, message: &str, timestamp: u64) -> ExternalAgentEvent {
    ExternalAgentEvent { kind, message: message.to_string(), timestamp }
}

fn io_error(error: std::io::Error) -> ExternalAgentError {
    ExternalAgentError::Io(error.to_string())
}
