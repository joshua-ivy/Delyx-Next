use crate::agent_test_executor::{
    execute_test_run_node, AgentTestExecutionResult, AgentTestExecutionStatus,
};
use crate::approval_bridge::ApprovalBridgeState;
use crate::test_runner_bridge::{TestRunRequest, TestRunnerBridgeState, TestRunnerBridgeStore};
use crate::thread_run_bridge::{ThreadRunBridgeState, ThreadRunStore};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTestExecutionBridgeView {
    pub status: String,
    pub run_id: String,
    pub test_artifact_id: Option<String>,
    pub message: String,
}

#[tauri::command]
pub fn agent_execute_test_run(
    threads: tauri::State<ThreadRunBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: TestRunRequest,
) -> Result<AgentTestExecutionBridgeView, String> {
    approvals.with_engine(|engine| {
        let mut thread_store = threads
            .store
            .lock()
            .map_err(|_| "Thread bridge lock failed.".to_string())?;
        let mut test_store = tests
            .store
            .lock()
            .map_err(|_| "Test bridge lock failed.".to_string())?;
        let view = execute_test_run_record(&mut thread_store, &mut test_store, engine, request)?;
        threads.persist(&thread_store)?;
        tests.save_if_persistent(&test_store)?;
        Ok(view)
    })?
}

pub fn execute_test_run_record(
    thread_store: &mut ThreadRunStore,
    test_store: &mut TestRunnerBridgeStore,
    approvals: &crate::approval::ApprovalEngine,
    request: TestRunRequest,
) -> Result<AgentTestExecutionBridgeView, String> {
    bridge_view(execute_test_run_node(
        &mut thread_store.ledger,
        test_store,
        approvals,
        request,
    )?)
}

fn bridge_view(result: AgentTestExecutionResult) -> Result<AgentTestExecutionBridgeView, String> {
    Ok(AgentTestExecutionBridgeView {
        message: result.message,
        run_id: result.run_id,
        status: status_key(result.status).to_string(),
        test_artifact_id: result.test_artifact_id,
    })
}

fn status_key(status: AgentTestExecutionStatus) -> &'static str {
    match status {
        AgentTestExecutionStatus::Completed => "completed",
        AgentTestExecutionStatus::Failed => "failed",
        AgentTestExecutionStatus::WaitingForApproval => "waiting_for_approval",
    }
}
