use crate::agent_run::{
    AgentRunError, AgentRunLedger, AgentRunStatus, EvidenceRecordInput, EvidenceRelevance,
};
use crate::approval::{ApprovalEngine, ApprovalError, RiskyAction};
use crate::test_runner_bridge::{run_test_record, TestRunRequest, TestRunnerBridgeStore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentTestExecutionStatus {
    Completed,
    Failed,
    WaitingForApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTestExecutionResult {
    pub status: AgentTestExecutionStatus,
    pub run_id: String,
    pub test_artifact_id: Option<String>,
    pub message: String,
}

pub fn execute_test_run_node(
    ledger: &mut AgentRunLedger,
    tests: &mut TestRunnerBridgeStore,
    approvals: &ApprovalEngine,
    request: TestRunRequest,
) -> Result<AgentTestExecutionResult, String> {
    match approvals.assert_can_execute_action_for_run(
        &request.approval_id,
        request.created_at_ms,
        RiskyAction::TerminalCommand,
        &request.run_id,
    ) {
        Ok(()) => run_test_node(ledger, tests, approvals, request),
        Err(ApprovalError::NotApproved) => wait_for_test_approval(ledger, &request),
        Err(error) => fail_without_node(
            ledger,
            &request.run_id,
            &format!("Test execution approval blocked: {error:?}"),
        ),
    }
}

fn run_test_node(
    ledger: &mut AgentRunLedger,
    tests: &mut TestRunnerBridgeStore,
    approvals: &ApprovalEngine,
    request: TestRunRequest,
) -> Result<AgentTestExecutionResult, String> {
    let run_id = request.run_id.clone();
    let node = ledger
        .append_node(&run_id, "test_execution", "Run approved test command")
        .map_err(agent_error)?;
    ledger
        .append_event(
            &run_id,
            "test_execution.started",
            "Approved test command started.",
        )
        .map_err(agent_error)?;
    match run_test_record(tests, approvals, request) {
        Ok(artifact) => finish_test_node(ledger, &run_id, &node.id, artifact),
        Err(error) => fail_with_node(ledger, &run_id, &node.id, &error),
    }
}

fn finish_test_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    node_id: &str,
    artifact: crate::test_runner_bridge::TestArtifactView,
) -> Result<AgentTestExecutionResult, String> {
    ledger
        .record_artifact(run_id, "test", &artifact.id)
        .map_err(agent_error)?;
    ledger
        .record_evidence_detail(run_id, test_evidence(&artifact.id, &artifact.status))
        .map_err(agent_error)?;
    if artifact.status == "passed" {
        mark_node(ledger, run_id, node_id, AgentRunStatus::Completed)?;
        ledger
            .append_event(
                run_id,
                "test_execution.completed",
                &format!("Test artifact {} passed.", artifact.id),
            )
            .map_err(agent_error)?;
        return Ok(result(
            AgentTestExecutionStatus::Completed,
            run_id,
            Some(artifact.id.clone()),
            format!("Test artifact {} passed.", artifact.id),
        ));
    }
    mark_node(ledger, run_id, node_id, AgentRunStatus::Failed)?;
    fail_without_node(
        ledger,
        run_id,
        &format!("Test artifact {} failed.", artifact.id),
    )
    .map(|mut result| {
        result.test_artifact_id = Some(artifact.id);
        result
    })
}

fn wait_for_test_approval(
    ledger: &mut AgentRunLedger,
    request: &TestRunRequest,
) -> Result<AgentTestExecutionResult, String> {
    ledger
        .wait_for_approval(&request.run_id, &request.approval_id)
        .map_err(agent_error)?;
    Ok(result(
        AgentTestExecutionStatus::WaitingForApproval,
        &request.run_id,
        None,
        format!("Waiting for approval {}.", request.approval_id),
    ))
}

fn fail_with_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    node_id: &str,
    message: &str,
) -> Result<AgentTestExecutionResult, String> {
    mark_node(ledger, run_id, node_id, AgentRunStatus::Failed)?;
    fail_without_node(ledger, run_id, message)
}

fn fail_without_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    message: &str,
) -> Result<AgentTestExecutionResult, String> {
    let _ = ledger.append_event(run_id, "agent_executor.failed", message);
    ledger.fail_run(run_id, message).map_err(agent_error)?;
    Ok(result(
        AgentTestExecutionStatus::Failed,
        run_id,
        None,
        message.to_string(),
    ))
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

fn test_evidence(artifact_id: &str, status: &str) -> EvidenceRecordInput {
    EvidenceRecordInput {
        hash: None,
        quote: None,
        relevance: Some(EvidenceRelevance {
            reason: format!("Test artifact completed with status {status}."),
            relationship: "test".to_string(),
            score: 5,
        }),
        retrieved_at: String::new(),
        source_id: artifact_id.to_string(),
        source_kind: "test".to_string(),
        title: format!("Test artifact {artifact_id}"),
        uri: None,
    }
}

fn result(
    status: AgentTestExecutionStatus,
    run_id: &str,
    test_artifact_id: Option<String>,
    message: String,
) -> AgentTestExecutionResult {
    AgentTestExecutionResult {
        message,
        run_id: run_id.to_string(),
        status,
        test_artifact_id,
    }
}

fn agent_error(error: AgentRunError) -> String {
    format!("{error:?}")
}
