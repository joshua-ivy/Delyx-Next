use crate::agent_run::{
    AgentRunError, AgentRunLedger, AgentRunStatus, EvidenceRecordInput, EvidenceRelevance,
};
use crate::approval::{ApprovalEngine, ApprovalError, RiskyAction};
use crate::patch_bridge::{propose_patch_record, PatchBridgeStore, PatchProposalRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentExecutionStatus {
    Completed,
    Failed,
    WaitingForApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentExecutionResult {
    pub status: AgentExecutionStatus,
    pub run_id: String,
    pub patch_id: Option<String>,
    pub message: String,
}

pub fn execute_patch_proposal_node(
    ledger: &mut AgentRunLedger,
    patches: &mut PatchBridgeStore,
    approvals: &ApprovalEngine,
    request: PatchProposalRequest,
    now_ms: u64,
) -> Result<AgentExecutionResult, String> {
    validate_request(&request, now_ms)?;
    match approvals.assert_can_execute_action_for_run(
        &request.approval_id,
        now_ms,
        RiskyAction::FileWrite,
        &request.run_id,
    ) {
        Ok(()) => run_patch_proposal(ledger, patches, request),
        Err(ApprovalError::NotApproved) => wait_for_approval(ledger, &request),
        Err(error) => fail_without_node(
            ledger,
            &request.run_id,
            &format!("Patch proposal approval blocked: {error:?}"),
        ),
    }
}

fn run_patch_proposal(
    ledger: &mut AgentRunLedger,
    patches: &mut PatchBridgeStore,
    request: PatchProposalRequest,
) -> Result<AgentExecutionResult, String> {
    let node = ledger
        .append_node(&request.run_id, "patch_proposal", "Create patch proposal")
        .map_err(agent_error)?;
    ledger
        .append_event(
            &request.run_id,
            "patch_proposal.started",
            "Patch proposal node started.",
        )
        .map_err(agent_error)?;
    match propose_patch_record(patches, request.clone()) {
        Ok(proposal) => {
            mark_node(ledger, &request.run_id, &node.id, AgentRunStatus::Completed)?;
            ledger
                .record_artifact(&request.run_id, "patch_proposal", &proposal.id)
                .map_err(agent_error)?;
            ledger
                .record_evidence_detail(&request.run_id, patch_evidence(&proposal.id))
                .map_err(agent_error)?;
            ledger
                .append_event(
                    &request.run_id,
                    "patch_proposal.completed",
                    &format!("Patch proposal {} captured.", proposal.id),
                )
                .map_err(agent_error)?;
            Ok(AgentExecutionResult {
                message: format!("Patch proposal {} captured.", proposal.id),
                patch_id: Some(proposal.id),
                run_id: request.run_id,
                status: AgentExecutionStatus::Completed,
            })
        }
        Err(error) => fail_with_node(ledger, &request.run_id, &node.id, &error),
    }
}

fn wait_for_approval(
    ledger: &mut AgentRunLedger,
    request: &PatchProposalRequest,
) -> Result<AgentExecutionResult, String> {
    ledger
        .wait_for_approval(&request.run_id, &request.approval_id)
        .map_err(agent_error)?;
    Ok(AgentExecutionResult {
        message: format!("Waiting for approval {}.", request.approval_id),
        patch_id: None,
        run_id: request.run_id.clone(),
        status: AgentExecutionStatus::WaitingForApproval,
    })
}

fn fail_with_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    node_id: &str,
    message: &str,
) -> Result<AgentExecutionResult, String> {
    mark_node(ledger, run_id, node_id, AgentRunStatus::Failed)?;
    fail_without_node(ledger, run_id, message)
}

fn fail_without_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    message: &str,
) -> Result<AgentExecutionResult, String> {
    let _ = ledger.append_event(run_id, "agent_executor.failed", message);
    ledger.fail_run(run_id, message).map_err(agent_error)?;
    Ok(AgentExecutionResult {
        message: message.to_string(),
        patch_id: None,
        run_id: run_id.to_string(),
        status: AgentExecutionStatus::Failed,
    })
}

fn mark_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    node_id: &str,
    status: AgentRunStatus,
) -> Result<(), String> {
    let run = ledger.run_mut(run_id).map_err(agent_error)?;
    let node = run
        .nodes
        .iter_mut()
        .find(|node| node.id == node_id)
        .ok_or_else(|| "Agent executor node not found.".to_string())?;
    node.status = status;
    Ok(())
}

fn patch_evidence(patch_id: &str) -> EvidenceRecordInput {
    EvidenceRecordInput {
        hash: None,
        quote: None,
        relevance: Some(EvidenceRelevance {
            reason: "Patch proposal diff was produced by this execution node.".to_string(),
            relationship: "direct_implementation".to_string(),
            score: 5,
        }),
        retrieved_at: String::new(),
        source_id: patch_id.to_string(),
        source_kind: "diff".to_string(),
        title: format!("Patch proposal {patch_id}"),
        uri: None,
    }
}

fn validate_request(request: &PatchProposalRequest, now_ms: u64) -> Result<(), String> {
    if request.run_id.trim().is_empty() || request.approval_id.trim().is_empty() || now_ms == 0 {
        return Err("Patch proposal execution requires run, approval, and clock.".to_string());
    }
    Ok(())
}

fn agent_error(error: AgentRunError) -> String {
    format!("{error:?}")
}
