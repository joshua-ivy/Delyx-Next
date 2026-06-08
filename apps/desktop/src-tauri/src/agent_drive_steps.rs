use crate::agent_drive::AgentDriveContext;
use crate::agent_drive_types::{step, AgentDriveStepView};
use crate::agent_patch_apply_step::{
    execute_scheduler_patch_apply_record, scheduler_patch_apply_request, AgentPatchApplyStepRequest,
};
use crate::agent_review_step::{
    execute_scheduler_review_record, scheduler_review_request, AgentReviewStepRequest,
};
use crate::agent_test_step::{
    execute_scheduler_test_record, scheduler_test_request, AgentTestStepRequest,
};
use crate::thread_run_bridge::ThreadRunStore;
use crate::thread_run_final_answer::{
    finalize_thread_record, passed_tests, ThreadFinalAnswerRequest,
};
use crate::thread_run_final_support::{approval_support_records, FinalSupportInput};

pub(crate) fn run_patch_apply(
    context: &mut AgentDriveContext<'_>,
) -> Result<AgentDriveStepView, String> {
    let request = AgentPatchApplyStepRequest {
        now_ms: context.now_ms,
        run_id: context.run_id.clone(),
        updated_at: context.updated_at.clone(),
    };
    let apply = scheduler_patch_apply_request(
        context.threads,
        context.approvals,
        context.patches,
        context.tests,
        context.reviews,
        context.plan_db,
        &request,
    )?;
    let view = execute_scheduler_patch_apply_record(
        context.threads,
        context.patches,
        &context.approvals.engine,
        apply,
        &context.updated_at,
    )?;
    Ok(step("run_patch_apply", view.status, view.message))
}

pub(crate) fn run_tests(context: &mut AgentDriveContext<'_>) -> Result<AgentDriveStepView, String> {
    let request = AgentTestStepRequest {
        now_ms: context.now_ms,
        run_id: context.run_id.clone(),
        started_at: context.updated_at.clone(),
        timeout_ms: context.timeout_ms,
        updated_at: context.updated_at.clone(),
    };
    let test = scheduler_test_request(
        context.threads,
        context.approvals,
        context.patches,
        context.tests,
        context.reviews,
        context.plan_db,
        &request,
    )?;
    let view = execute_scheduler_test_record(
        context.threads,
        context.tests,
        &context.approvals.engine,
        test,
        &context.updated_at,
        &context.updated_at,
    )?;
    Ok(step("run_tests", view.status, view.message))
}

pub(crate) fn run_review(
    context: &mut AgentDriveContext<'_>,
) -> Result<AgentDriveStepView, String> {
    let request = AgentReviewStepRequest {
        now_ms: context.now_ms,
        run_id: context.run_id.clone(),
        updated_at: context.updated_at.clone(),
    };
    let review = scheduler_review_request(
        context.threads,
        context.approvals,
        context.patches,
        context.tests,
        context.reviews,
        context.plan_db,
        &request,
    )?;
    let view = execute_scheduler_review_record(
        context.threads,
        context.patches,
        context.tests,
        context.reviews,
        review,
        &context.updated_at,
    )?;
    Ok(step("run_review", view.status, view.message))
}

pub(crate) fn record_final_support(
    context: &mut AgentDriveContext<'_>,
    review_report_id: Option<String>,
) -> Result<AgentDriveStepView, String> {
    let summary = context.final_summary.as_deref().ok_or_else(|| {
        "Driver needs a final summary before recording final support.".to_string()
    })?;
    let thread_id = thread_id_for_run(context.threads, &context.run_id)?;
    let request = ThreadFinalAnswerRequest {
        summary: summary.to_string(),
        thread_id,
        updated_at: context.updated_at.clone(),
    };
    let support = FinalSupportInput {
        approval_records: approval_support_records(&context.approvals.engine, &context.run_id),
        test_artifacts: passed_tests(context.tests, &context.run_id),
    };
    finalize_thread_record(context.threads, request, support)?;
    Ok(step(
        "ready_for_final_support",
        "completed",
        format!(
            "Final support recorded{}.",
            review_report_id
                .map(|id| format!(" from review {id}"))
                .unwrap_or_default()
        ),
    ))
}

fn thread_id_for_run(store: &ThreadRunStore, run_id: &str) -> Result<String, String> {
    store
        .records
        .iter()
        .find(|record| record.run_id == run_id)
        .map(|record| record.thread_id.clone())
        .ok_or_else(|| "Thread run record was not found.".to_string())
}
