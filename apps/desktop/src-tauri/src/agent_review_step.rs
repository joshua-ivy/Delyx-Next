use crate::agent_review_executor_bridge::{
    execute_review_record, AgentReviewExecuteRequest, AgentReviewExecutionBridgeView,
};
use crate::agent_scheduler_bridge::{schedule_next_record, AgentScheduleRequest};
use crate::agent_scheduler_patch_context::hydrate_patch_draft_request;
use crate::agent_scheduler_test_context::hydrate_schedule_request;
use crate::approval_bridge::{ApprovalBridgeState, ApprovalBridgeStore};
use crate::patch_bridge::{PatchBridgeState, PatchBridgeStore};
use crate::plan_bridge::PlanBridgeState;
use crate::review_bridge::{ReviewBridgeState, ReviewBridgeStore};
use crate::test_runner_bridge::{TestRunnerBridgeState, TestRunnerBridgeStore};
use crate::thread_run_bridge::{ThreadRunBridgeState, ThreadRunStore};
use crate::threads::ThreadStatus;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewStepRequest {
    pub run_id: String,
    pub now_ms: u64,
    pub updated_at: String,
}

#[tauri::command]
pub fn agent_run_review_step(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    plans: tauri::State<PlanBridgeState>,
    request: AgentReviewStepRequest,
) -> Result<AgentReviewExecutionBridgeView, String> {
    let review_request = {
        let approval_store = approvals
            .store
            .lock()
            .map_err(|_| "Approval bridge lock failed.".to_string())?;
        let thread_store = threads
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
        let review_store = reviews
            .store
            .lock()
            .map_err(|_| "Review bridge lock failed.".to_string())?;
        scheduler_review_request(
            &thread_store,
            &approval_store,
            &patch_store,
            &test_store,
            &review_store,
            plans.database_path(),
            &request,
        )?
    };
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
    let view = execute_scheduler_review_record(
        &mut thread_store,
        &patch_store,
        &test_store,
        &mut review_store,
        review_request,
        &request.updated_at,
    )?;
    threads.persist(&thread_store)?;
    reviews.save_if_persistent(&review_store)?;
    Ok(view)
}

pub(crate) fn scheduler_review_request(
    threads: &ThreadRunStore,
    approvals: &ApprovalBridgeStore,
    patches: &PatchBridgeStore,
    tests: &TestRunnerBridgeStore,
    reviews: &ReviewBridgeStore,
    plan_db: &Path,
    request: &AgentReviewStepRequest,
) -> Result<AgentReviewExecuteRequest, String> {
    validate_request(request)?;
    let schedule_request = hydrate_patch_draft_request(
        threads,
        approvals,
        patches,
        reviews,
        plan_db,
        AgentScheduleRequest {
            has_supported_test_command: false,
            now_ms: request.now_ms,
            patch_apply_approval_id: None,
            patch_draft_approval_id: None,
            run_id: request.run_id.clone(),
            test_approval_id: None,
        },
    )?;
    let schedule_request = hydrate_schedule_request(threads, approvals, plan_db, schedule_request)?;
    let run = threads
        .ledger
        .get_run(&request.run_id)
        .map_err(|error| format!("{error:?}"))?;
    let decision = schedule_next_record(
        run,
        &approvals.engine,
        patches,
        tests,
        reviews,
        &schedule_request,
    );
    if decision.kind != "run_review" {
        return Err(format!(
            "Scheduler selected `{}` instead of review.",
            decision.kind
        ));
    }
    Ok(AgentReviewExecuteRequest {
        run_id: request.run_id.clone(),
    })
}

pub(crate) fn execute_scheduler_review_record(
    thread_store: &mut ThreadRunStore,
    patch_store: &PatchBridgeStore,
    test_store: &TestRunnerBridgeStore,
    review_store: &mut ReviewBridgeStore,
    request: AgentReviewExecuteRequest,
    updated_at: &str,
) -> Result<AgentReviewExecutionBridgeView, String> {
    move_thread_to_reviewing(thread_store, &request.run_id, updated_at)?;
    let view = execute_review_record(thread_store, patch_store, test_store, review_store, request)?;
    match view.status.as_str() {
        "completed" => set_record_timestamp(thread_store, &view.run_id, updated_at)?,
        "failed" => {
            set_thread_status(thread_store, &view.run_id, ThreadStatus::Failed)?;
            set_record_timestamp(thread_store, &view.run_id, updated_at)?;
        }
        _ => return Err("Scheduler review step did not complete execution.".to_string()),
    }
    Ok(view)
}

fn move_thread_to_reviewing(
    store: &mut ThreadRunStore,
    run_id: &str,
    updated_at: &str,
) -> Result<(), String> {
    if updated_at.trim().is_empty() {
        return Err("Review step requires an update timestamp.".to_string());
    }
    match thread_status_for_run(store, run_id)? {
        ThreadStatus::Reviewing => {}
        ThreadStatus::Building | ThreadStatus::Testing => {
            set_thread_status(store, run_id, ThreadStatus::Reviewing)?;
        }
        ThreadStatus::Planning | ThreadStatus::WaitingForApproval | ThreadStatus::Blocked => {
            set_thread_status(store, run_id, ThreadStatus::Building)?;
            set_thread_status(store, run_id, ThreadStatus::Reviewing)?;
        }
        _ => return Err("Review cannot move this thread to reviewing.".to_string()),
    }
    set_record_timestamp(store, run_id, updated_at)
}

fn thread_status_for_run(store: &ThreadRunStore, run_id: &str) -> Result<ThreadStatus, String> {
    let thread_id = thread_id_for_run(store, run_id)?;
    Ok(store
        .manager
        .get_thread(&thread_id)
        .map_err(|error| format!("{error:?}"))?
        .status)
}

fn set_thread_status(
    store: &mut ThreadRunStore,
    run_id: &str,
    status: ThreadStatus,
) -> Result<(), String> {
    let thread_id = thread_id_for_run(store, run_id)?;
    store
        .manager
        .set_status(&thread_id, status)
        .map_err(|error| format!("{error:?}"))
}

fn set_record_timestamp(
    store: &mut ThreadRunStore,
    run_id: &str,
    updated_at: &str,
) -> Result<(), String> {
    let record = store
        .records
        .iter_mut()
        .find(|record| record.run_id == run_id)
        .ok_or_else(|| "Thread run record was not found.".to_string())?;
    record.updated_at = updated_at.to_string();
    Ok(())
}

fn thread_id_for_run(store: &ThreadRunStore, run_id: &str) -> Result<String, String> {
    store
        .records
        .iter()
        .find(|record| record.run_id == run_id)
        .map(|record| record.thread_id.clone())
        .ok_or_else(|| "Thread run record was not found.".to_string())
}

fn validate_request(request: &AgentReviewStepRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty()
        || request.now_ms == 0
        || request.updated_at.trim().is_empty()
    {
        return Err("Review step requires run, clock, and timestamp.".to_string());
    }
    Ok(())
}
