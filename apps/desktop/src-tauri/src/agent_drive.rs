use crate::agent_drive_steps::{record_final_support, run_patch_apply, run_review, run_tests};
use crate::agent_drive_types::{
    outcome, step, stop, stop_with_approvals, stop_with_proposal, stop_with_review,
    stop_with_status, AgentDriveOutcomeView, AgentDriveStopView,
};
use crate::agent_scheduler_bridge::{
    resume_waiting_run_record, schedule_next_record, AgentScheduleDecisionView,
    AgentScheduleRequest,
};
use crate::agent_scheduler_patch_context::hydrate_patch_draft_request;
use crate::agent_scheduler_test_context::hydrate_schedule_request;
use crate::approval_bridge::ApprovalBridgeStore;
use crate::patch_bridge::PatchBridgeStore;
use crate::review_bridge::ReviewBridgeStore;
use crate::test_runner_bridge::TestRunnerBridgeStore;
use crate::thread_run_bridge::ThreadRunStore;
use std::path::Path;

pub const MAX_DRIVE_STEPS: usize = 24;

pub struct AgentDriveContext<'a> {
    pub approvals: &'a ApprovalBridgeStore,
    pub final_summary: Option<String>,
    pub now_ms: u64,
    pub patches: &'a mut PatchBridgeStore,
    pub plan_db: &'a Path,
    pub reviews: &'a mut ReviewBridgeStore,
    pub run_id: String,
    pub tests: &'a mut TestRunnerBridgeStore,
    pub threads: &'a mut ThreadRunStore,
    pub timeout_ms: Option<u64>,
    pub updated_at: String,
}

pub fn drive_run(
    context: &mut AgentDriveContext<'_>,
    mut after_progress: impl FnMut(
        &ThreadRunStore,
        &PatchBridgeStore,
        &TestRunnerBridgeStore,
        &ReviewBridgeStore,
    ) -> Result<(), String>,
) -> Result<AgentDriveOutcomeView, String> {
    validate_context(context)?;
    let mut steps = Vec::new();
    let mut last_progress_signature: Option<String> = None;
    for _ in 0..MAX_DRIVE_STEPS {
        let decision = next_decision(context)?;
        let signature = decision_signature(&decision);
        if last_progress_signature.as_deref() == Some(&signature) {
            return Ok(outcome(
                context,
                steps,
                stop("blocked", "Driver made no progress."),
            ));
        }
        match decision.kind.as_str() {
            "run_patch_apply" => {
                last_progress_signature = Some(signature);
                let progress = run_patch_apply(context)?;
                let halted = progress.status == "failed";
                steps.push(progress);
                persist_progress(context, &mut after_progress)?;
                if halted {
                    return Ok(outcome(context, steps, halt_stop()));
                }
            }
            "run_tests" if !decision.approval_ids.is_empty() => {
                last_progress_signature = Some(signature);
                let progress = run_tests(context)?;
                let halted = progress.status == "failed";
                steps.push(progress);
                persist_progress(context, &mut after_progress)?;
                if halted {
                    return Ok(outcome(context, steps, halt_stop()));
                }
            }
            "run_review" => {
                last_progress_signature = Some(signature);
                let progress = run_review(context)?;
                let halted = progress.status == "failed";
                steps.push(progress);
                persist_progress(context, &mut after_progress)?;
                if halted {
                    return Ok(outcome(context, steps, halt_stop()));
                }
            }
            "ready_for_final_support" => {
                if final_summary_missing(context) {
                    return Ok(outcome(
                        context,
                        steps,
                        stop("needs_final_summary", decision.message),
                    ));
                }
                steps.push(record_final_support(
                    context,
                    decision.review_report_id.clone(),
                )?);
                persist_progress(context, &mut after_progress)?;
                return Ok(outcome(context, steps, stop("completed", "Run completed.")));
            }
            "resume_after_approval" => {
                last_progress_signature = Some(signature);
                let approval_id = decision.approval_ids.first().cloned().unwrap_or_default();
                let schedule = schedule_request(context)?;
                let resumed = resume_waiting_run_record(
                    context.threads,
                    &context.approvals.engine,
                    context.patches,
                    context.tests,
                    context.reviews,
                    &schedule,
                )?;
                steps.push(step(
                    "resume_after_approval",
                    "completed",
                    format!("Resumed after approval {approval_id}."),
                ));
                persist_progress(context, &mut after_progress)?;
                if resumed.kind == "complete" {
                    return Ok(outcome(context, steps, stop("completed", resumed.message)));
                }
            }
            "wait_for_approval" => {
                return Ok(outcome(
                    context,
                    steps,
                    stop_with_approvals(
                        "awaiting_approval",
                        decision.message,
                        decision.approval_ids,
                    ),
                ));
            }
            "request_patch_apply_approval" => {
                return Ok(outcome(
                    context,
                    steps,
                    stop_with_proposal(
                        "needs_patch_apply_approval",
                        decision.message,
                        decision.proposal_id,
                    ),
                ));
            }
            "repair_requested" => {
                return Ok(outcome(
                    context,
                    steps,
                    stop_with_review(
                        "repair_requested",
                        decision.message,
                        decision.review_report_id,
                        decision.finding_id,
                    ),
                ));
            }
            "run_patch_draft" => {
                return Ok(outcome(
                    context,
                    steps,
                    stop("patch_draft_ready", decision.message),
                ));
            }
            "run_tests" => {
                return Ok(outcome(
                    context,
                    steps,
                    stop("needs_test_approval", decision.message),
                ));
            }
            "blocked" => return Ok(outcome(context, steps, stop("blocked", decision.message))),
            "complete" => return Ok(outcome(context, steps, stop("completed", decision.message))),
            "terminal" => {
                return Ok(outcome(
                    context,
                    steps,
                    stop_with_status("terminal", decision.message, decision.status),
                ));
            }
            _ => return Ok(outcome(context, steps, stop("blocked", decision.message))),
        }
    }
    Ok(outcome(
        context,
        steps,
        stop("step_budget_exhausted", "Driver step budget was exhausted."),
    ))
}

fn persist_progress(
    context: &mut AgentDriveContext<'_>,
    after_progress: &mut impl FnMut(
        &ThreadRunStore,
        &PatchBridgeStore,
        &TestRunnerBridgeStore,
        &ReviewBridgeStore,
    ) -> Result<(), String>,
) -> Result<(), String> {
    after_progress(
        context.threads,
        context.patches,
        context.tests,
        context.reviews,
    )
}

fn halt_stop() -> AgentDriveStopView {
    stop("failed", "Driver halted on a failed node.")
}

fn next_decision(context: &AgentDriveContext<'_>) -> Result<AgentScheduleDecisionView, String> {
    let request = schedule_request(context)?;
    let run = context
        .threads
        .ledger
        .get_run(&context.run_id)
        .map_err(|error| format!("{error:?}"))?;
    Ok(schedule_next_record(
        run,
        &context.approvals.engine,
        context.patches,
        context.tests,
        context.reviews,
        &request,
    ))
}

fn schedule_request(context: &AgentDriveContext<'_>) -> Result<AgentScheduleRequest, String> {
    let request = AgentScheduleRequest {
        has_supported_test_command: false,
        now_ms: context.now_ms,
        patch_apply_approval_id: None,
        patch_draft_approval_id: None,
        run_id: context.run_id.clone(),
        test_approval_id: None,
    };
    let request = hydrate_patch_draft_request(
        context.threads,
        context.approvals,
        context.patches,
        context.reviews,
        context.plan_db,
        request,
    )?;
    hydrate_schedule_request(context.threads, context.approvals, context.plan_db, request)
}

fn validate_context(context: &AgentDriveContext<'_>) -> Result<(), String> {
    if context.run_id.trim().is_empty()
        || context.now_ms == 0
        || context.updated_at.trim().is_empty()
        || context.timeout_ms == Some(0)
    {
        return Err("Drive run requires run, clock, timestamp, and non-zero timeout.".to_string());
    }
    Ok(())
}

fn final_summary_missing(context: &AgentDriveContext<'_>) -> bool {
    context
        .final_summary
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
}

fn decision_signature(decision: &AgentScheduleDecisionView) -> String {
    format!(
        "{}:{:?}:{:?}:{:?}:{:?}",
        decision.kind,
        decision.approval_ids,
        decision.proposal_id,
        decision.review_report_id,
        decision.status
    )
}
