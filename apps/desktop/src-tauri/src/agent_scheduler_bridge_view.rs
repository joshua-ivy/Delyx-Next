use crate::agent_scheduler::AgentScheduleDecision;
use crate::agent_scheduler_bridge::AgentScheduleDecisionView;

pub(crate) fn decision_view(
    run_id: &str,
    decision: AgentScheduleDecision,
) -> AgentScheduleDecisionView {
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
        AgentScheduleDecision::RepairRequested {
            finding_id,
            review_report_id,
        } => {
            let mut output = view(
                "repair_requested",
                run_id,
                format!("Repair requested from review {review_report_id} finding {finding_id}."),
            );
            output.finding_id = Some(finding_id);
            output.review_report_id = Some(review_report_id);
            output
        }
        AgentScheduleDecision::RequestPatchApplyApproval { proposal_id } => {
            let mut output = view(
                "request_patch_apply_approval",
                run_id,
                format!("Patch proposal {proposal_id} needs apply approval before disk write."),
            );
            output.proposal_id = Some(proposal_id);
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
        AgentScheduleDecision::RunPatchDraft { approval_id } => {
            let mut output = view(
                "run_patch_draft",
                run_id,
                format!("Approved plan {approval_id} is ready for PatchDraftAgent."),
            );
            output.approval_ids = vec![approval_id];
            output
        }
        AgentScheduleDecision::RunPatchApply {
            proposal_id,
            approval_id,
        } => {
            let mut output = view(
                "run_patch_apply",
                run_id,
                format!("Patch proposal {proposal_id} has apply approval {approval_id}."),
            );
            output.approval_ids = vec![approval_id];
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
        AgentScheduleDecision::RunTests {
            approval_id,
            reason,
        } => {
            let mut output = view("run_tests", run_id, reason);
            if let Some(approval_id) = approval_id {
                output.approval_ids = vec![approval_id];
            }
            output
        }
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
        finding_id: None,
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
