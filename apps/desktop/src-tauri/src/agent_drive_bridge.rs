use crate::agent_drive::{drive_run, AgentDriveContext};
use crate::agent_drive_types::AgentDriveOutcomeView;
use crate::approval_bridge::ApprovalBridgeState;
use crate::patch_bridge::PatchBridgeState;
use crate::plan_bridge::PlanBridgeState;
use crate::review_bridge::ReviewBridgeState;
use crate::test_runner_bridge::TestRunnerBridgeState;
use crate::thread_run_bridge::ThreadRunBridgeState;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDriveRequest {
    pub run_id: String,
    pub now_ms: u64,
    pub updated_at: String,
    pub final_summary: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[tauri::command]
pub fn agent_drive_run(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    plans: tauri::State<PlanBridgeState>,
    request: AgentDriveRequest,
) -> Result<AgentDriveOutcomeView, String> {
    let approval_store = approvals
        .store
        .lock()
        .map_err(|_| "Approval bridge lock failed.".to_string())?;
    let mut thread_store = threads
        .store
        .lock()
        .map_err(|_| "Thread bridge lock failed.".to_string())?;
    let mut patch_store = patches
        .store
        .lock()
        .map_err(|_| "Patch bridge lock failed.".to_string())?;
    let mut test_store = tests
        .store
        .lock()
        .map_err(|_| "Test bridge lock failed.".to_string())?;
    let mut review_store = reviews
        .store
        .lock()
        .map_err(|_| "Review bridge lock failed.".to_string())?;
    let mut context = AgentDriveContext {
        approvals: &approval_store,
        final_summary: request.final_summary,
        now_ms: request.now_ms,
        patches: &mut patch_store,
        plan_db: plans.database_path(),
        reviews: &mut review_store,
        run_id: request.run_id,
        tests: &mut test_store,
        threads: &mut thread_store,
        timeout_ms: request.timeout_ms,
        updated_at: request.updated_at,
    };
    drive_run(
        &mut context,
        |thread_store, patch_store, test_store, review_store| {
            threads.persist(thread_store)?;
            patches.save_if_persistent(patch_store)?;
            tests.save_if_persistent(test_store)?;
            reviews.save_if_persistent(review_store)?;
            Ok(())
        },
    )
}
