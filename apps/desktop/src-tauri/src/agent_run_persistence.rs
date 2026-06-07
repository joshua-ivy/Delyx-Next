use std::fs;
use std::path::Path;

use crate::agent_run::{
    AgentEvent, AgentOutcome, AgentRun, AgentRunError, AgentRunLedger, AgentRunStatus, RunMetrics,
};

pub fn save_to_path(ledger: &AgentRunLedger, path: &Path) -> Result<(), AgentRunError> {
    fs::write(path, encode(ledger)).map_err(|error| AgentRunError::Io(error.to_string()))
}

pub fn load_from_path(path: &Path) -> Result<AgentRunLedger, AgentRunError> {
    let contents = fs::read_to_string(path).map_err(|error| AgentRunError::Io(error.to_string()))?;
    decode(&contents)
}

fn encode(ledger: &AgentRunLedger) -> String {
    ledger
        .runs
        .iter()
        .map(|run| {
            let outcome = run.outcome.as_ref().map(|value| value.summary.as_str()).unwrap_or("");
            format!("RUN\t{}\t{}\t{}\t{}", esc(&run.id), esc(&run.thread_id), status_key(run.status), esc(outcome))
        })
        .chain(ledger.runs.iter().flat_map(|run| {
            run.events.iter().map(move |event| {
                format!("EVENT\t{}\t{}\t{}\t{}", esc(&run.id), esc(&event.id), esc(&event.kind), esc(&event.message))
            })
        }))
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode(contents: &str) -> Result<AgentRunLedger, AgentRunError> {
    let mut ledger = AgentRunLedger::new();
    for line in contents.lines() {
        let parts: Vec<_> = line.split('\t').collect();
        match parts.as_slice() {
            ["RUN", id, thread_id, status, outcome] => push_loaded_run(&mut ledger, id, thread_id, status, outcome)?,
            ["EVENT", run_id, id, kind, message] => push_loaded_event(&mut ledger, run_id, id, kind, message)?,
            _ => return Err(AgentRunError::InvalidLedger(line.to_string())),
        }
    }
    Ok(ledger)
}

fn push_loaded_run(
    ledger: &mut AgentRunLedger,
    id: &str,
    thread_id: &str,
    status: &str,
    outcome: &str,
) -> Result<(), AgentRunError> {
    let status = parse_status(status)?;
    ledger.runs.push(AgentRun {
        id: unesc(id),
        thread_id: unesc(thread_id),
        status,
        nodes: Vec::new(),
        events: Vec::new(),
        artifacts: Vec::new(),
        evidence: Vec::new(),
        metrics: RunMetrics::default(),
        outcome: (!outcome.is_empty()).then(|| AgentOutcome { status, summary: unesc(outcome) }),
    });
    ledger.next_run = ledger.runs.len();
    Ok(())
}

fn push_loaded_event(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    id: &str,
    kind: &str,
    message: &str,
) -> Result<(), AgentRunError> {
    let run = ledger.run_mut(&unesc(run_id))?;
    run.events.push(AgentEvent { id: unesc(id), kind: unesc(kind), message: unesc(message) });
    run.metrics.event_count = run.events.len();
    Ok(())
}

fn status_key(status: AgentRunStatus) -> &'static str {
    match status {
        AgentRunStatus::Running => "running",
        AgentRunStatus::WaitingForApproval => "waiting_for_approval",
        AgentRunStatus::Completed => "completed",
        AgentRunStatus::Failed => "failed",
    }
}

fn parse_status(value: &str) -> Result<AgentRunStatus, AgentRunError> {
    match value {
        "running" => Ok(AgentRunStatus::Running),
        "waiting_for_approval" => Ok(AgentRunStatus::WaitingForApproval),
        "completed" => Ok(AgentRunStatus::Completed),
        "failed" => Ok(AgentRunStatus::Failed),
        _ => Err(AgentRunError::InvalidLedger(value.to_string())),
    }
}

fn esc(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n")
}

fn unesc(value: &str) -> String {
    value.replace("\\n", "\n").replace("\\t", "\t").replace("\\\\", "\\")
}
