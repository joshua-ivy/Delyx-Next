use crate::agent_run::{AgentEvent, AgentRun, AgentRunError, AgentRunLedger};

pub fn create_agent_run(
    ledger: &mut AgentRunLedger,
    thread_id: &str,
) -> Result<AgentRun, AgentRunError> {
    ledger.create_run(thread_id)
}

pub fn list_agent_runs<'a>(ledger: &'a AgentRunLedger, thread_id: &str) -> Vec<&'a AgentRun> {
    ledger.list_runs(thread_id)
}

pub fn get_agent_run<'a>(
    ledger: &'a AgentRunLedger,
    run_id: &str,
) -> Result<&'a AgentRun, AgentRunError> {
    ledger.get_run(run_id)
}

pub fn append_agent_event(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    kind: &str,
    message: &str,
) -> Result<AgentEvent, AgentRunError> {
    ledger.append_event(run_id, kind, message)
}
