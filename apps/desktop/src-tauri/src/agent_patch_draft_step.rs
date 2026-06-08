use crate::agent_patch_draft_bridge::{execute_patch_draft_record, AgentPatchDraftBridgeView};
use crate::agent_patch_draft_context::{context_execute_request, AgentPatchDraftContextRequest};
use crate::agent_patch_draft_dispatch::{
    verify_scheduler_patch_draft, AgentPatchDraftDispatchRequest,
};
use crate::agent_scheduler_bridge::{schedule_next_record, AgentScheduleRequest};
use crate::agent_scheduler_patch_context::hydrate_patch_draft_request;
use crate::agent_scheduler_test_context::hydrate_schedule_request;
use crate::approval_bridge::ApprovalBridgeState;
use crate::patch_bridge::PatchBridgeState;
use crate::plan_bridge::PlanBridgeState;
use crate::review_bridge::ReviewBridgeState;
use crate::test_runner_bridge::TestRunnerBridgeState;
use crate::thread_run_bridge::ThreadRunBridgeState;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPatchDraftStepRequest {
    pub run_id: String,
    pub project_id: String,
    pub model: String,
    pub now_ms: u64,
    pub max_bytes_per_file: Option<usize>,
}

#[tauri::command]
pub fn agent_run_patch_draft_step(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    plans: tauri::State<PlanBridgeState>,
    request: AgentPatchDraftStepRequest,
) -> Result<AgentPatchDraftBridgeView, String> {
    let context = scheduler_patch_draft_context_request(
        &threads, &patches, &tests, &reviews, &approvals, &plans, &request,
    )?;
    let execute = context_execute_request(&threads, &reviews, &approvals, &plans, &context)?;
    let dispatch = AgentPatchDraftDispatchRequest {
        execute,
        has_supported_test_command: false,
        now_ms: request.now_ms,
        patch_draft_approval_id: None,
        test_approval_id: None,
    };
    verify_scheduler_patch_draft(
        &threads, &patches, &tests, &reviews, &approvals, &plans, &dispatch,
    )?;
    execute_patch_draft_record(&threads, &patches, &approvals, dispatch.execute)
}

pub(crate) fn scheduler_patch_draft_context_request(
    threads: &ThreadRunBridgeState,
    patches: &PatchBridgeState,
    tests: &TestRunnerBridgeState,
    reviews: &ReviewBridgeState,
    approvals: &ApprovalBridgeState,
    plans: &PlanBridgeState,
    request: &AgentPatchDraftStepRequest,
) -> Result<AgentPatchDraftContextRequest, String> {
    validate_step_request(request)?;
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
    let schedule_request = hydrate_patch_draft_request(
        &thread_store,
        &approval_store,
        &patch_store,
        &review_store,
        plans.database_path(),
        AgentScheduleRequest {
            has_supported_test_command: false,
            now_ms: request.now_ms,
            patch_apply_approval_id: None,
            patch_draft_approval_id: None,
            run_id: request.run_id.clone(),
            test_approval_id: None,
        },
    )?;
    let schedule_request = hydrate_schedule_request(
        &thread_store,
        &approval_store,
        plans.database_path(),
        schedule_request,
    )?;
    let run = thread_store
        .ledger
        .get_run(&request.run_id)
        .map_err(|error| format!("{error:?}"))?;
    let decision = schedule_next_record(
        run,
        &approval_store.engine,
        &patch_store,
        &test_store,
        &review_store,
        &schedule_request,
    );
    if decision.kind != "run_patch_draft" {
        return Err(format!(
            "Scheduler selected `{}` instead of PatchDraft.",
            decision.kind
        ));
    }
    let approval_id = decision
        .approval_ids
        .first()
        .ok_or_else(|| "Scheduler did not return a PatchDraft approval.".to_string())?;
    Ok(AgentPatchDraftContextRequest {
        approval_id: approval_id.clone(),
        has_supported_test_command: false,
        max_bytes_per_file: request.max_bytes_per_file,
        model: request.model.clone(),
        now_ms: request.now_ms,
        project_id: request.project_id.clone(),
        run_id: request.run_id.clone(),
        test_approval_id: None,
    })
}

fn validate_step_request(request: &AgentPatchDraftStepRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty()
        || request.project_id.trim().is_empty()
        || request.model.trim().is_empty()
        || request.now_ms == 0
    {
        return Err("PatchDraft step requires run, project, model, and clock.".to_string());
    }
    Ok(())
}
