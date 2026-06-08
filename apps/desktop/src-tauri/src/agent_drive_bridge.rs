use crate::agent_drive::{drive_run, AgentDriveContext};
use crate::agent_drive_approvals::ApprovalExpiry;
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
    /// When present, the driver creates the pending apply/repair approval card
    /// itself using this renderer-supplied expiry. Absent = yield to the renderer.
    pub approval_expires_at: Option<String>,
    pub approval_expires_at_ms: Option<u64>,
}

/// Canonical store lock order for the driver and any command that holds more
/// than one bridge store at once. Always acquire in this order to avoid deadlock:
/// approvals -> threads -> patches -> tests -> reviews -> plans. The driver only
/// reads approvals today, so it locks that store first and never re-acquires it.
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
    let mut approval_store = approvals
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
    let outcome = {
        let mut context = AgentDriveContext {
            approvals: &mut approval_store,
            approval_expiry: approval_expiry(&request),
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
        )?
    };
    approvals.persist(&approval_store)?;
    Ok(outcome)
}

fn approval_expiry(request: &AgentDriveRequest) -> Option<ApprovalExpiry> {
    match (&request.approval_expires_at, request.approval_expires_at_ms) {
        (Some(iso), Some(ms)) if !iso.trim().is_empty() && ms != 0 => Some(ApprovalExpiry {
            iso: iso.clone(),
            ms,
        }),
        _ => None,
    }
}
