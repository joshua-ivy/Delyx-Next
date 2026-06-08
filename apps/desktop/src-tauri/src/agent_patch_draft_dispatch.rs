use crate::agent_patch_draft_bridge::{
    execute_patch_draft_record, AgentPatchDraftBridgeView, AgentPatchDraftExecuteRequest,
};
use crate::agent_scheduler_bridge::{
    schedule_next_record, AgentScheduleDecisionView, AgentScheduleRequest,
};
use crate::approval_bridge::ApprovalBridgeState;
use crate::patch_bridge::PatchBridgeState;
use crate::review_bridge::ReviewBridgeState;
use crate::test_runner_bridge::TestRunnerBridgeState;
use crate::thread_run_bridge::ThreadRunBridgeState;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPatchDraftDispatchRequest {
    pub execute: AgentPatchDraftExecuteRequest,
    pub has_supported_test_command: bool,
    #[serde(default)]
    pub patch_draft_approval_id: Option<String>,
    #[serde(default)]
    pub test_approval_id: Option<String>,
    pub now_ms: u64,
}

#[tauri::command]
pub fn agent_dispatch_patch_draft(
    threads: tauri::State<ThreadRunBridgeState>,
    patches: tauri::State<PatchBridgeState>,
    tests: tauri::State<TestRunnerBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: AgentPatchDraftDispatchRequest,
) -> Result<AgentPatchDraftBridgeView, String> {
    verify_scheduler_patch_draft(&threads, &patches, &tests, &reviews, &approvals, &request)?;
    execute_patch_draft_record(&threads, &patches, &approvals, request.execute)
}

pub(crate) fn verify_scheduler_patch_draft(
    threads: &ThreadRunBridgeState,
    patches: &PatchBridgeState,
    tests: &TestRunnerBridgeState,
    reviews: &ReviewBridgeState,
    approvals: &ApprovalBridgeState,
    request: &AgentPatchDraftDispatchRequest,
) -> Result<(), String> {
    let decision = approvals.with_engine(|engine| {
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
        let schedule_request = schedule_request(request);
        let run = thread_store
            .ledger
            .get_run(&request.execute.run_id)
            .map_err(|error| format!("{error:?}"))?;
        Ok::<AgentScheduleDecisionView, String>(schedule_next_record(
            run,
            engine,
            &patch_store,
            &test_store,
            &review_store,
            &schedule_request,
        ))
    })??;
    verify_patch_draft_decision(&decision, &request.execute.approval_id)
}

fn schedule_request(request: &AgentPatchDraftDispatchRequest) -> AgentScheduleRequest {
    AgentScheduleRequest {
        has_supported_test_command: request.has_supported_test_command,
        now_ms: request.now_ms,
        patch_draft_approval_id: request.patch_draft_approval_id.clone(),
        run_id: request.execute.run_id.clone(),
        test_approval_id: request.test_approval_id.clone(),
    }
}

pub(crate) fn verify_patch_draft_decision(
    decision: &AgentScheduleDecisionView,
    approval_id: &str,
) -> Result<(), String> {
    if decision.kind != "run_patch_draft" {
        return Err(format!(
            "Scheduler selected `{}` instead of PatchDraft.",
            decision.kind
        ));
    }
    if !decision.approval_ids.iter().any(|id| id == approval_id) {
        return Err("Scheduler did not verify the requested PatchDraft approval.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_draft_dispatch_requires_scheduler_verified_approval() {
        let mut decision = view("run_patch_draft", vec!["approval-1".to_string()]);

        assert!(verify_patch_draft_decision(&decision, "approval-1").is_ok());
        assert!(verify_patch_draft_decision(&decision, "other").is_err());

        decision.kind = "run_tests".to_string();
        assert!(verify_patch_draft_decision(&decision, "approval-1").is_err());
    }

    fn view(kind: &str, approval_ids: Vec<String>) -> AgentScheduleDecisionView {
        AgentScheduleDecisionView {
            approval_ids,
            finding_id: None,
            kind: kind.to_string(),
            message: "test".to_string(),
            patch_count: 0,
            proposal_id: None,
            review_report_id: None,
            run_id: "run-1".to_string(),
            status: None,
            test_count: 0,
        }
    }
}
