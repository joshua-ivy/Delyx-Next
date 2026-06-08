use crate::agent_scheduler_bridge::{schedule_next_record, AgentScheduleRequest};
use crate::agent_scheduler_patch_context::hydrate_patch_draft_request;
use crate::agent_scheduler_test_context::{hydrate_schedule_request, test_command_for_run};
use crate::agent_test_executor_bridge::{execute_test_run_record, AgentTestExecutionBridgeView};
use crate::approval::{ApprovalEngine, RiskyAction};
use crate::approval_bridge::{ApprovalBridgeState, ApprovalBridgeStore};
use crate::patch_bridge::{PatchBridgeState, PatchBridgeStore};
use crate::plan_bridge::PlanBridgeState;
use crate::review_bridge::{ReviewBridgeState, ReviewBridgeStore};
use crate::test_runner_bridge::{TestRunRequest, TestRunnerBridgeState, TestRunnerBridgeStore};
use crate::thread_run_bridge::{ThreadRunBridgeState, ThreadRunStore};
use crate::threads::ThreadStatus;
use serde::Deserialize;
use std::path::Path;

const DEFAULT_TEST_TIMEOUT_MS: u64 = 5 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTestStepRequest {
    pub run_id: String,
    pub now_ms: u64,
    pub started_at: String,
    pub updated_at: String,
    pub timeout_ms: Option<u64>,
}

#[tauri::command]
pub fn agent_run_test_step(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    plans: tauri::State<PlanBridgeState>,
    request: AgentTestStepRequest,
) -> Result<AgentTestExecutionBridgeView, String> {
    let test_request = {
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
        scheduler_test_request(
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
        let mut test_store = tests
            .store
            .lock()
            .map_err(|_| "Test bridge lock failed.".to_string())?;
        let view = execute_scheduler_test_record(
            &mut thread_store,
            &mut test_store,
            engine,
            test_request,
            &request.started_at,
            &request.updated_at,
        )?;
        threads.persist(&thread_store)?;
        tests.save_if_persistent(&test_store)?;
        Ok(view)
    })?
}

pub(crate) fn scheduler_test_request(
    threads: &ThreadRunStore,
    approvals: &ApprovalBridgeStore,
    patches: &PatchBridgeStore,
    tests: &TestRunnerBridgeStore,
    reviews: &ReviewBridgeStore,
    plan_db: &Path,
    request: &AgentTestStepRequest,
) -> Result<TestRunRequest, String> {
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
    if decision.kind != "run_tests" {
        return Err(format!(
            "Scheduler selected `{}` instead of test execution.",
            decision.kind
        ));
    }
    let approval_id = decision
        .approval_ids
        .first()
        .ok_or_else(|| "Scheduler did not return an executable test approval.".to_string())?;
    let command = test_command_for_run(threads, plan_db, &request.run_id)?
        .ok_or_else(|| "Persisted runnable test command was not found.".to_string())?;
    let project = project_for_run(threads, plan_db, &request.run_id)?;
    Ok(TestRunRequest {
        approval_id: approval_id.clone(),
        approved_roots: project.approved_roots,
        args: command.args,
        completed_at: None,
        created_at_ms: request.now_ms,
        program: command.program,
        run_id: request.run_id.clone(),
        started_at: request.started_at.clone(),
        timeout_ms: request.timeout_ms.unwrap_or(DEFAULT_TEST_TIMEOUT_MS),
        working_directory: project.path,
    })
}

pub(crate) fn execute_scheduler_test_record(
    thread_store: &mut ThreadRunStore,
    test_store: &mut TestRunnerBridgeStore,
    approvals: &ApprovalEngine,
    request: TestRunRequest,
    started_at: &str,
    updated_at: &str,
) -> Result<AgentTestExecutionBridgeView, String> {
    approvals
        .assert_can_execute_action_for_run(
            &request.approval_id,
            request.created_at_ms,
            RiskyAction::TerminalCommand,
            &request.run_id,
        )
        .map_err(|error| format!("Test execution approval blocked: {error:?}"))?;
    move_thread_to_testing(thread_store, &request.run_id, started_at)?;
    let view = execute_test_run_record(thread_store, test_store, approvals, request)?;
    move_thread_after_test(thread_store, &view.run_id, &view.status, updated_at)?;
    Ok(view)
}

fn project_for_run(
    threads: &ThreadRunStore,
    database_path: &Path,
    run_id: &str,
) -> Result<crate::workspace_bridge::WorkspaceProjectView, String> {
    let record = threads
        .records
        .iter()
        .find(|item| item.run_id == run_id)
        .ok_or_else(|| "Thread run record was not found.".to_string())?;
    let project =
        crate::workspace_persistence::load_project_by_id(database_path, &record.project_id)?
            .ok_or_else(|| "Persisted workspace project was not found.".to_string())?;
    if project.approved_roots.is_empty() {
        return Err("Test execution requires a persisted approved root.".to_string());
    }
    Ok(project)
}

fn move_thread_to_testing(
    store: &mut ThreadRunStore,
    run_id: &str,
    updated_at: &str,
) -> Result<(), String> {
    let status = thread_status_for_run(store, run_id)?;
    match status {
        ThreadStatus::Testing => {}
        ThreadStatus::Building => set_thread_status(store, run_id, ThreadStatus::Testing)?,
        ThreadStatus::Planning
        | ThreadStatus::WaitingForApproval
        | ThreadStatus::Reviewing
        | ThreadStatus::Blocked => {
            set_thread_status(store, run_id, ThreadStatus::Building)?;
            set_thread_status(store, run_id, ThreadStatus::Testing)?;
        }
        _ => return Err("Test execution cannot move this thread to testing.".to_string()),
    }
    set_record_timestamp(store, run_id, updated_at)
}

fn move_thread_after_test(
    store: &mut ThreadRunStore,
    run_id: &str,
    status: &str,
    updated_at: &str,
) -> Result<(), String> {
    match status {
        "completed" => set_thread_status(store, run_id, ThreadStatus::Reviewing)?,
        "failed" => set_thread_status(store, run_id, ThreadStatus::Failed)?,
        _ => return Err("Scheduler test step did not complete execution.".to_string()),
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

fn validate_request(request: &AgentTestStepRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty()
        || request.now_ms == 0
        || request.started_at.trim().is_empty()
        || request.updated_at.trim().is_empty()
        || request.timeout_ms == Some(0)
    {
        return Err("Test step requires run, clock, timestamps, and non-zero timeout.".to_string());
    }
    Ok(())
}
