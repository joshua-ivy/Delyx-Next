use crate::terminal_command_prep::prepare_terminal_command;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const OUTPUT_CAP_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandExecRequest {
    pub run_id: String,
    pub approval_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub timeout_ms: u64,
    pub started_at_ms: u64,
    pub prepare_terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandExecArtifact {
    pub run_id: String,
    pub approval_id: String,
    pub command: String,
    pub working_directory: PathBuf,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub status: CommandExecStatus,
    pub timeout_ms: u64,
    pub started_at_ms: u64,
    pub completed_at_ms: u64,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub events: Vec<CommandExecEvent>,
}

impl CommandExecArtifact {
    pub fn terminal_output(&self) -> String {
        format!(
            "command: {}\nexit: {:?}\nduration_ms: {}\nstdout:\n{}\nstderr:\n{}",
            self.command, self.exit_code, self.duration_ms, self.stdout, self.stderr
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandExecEvent {
    pub kind: CommandExecEventKind,
    pub message: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExecEventKind {
    Started,
    Stdout,
    Stderr,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExecStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandExecError {
    EmptyCommand,
    Io(String),
    Timeout,
}

pub fn run_command_exec(
    request: CommandExecRequest,
) -> Result<CommandExecArtifact, CommandExecError> {
    if request.program.trim().is_empty() {
        return Err(CommandExecError::EmptyCommand);
    }
    if request.timeout_ms == 0 {
        return Err(CommandExecError::Timeout);
    }

    let started = Instant::now();
    let prepared = if request.prepare_terminal {
        prepare_terminal_command(&request.program, &request.args)
    } else {
        crate::terminal_command_prep::PreparedTerminalCommand {
            program: request.program.clone(),
            args: request.args.clone(),
        }
    };
    let mut child = Command::new(&prepared.program)
        .args(&prepared.args)
        .current_dir(&request.working_directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(io_error)?;

    loop {
        if child.try_wait().map_err(io_error)?.is_some() {
            let output = child.wait_with_output().map_err(io_error)?;
            return Ok(build_artifact(request, started.elapsed(), output));
        }
        if started.elapsed() >= Duration::from_millis(request.timeout_ms) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(CommandExecError::Timeout);
        }
        sleep(Duration::from_millis(10));
    }
}

fn build_artifact(
    request: CommandExecRequest,
    elapsed: Duration,
    output: std::process::Output,
) -> CommandExecArtifact {
    let duration_ms = elapsed.as_millis() as u64;
    let (stdout, stdout_truncated) =
        cap_output(String::from_utf8_lossy(&output.stdout).to_string());
    let (stderr, stderr_truncated) =
        cap_output(String::from_utf8_lossy(&output.stderr).to_string());
    let status = if output.status.success() {
        CommandExecStatus::Succeeded
    } else {
        CommandExecStatus::Failed
    };
    let mut events = vec![event(
        CommandExecEventKind::Started,
        &format!(
            "command started: {}",
            command_label(&request.program, &request.args)
        ),
        request.started_at_ms,
    )];
    push_stream(
        &mut events,
        CommandExecEventKind::Stdout,
        &stdout,
        request.started_at_ms,
    );
    push_stream(
        &mut events,
        CommandExecEventKind::Stderr,
        &stderr,
        request.started_at_ms,
    );
    events.push(event(
        final_event_kind(status),
        final_event_message(status),
        request.started_at_ms + duration_ms,
    ));

    CommandExecArtifact {
        approval_id: request.approval_id,
        command: command_label(&request.program, &request.args),
        completed_at_ms: request.started_at_ms.saturating_add(duration_ms),
        duration_ms,
        events,
        exit_code: output.status.code(),
        run_id: request.run_id,
        started_at_ms: request.started_at_ms,
        status,
        stderr,
        stderr_truncated,
        stdout,
        stdout_truncated,
        timeout_ms: request.timeout_ms,
        working_directory: request.working_directory,
    }
}

pub(crate) fn cap_output(text: String) -> (String, bool) {
    if text.len() <= OUTPUT_CAP_BYTES {
        return (text, false);
    }
    let mut end = OUTPUT_CAP_BYTES;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (format!("{}...[truncated]", &text[..end]), true)
}

fn push_stream(
    events: &mut Vec<CommandExecEvent>,
    kind: CommandExecEventKind,
    text: &str,
    timestamp_ms: u64,
) {
    if !text.trim().is_empty() {
        events.push(event(kind, text.trim(), timestamp_ms));
    }
}

fn command_label(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn final_event_kind(status: CommandExecStatus) -> CommandExecEventKind {
    match status {
        CommandExecStatus::Succeeded => CommandExecEventKind::Completed,
        CommandExecStatus::Failed => CommandExecEventKind::Failed,
    }
}

fn final_event_message(status: CommandExecStatus) -> &'static str {
    match status {
        CommandExecStatus::Succeeded => "command completed",
        CommandExecStatus::Failed => "command failed",
    }
}

fn event(kind: CommandExecEventKind, message: &str, timestamp_ms: u64) -> CommandExecEvent {
    CommandExecEvent {
        kind,
        message: message.to_string(),
        timestamp_ms,
    }
}

fn io_error(error: std::io::Error) -> CommandExecError {
    CommandExecError::Io(error.to_string())
}
