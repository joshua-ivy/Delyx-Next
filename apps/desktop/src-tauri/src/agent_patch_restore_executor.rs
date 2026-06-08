use crate::agent_executor::{AgentExecutionResult, AgentExecutionStatus};
use crate::agent_run::{
    AgentRunError, AgentRunLedger, AgentRunStatus, EvidenceRecordInput, EvidenceRelevance,
};
use crate::approval::{ApprovalEngine, ApprovalError, RiskyAction};
use crate::patch_bridge::{PatchBridgeStore, PatchProposalView};
use crate::patch_restore_bridge::{restore_patch_record, PatchRestoreRequest};

pub fn execute_patch_restore_node(
    ledger: &mut AgentRunLedger,
    patches: &mut PatchBridgeStore,
    approvals: &ApprovalEngine,
    request: PatchRestoreRequest,
) -> Result<AgentExecutionResult, String> {
    let proposal = patches
        .records
        .iter()
        .find(|record| record.id == request.proposal_id)
        .cloned()
        .ok_or_else(|| "Patch proposal not found.".to_string())?;
    match approvals.assert_can_execute_action_for_run(
        &request.approval_id,
        request.created_at_ms,
        RiskyAction::FileWrite,
        &proposal.run_id,
    ) {
        Ok(()) => run_patch_restore(ledger, patches, approvals, request, &proposal.run_id),
        Err(ApprovalError::NotApproved) => wait_for_restore_approval(ledger, &proposal, &request),
        Err(error) => fail_without_node(
            ledger,
            &proposal.run_id,
            &format!("Patch restore approval blocked: {error:?}"),
        ),
    }
}

fn run_patch_restore(
    ledger: &mut AgentRunLedger,
    patches: &mut PatchBridgeStore,
    approvals: &ApprovalEngine,
    request: PatchRestoreRequest,
    run_id: &str,
) -> Result<AgentExecutionResult, String> {
    let node = ledger
        .append_node(run_id, "tool_execution", "Restore patch checkpoint")
        .map_err(agent_error)?;
    ledger
        .append_event(
            run_id,
            "patch_restore.started",
            "Patch restore node started.",
        )
        .map_err(agent_error)?;
    match restore_patch_record(patches, approvals, request) {
        Ok(restored) => {
            mark_node(ledger, run_id, &node.id, AgentRunStatus::Completed)?;
            ledger
                .record_artifact(run_id, "patch_restore", &restored.id)
                .map_err(agent_error)?;
            ledger
                .record_evidence_detail(run_id, patch_restore_evidence(&restored.id))
                .map_err(agent_error)?;
            ledger
                .append_event(
                    run_id,
                    "patch_restore.completed",
                    &format!("Patch proposal {} restored.", restored.id),
                )
                .map_err(agent_error)?;
            Ok(AgentExecutionResult {
                message: format!("Patch proposal {} restored.", restored.id),
                patch_id: Some(restored.id),
                run_id: run_id.to_string(),
                status: AgentExecutionStatus::Completed,
            })
        }
        Err(error) => fail_with_node(ledger, run_id, &node.id, &error),
    }
}

fn wait_for_restore_approval(
    ledger: &mut AgentRunLedger,
    proposal: &PatchProposalView,
    request: &PatchRestoreRequest,
) -> Result<AgentExecutionResult, String> {
    ledger
        .wait_for_approval(&proposal.run_id, &request.approval_id)
        .map_err(agent_error)?;
    Ok(AgentExecutionResult {
        message: format!("Waiting for approval {}.", request.approval_id),
        patch_id: Some(proposal.id.clone()),
        run_id: proposal.run_id.clone(),
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

fn patch_restore_evidence(patch_id: &str) -> EvidenceRecordInput {
    EvidenceRecordInput {
        hash: None,
        quote: None,
        relevance: Some(EvidenceRelevance {
            reason: "Patch checkpoint was restored by this execution node.".to_string(),
            relationship: "direct_implementation".to_string(),
            score: 5,
        }),
        retrieved_at: String::new(),
        source_id: patch_id.to_string(),
        source_kind: "diff".to_string(),
        title: format!("Restored patch proposal {patch_id}"),
        uri: None,
    }
}

fn agent_error(error: AgentRunError) -> String {
    format!("{error:?}")
}
