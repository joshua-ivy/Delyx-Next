use crate::agent_scheduler::{
    resume_waiting_run, schedule_next, AgentScheduleDecision, AgentSchedulerContext,
};
use crate::approval_bridge::ApprovalBridgeState;
use crate::patch_bridge::PatchBridgeState;
use crate::review_bridge::ReviewBridgeState;
use crate::test_runner_bridge::TestRunnerBridgeState;
use crate::thread_run_bridge::ThreadRunBridgeState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentScheduleRequest {
    pub run_id: String,
    pub has_supported_test_command: bool,
    pub now_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentScheduleDecisionView {
    pub kind: String,
    pub run_id: String,
    pub message: String,
    pub approval_ids: Vec<String>,
    pub proposal_id: Option<String>,
    pub review_report_id: Option<String>,
    pub patch_count: usize,
    pub test_count: usize,
    pub status: Option<String>,
}

#[tauri::command]
pub fn agent_schedule_next(
    threads: tauri::State<ThreadRunBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    request: AgentScheduleRequest,
) -> Result<AgentScheduleDecisionView, String> {
    approvals.with_engine(|engine| {
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
        let run = thread_store
            .ledger
            .get_run(&request.run_id)
            .map_err(|error| format!("{error:?}"))?;
        Ok(schedule_next_record(
            run,
            engine,
            &patch_store,
            &test_store,
            &review_store,
            &request,
        ))
    })?
}

#[tauri::command]
pub fn agent_resume_waiting_run(
    threads: tauri::State<ThreadRunBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    request: AgentScheduleRequest,
) -> Result<AgentScheduleDecisionView, String> {
    approvals.with_engine(|engine| {
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
        let review_store = reviews
            .store
            .lock()
            .map_err(|_| "Review bridge lock failed.".to_string())?;
        let decision = resume_waiting_run_record(
            &mut thread_store,
            engine,
            &patch_store,
            &test_store,
            &review_store,
            &request,
        )?;
        threads.persist(&thread_store)?;
        Ok(decision)
    })?
}

pub fn schedule_next_record(
    run: &crate::agent_run::AgentRun,
    approvals: &crate::approval::ApprovalEngine,
    patches: &crate::patch_bridge::PatchBridgeStore,
    tests: &crate::test_runner_bridge::TestRunnerBridgeStore,
    reviews: &crate::review_bridge::ReviewBridgeStore,
    request: &AgentScheduleRequest,
) -> AgentScheduleDecisionView {
    decision_view(
        &run.id,
        schedule_next(AgentSchedulerContext {
            approvals,
            has_supported_test_command: request.has_supported_test_command,
            now_ms: request.now_ms,
            patches,
            reviews,
            run,
            tests,
        }),
    )
}

pub fn resume_waiting_run_record(
    thread_store: &mut crate::thread_run_bridge::ThreadRunStore,
    approvals: &crate::approval::ApprovalEngine,
    patches: &crate::patch_bridge::PatchBridgeStore,
    tests: &crate::test_runner_bridge::TestRunnerBridgeStore,
    reviews: &crate::review_bridge::ReviewBridgeStore,
    request: &AgentScheduleRequest,
) -> Result<AgentScheduleDecisionView, String> {
    let decision = resume_waiting_run(
        &mut thread_store.ledger,
        approvals,
        &request.run_id,
        request.now_ms,
    )?;
    if !matches!(decision, AgentScheduleDecision::ResumeAfterApproval { .. }) {
        return Ok(decision_view(&request.run_id, decision));
    }
    let run = thread_store
        .ledger
        .get_run(&request.run_id)
        .map_err(|error| format!("{error:?}"))?;
    let next = schedule_next_record(run, approvals, patches, tests, reviews, request);
    if next.kind == "complete" {
        return Ok(decision_view(&request.run_id, decision));
    }
    Ok(next)
}

fn decision_view(run_id: &str, decision: AgentScheduleDecision) -> AgentScheduleDecisionView {
    match decision {
        AgentScheduleDecision::Blocked { reason } => view("blocked", run_id, reason),
        AgentScheduleDecision::Complete { reason } => view("complete", run_id, reason),
        AgentScheduleDecision::ReadyForFinalSupport { review_report_id } => {
            let mut output = view(
                "ready_for_final_support",
                run_id,
                format!("Review {review_report_id} is ready for final support synthesis."),
            );
            output.review_report_id = Some(review_report_id);
            output
        }
        AgentScheduleDecision::ResumeAfterApproval { approval_id } => {
            let mut output = view(
                "resume_after_approval",
                run_id,
                format!("Approval {approval_id} is ready; run can resume."),
            );
            output.approval_ids = vec![approval_id];
            output
        }
        AgentScheduleDecision::RunPatchApply { proposal_id } => {
            let mut output = view(
                "run_patch_apply",
                run_id,
                format!("Patch proposal {proposal_id} is approved and ready to apply."),
            );
            output.proposal_id = Some(proposal_id);
            output
        }
        AgentScheduleDecision::RunReview {
            patch_count,
            test_count,
        } => {
            let mut output = view(
                "run_review",
                run_id,
                format!(
                    "Review is ready from {patch_count} patch and {test_count} test artifact(s)."
                ),
            );
            output.patch_count = patch_count;
            output.test_count = test_count;
            output
        }
        AgentScheduleDecision::RunTests { reason } => view("run_tests", run_id, reason),
        AgentScheduleDecision::Terminal { status } => {
            let mut output = view("terminal", run_id, format!("Run is {status:?}."));
            output.status = Some(format!("{status:?}"));
            output
        }
        AgentScheduleDecision::WaitForApproval { approval_ids } => {
            let mut output = view(
                "wait_for_approval",
                run_id,
                format!("Waiting for {} approval(s).", approval_ids.len()),
            );
            output.approval_ids = approval_ids;
            output
        }
    }
}

fn view(kind: &str, run_id: &str, message: impl Into<String>) -> AgentScheduleDecisionView {
    AgentScheduleDecisionView {
        approval_ids: Vec::new(),
        kind: kind.to_string(),
        message: message.into(),
        patch_count: 0,
        proposal_id: None,
        review_report_id: None,
        run_id: run_id.to_string(),
        status: None,
        test_count: 0,
    }
}
