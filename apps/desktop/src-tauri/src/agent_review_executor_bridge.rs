use crate::agent_review_executor::{
    execute_review_node, AgentReviewExecutionResult, AgentReviewExecutionStatus,
};
use crate::patch_bridge::{patch_snapshot_from_store, PatchBridgeState};
use crate::review_bridge::{ReviewBridgeState, ReviewBridgeStore};
use crate::test_runner_bridge::{
    test_snapshot_from_store, TestRunnerBridgeState, TestRunnerBridgeStore,
};
use crate::thread_run_bridge::{ThreadRunBridgeState, ThreadRunStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewExecuteRequest {
    pub run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewExecutionBridgeView {
    pub status: String,
    pub run_id: String,
    pub review_report_id: Option<String>,
    pub message: String,
}

#[tauri::command]
pub fn agent_execute_review(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    request: AgentReviewExecuteRequest,
) -> Result<AgentReviewExecutionBridgeView, String> {
    let mut thread_store = threads
        .store
        .lock()
        .map_err(|_| "Thread bridge lock failed.".to_string())?;
    let patch_store = patches
        .store
        .lock()
        .map_err(|_| "Patch bridge lock failed.".to_string())?;
    let test_store = tests
        .store
        .lock()
        .map_err(|_| "Test bridge lock failed.".to_string())?;
    let mut review_store = reviews
        .store
        .lock()
        .map_err(|_| "Review bridge lock failed.".to_string())?;
    let view = execute_review_record(
        &mut thread_store,
        &patch_store,
        &test_store,
        &mut review_store,
        request,
    )?;
    threads.persist(&thread_store)?;
    reviews.save_if_persistent(&review_store)?;
    Ok(view)
}

pub fn execute_review_record(
    thread_store: &mut ThreadRunStore,
    patch_store: &crate::patch_bridge::PatchBridgeStore,
    test_store: &TestRunnerBridgeStore,
    review_store: &mut ReviewBridgeStore,
    request: AgentReviewExecuteRequest,
) -> Result<AgentReviewExecutionBridgeView, String> {
    bridge_view(execute_review_node(
        &mut thread_store.ledger,
        review_store,
        request.run_id.clone(),
        patch_snapshot_from_store(patch_store, &request.run_id),
        test_snapshot_from_store(test_store, &request.run_id),
    )?)
}

fn bridge_view(
    result: AgentReviewExecutionResult,
) -> Result<AgentReviewExecutionBridgeView, String> {
    Ok(AgentReviewExecutionBridgeView {
        message: result.message,
        review_report_id: result.review_report_id,
        run_id: result.run_id,
        status: status_key(result.status).to_string(),
    })
}

fn status_key(status: AgentReviewExecutionStatus) -> &'static str {
    match status {
        AgentReviewExecutionStatus::Completed => "completed",
        AgentReviewExecutionStatus::Failed => "failed",
    }
}
