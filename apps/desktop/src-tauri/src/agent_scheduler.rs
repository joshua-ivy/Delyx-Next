use crate::agent_run::{AgentRun, AgentRunError, AgentRunLedger, AgentRunStatus};
use crate::approval::{ApprovalEngine, ApprovalGateState, RiskyAction};
use crate::patch_bridge::{patch_snapshot_from_store, PatchBridgeStore, PatchProposalView};
use crate::review_bridge::{review_snapshot_from_store, ReviewBridgeStore};
use crate::test_runner_bridge::{test_snapshot_from_store, TestRunnerBridgeStore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentScheduleDecision {
    Blocked {
        reason: String,
    },
    Complete {
        reason: String,
    },
    ReadyForFinalSupport {
        review_report_id: String,
    },
    RepairRequested {
        review_report_id: String,
        finding_id: String,
    },
    ResumeAfterApproval {
        approval_id: String,
    },
    RunPatchDraft {
        approval_id: String,
    },
    RunPatchApply {
        proposal_id: String,
    },
    RunReview {
        patch_count: usize,
        test_count: usize,
    },
    RunTests {
        approval_id: Option<String>,
        reason: String,
    },
    Terminal {
        status: AgentRunStatus,
    },
    WaitForApproval {
        approval_ids: Vec<String>,
    },
}

pub struct AgentSchedulerContext<'a> {
    pub approvals: &'a ApprovalEngine,
    pub has_supported_test_command: bool,
    pub now_ms: u64,
    pub patch_draft_approval_id: Option<&'a str>,
    pub patches: &'a PatchBridgeStore,
    pub reviews: &'a ReviewBridgeStore,
    pub run: &'a AgentRun,
    pub test_approval_id: Option<&'a str>,
    pub tests: &'a TestRunnerBridgeStore,
}

pub fn schedule_next(context: AgentSchedulerContext<'_>) -> AgentScheduleDecision {
    if context.now_ms == 0 {
        return blocked("Scheduler requires a non-zero clock.");
    }
    if matches!(
        context.run.status,
        AgentRunStatus::Completed | AgentRunStatus::Failed
    ) {
        return AgentScheduleDecision::Terminal {
            status: context.run.status,
        };
    }
    if context.run.status == AgentRunStatus::WaitingForApproval {
        return approval_wait_decision(context.run, context.approvals, context.now_ms);
    }

    let patches = patch_snapshot_from_store(context.patches, &context.run.id);
    let tests = test_snapshot_from_store(context.tests, &context.run.id);
    let reviews = review_snapshot_from_store(context.reviews, &context.run.id);
    if let Some(proposal) = patches.iter().find(|patch| patch.status == "proposed") {
        return patch_apply_decision(proposal, context.approvals, context.now_ms);
    }
    if patches.iter().any(|patch| patch.status == "applied") && tests.is_empty() {
        return if context.has_supported_test_command {
            let approval_id = test_approval_id(&context);
            AgentScheduleDecision::RunTests {
                approval_id: approval_id.clone(),
                reason: test_reason(approval_id.as_deref()),
            }
        } else {
            blocked("An applied patch exists, but no supported test command is available.")
        };
    }
    if (!patches.is_empty() || !tests.is_empty()) && reviews.is_empty() {
        return AgentScheduleDecision::RunReview {
            patch_count: patches.len(),
            test_count: tests.len(),
        };
    }
    if let Some(report) = reviews.last() {
        return review_decision(report);
    }
    if let Some(approval_id) = context.patch_draft_approval_id {
        if patch_draft_approval_ready(
            context.approvals,
            approval_id,
            context.now_ms,
            &context.run.id,
        ) {
            return AgentScheduleDecision::RunPatchDraft {
                approval_id: approval_id.to_string(),
            };
        }
        return blocked("PatchDraft approval hint is not executable for this run.");
    }

    AgentScheduleDecision::Complete {
        reason: "No runnable persisted artifacts remain for this run.".to_string(),
    }
}

pub fn resume_waiting_run(
    ledger: &mut AgentRunLedger,
    approvals: &ApprovalEngine,
    run_id: &str,
    now_ms: u64,
) -> Result<AgentScheduleDecision, String> {
    let run = ledger.get_run(run_id).map_err(agent_error)?.clone();
    match approval_wait_decision(&run, approvals, now_ms) {
        AgentScheduleDecision::ResumeAfterApproval { approval_id } => {
            ledger
                .resume_after_approval(run_id, &approval_id)
                .map_err(agent_error)?;
            Ok(AgentScheduleDecision::ResumeAfterApproval { approval_id })
        }
        other => Ok(other),
    }
}

fn approval_wait_decision(
    run: &AgentRun,
    approvals: &ApprovalEngine,
    now_ms: u64,
) -> AgentScheduleDecision {
    let mut waiting = Vec::new();
    let mut ready = Vec::new();
    for proposal in approvals.list_proposals(&run.id) {
        match approvals.gate_state(&proposal.id, now_ms) {
            Ok(ApprovalGateState::Ready) => ready.push(proposal.id.clone()),
            Ok(ApprovalGateState::WaitingForApproval) => waiting.push(proposal.id.clone()),
            Ok(ApprovalGateState::Blocked) | Err(_) => {}
        }
    }
    match ready.len() {
        1 => AgentScheduleDecision::ResumeAfterApproval {
            approval_id: ready.remove(0),
        },
        0 if !waiting.is_empty() => AgentScheduleDecision::WaitForApproval {
            approval_ids: waiting,
        },
        0 => blocked("Run is waiting, but no executable or pending approval was found."),
        _ => blocked("Run has multiple ready approvals; scheduler will not guess."),
    }
}

fn patch_apply_decision(
    proposal: &PatchProposalView,
    _approvals: &ApprovalEngine,
    _now_ms: u64,
) -> AgentScheduleDecision {
    AgentScheduleDecision::RunPatchApply {
        proposal_id: proposal.id.clone(),
    }
}

fn patch_draft_approval_ready(
    approvals: &ApprovalEngine,
    approval_id: &str,
    now_ms: u64,
    run_id: &str,
) -> bool {
    approval_ready(
        approvals,
        approval_id,
        now_ms,
        RiskyAction::FileWrite,
        run_id,
    )
}

fn test_approval_id(context: &AgentSchedulerContext<'_>) -> Option<String> {
    let approval_id = context.test_approval_id?;
    approval_ready(
        context.approvals,
        approval_id,
        context.now_ms,
        RiskyAction::TerminalCommand,
        &context.run.id,
    )
    .then(|| approval_id.to_string())
}

fn approval_ready(
    approvals: &ApprovalEngine,
    approval_id: &str,
    now_ms: u64,
    action: RiskyAction,
    run_id: &str,
) -> bool {
    !approval_id.trim().is_empty()
        && approvals
            .assert_can_execute_action_for_run(approval_id, now_ms, action, run_id)
            .is_ok()
}

fn test_reason(approval_id: Option<&str>) -> String {
    match approval_id {
        Some(id) => format!("Approved test command {id} is ready to run."),
        None => "An applied patch exists and a supported test command is available.".to_string(),
    }
}

fn review_decision(report: &crate::review_bridge::ReviewReportView) -> AgentScheduleDecision {
    if report.decision == "revise_requested" {
        return match report.findings.first() {
            Some(finding) => AgentScheduleDecision::RepairRequested {
                finding_id: finding.id.clone(),
                review_report_id: report.id.clone(),
            },
            None => blocked("Review requested repair but has no linked finding."),
        };
    }
    if report.decision == "revert_requested" {
        return blocked(format!(
            "Review {} requested revert before final support.",
            report.id
        ));
    }
    if report.decision == "pending" && !report.findings.is_empty() {
        return blocked(format!(
            "Review {} has {} finding(s); request repair before final support.",
            report.id,
            report.findings.len()
        ));
    }
    AgentScheduleDecision::ReadyForFinalSupport {
        review_report_id: report.id.clone(),
    }
}

fn blocked(reason: impl Into<String>) -> AgentScheduleDecision {
    AgentScheduleDecision::Blocked {
        reason: reason.into(),
    }
}

fn agent_error(error: AgentRunError) -> String {
    format!("{error:?}")
}
