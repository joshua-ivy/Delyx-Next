use crate::agent_executor_bridge::{execute_patch_apply_record, AgentExecutionBridgeView};
use crate::agent_scheduler_bridge::{schedule_next_record, AgentScheduleRequest};
use crate::agent_scheduler_patch_context::hydrate_patch_draft_request;
use crate::agent_scheduler_test_context::hydrate_schedule_request;
use crate::approval_bridge::{ApprovalBridgeState, ApprovalBridgeStore};
use crate::patch_apply_bridge::PatchApplyRequest;
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
pub struct AgentPatchApplyStepRequest {
    pub run_id: String,
    pub now_ms: u64,
    pub updated_at: String,
}

#[tauri::command]
pub fn agent_run_patch_apply_step(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    plans: tauri::State<PlanBridgeState>,
    request: AgentPatchApplyStepRequest,
) -> Result<AgentExecutionBridgeView, String> {
    let apply_request = {
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
        scheduler_patch_apply_request(
            &thread_store,
            &approval_store,
            &patch_store,
            &test_store,
            &review_store,
            plans.database_path(),
            &request,
        )?
    };
    approvals.with_engine(|engine| {
        let mut thread_store = threads
            .store
            .lock()
            .map_err(|_| "Thread bridge lock failed.".to_string())?;
        let mut patch_store = patches
            .store
            .lock()
            .map_err(|_| "Patch bridge lock failed.".to_string())?;
        let view = execute_scheduler_patch_apply_record(
            &mut thread_store,
            &mut patch_store,
            engine,
            apply_request,
            &request.updated_at,
        )?;
        threads.persist(&thread_store)?;
        patches.save_if_persistent(&patch_store)?;
        Ok(view)
    })?
}

pub(crate) fn scheduler_patch_apply_request(
    threads: &ThreadRunStore,
    approvals: &ApprovalBridgeStore,
    patches: &PatchBridgeStore,
    tests: &TestRunnerBridgeStore,
    reviews: &ReviewBridgeStore,
    plan_db: &Path,
    request: &AgentPatchApplyStepRequest,
) -> Result<PatchApplyRequest, String> {
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
    if decision.kind != "run_patch_apply" {
        return Err(format!(
            "Scheduler selected `{}` instead of patch apply.",
            decision.kind
        ));
    }
    let proposal_id = decision
        .proposal_id
        .ok_or_else(|| "Scheduler did not return a patch proposal.".to_string())?;
    let approval_id = decision
        .approval_ids
        .first()
        .ok_or_else(|| "Scheduler did not return a patch apply approval.".to_string())?;
    Ok(PatchApplyRequest {
        approval_id: approval_id.clone(),
        approved_roots: approved_roots_for_run(threads, plan_db, &request.run_id)?,
        created_at_ms: request.now_ms,
        proposal_id,
    })
}

pub(crate) fn execute_scheduler_patch_apply_record(
    thread_store: &mut ThreadRunStore,
    patch_store: &mut PatchBridgeStore,
    approvals: &crate::approval::ApprovalEngine,
    request: PatchApplyRequest,
    updated_at: &str,
) -> Result<AgentExecutionBridgeView, String> {
    let run_id = patch_store
        .records
        .iter()
        .find(|patch| patch.id == request.proposal_id)
        .map(|patch| patch.run_id.clone())
        .ok_or_else(|| "Patch proposal not found.".to_string())?;
    ensure_thread_can_move_to_testing(thread_store, &run_id)?;
    let view = execute_patch_apply_record(thread_store, patch_store, approvals, request)?;
    if view.status == "completed" {
        move_thread_to_testing(thread_store, &view.run_id, updated_at)?;
    }
    Ok(view)
}

fn ensure_thread_can_move_to_testing(store: &ThreadRunStore, run_id: &str) -> Result<(), String> {
    if can_move_to_testing(thread_status_for_run(store, run_id)?) {
        return Ok(());
    }
    Err("Patch apply cannot move this thread to testing.".to_string())
}

fn approved_roots_for_run(
    threads: &ThreadRunStore,
    database_path: &Path,
    run_id: &str,
) -> Result<Vec<String>, String> {
    let record = threads
        .records
        .iter()
        .find(|item| item.run_id == run_id)
        .ok_or_else(|| "Thread run record was not found.".to_string())?;
    let project =
        crate::workspace_persistence::load_project_by_id(database_path, &record.project_id)?
            .ok_or_else(|| "Persisted workspace project was not found.".to_string())?;
    if project.approved_roots.is_empty() {
        return Err("Patch apply requires a persisted approved root.".to_string());
    }
    Ok(project.approved_roots)
}

fn move_thread_to_testing(
    store: &mut ThreadRunStore,
    run_id: &str,
    updated_at: &str,
) -> Result<(), String> {
    if updated_at.trim().is_empty() {
        return Err("Patch apply step requires an update timestamp.".to_string());
    }
    let thread_id = store
        .records
        .iter()
        .find(|record| record.run_id == run_id)
        .map(|record| record.thread_id.clone())
        .ok_or_else(|| "Thread run record was not found.".to_string())?;
    let status = thread_status_for_run(store, run_id)?;
    match status {
        ThreadStatus::Testing => {}
        ThreadStatus::Building => set_status(store, &thread_id, ThreadStatus::Testing)?,
        ThreadStatus::Planning
        | ThreadStatus::WaitingForApproval
        | ThreadStatus::Reviewing
        | ThreadStatus::Blocked => {
            set_status(store, &thread_id, ThreadStatus::Building)?;
            set_status(store, &thread_id, ThreadStatus::Testing)?;
        }
        _ => return Err("Patch apply cannot move this thread to testing.".to_string()),
    }
    let record = store
        .records
        .iter_mut()
        .find(|record| record.run_id == run_id)
        .ok_or_else(|| "Thread run record was not found.".to_string())?;
    record.updated_at = updated_at.to_string();
    Ok(())
}

fn thread_status_for_run(store: &ThreadRunStore, run_id: &str) -> Result<ThreadStatus, String> {
    let thread_id = store
        .records
        .iter()
        .find(|record| record.run_id == run_id)
        .map(|record| record.thread_id.clone())
        .ok_or_else(|| "Thread run record was not found.".to_string())?;
    Ok(store
        .manager
        .get_thread(&thread_id)
        .map_err(|error| format!("{error:?}"))?
        .status)
}

fn can_move_to_testing(status: ThreadStatus) -> bool {
    matches!(
        status,
        ThreadStatus::Testing
            | ThreadStatus::Building
            | ThreadStatus::Planning
            | ThreadStatus::WaitingForApproval
            | ThreadStatus::Reviewing
            | ThreadStatus::Blocked
    )
}

fn set_status(
    store: &mut ThreadRunStore,
    thread_id: &str,
    status: ThreadStatus,
) -> Result<(), String> {
    store
        .manager
        .set_status(thread_id, status)
        .map_err(|error| format!("{error:?}"))
}

fn validate_request(request: &AgentPatchApplyStepRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty()
        || request.now_ms == 0
        || request.updated_at.trim().is_empty()
    {
        return Err("Patch apply step requires run, clock, and timestamp.".to_string());
    }
    Ok(())
}
