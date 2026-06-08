use crate::agent_run::{AgentRunError, AgentRunLedger, AgentRunStatus};
use crate::patch_bridge::{DiffLineView, PatchProposalView};
use crate::review_bridge::{
    create_review_record, DiffLineReviewInput, PatchFileReviewInput, PatchReviewInput,
    ReviewBridgeStore, ReviewCreateRequest, ReviewReportView, TestReviewInput,
};
use crate::test_runner_bridge::TestArtifactView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentReviewExecutionStatus {
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentReviewExecutionResult {
    pub status: AgentReviewExecutionStatus,
    pub run_id: String,
    pub review_report_id: Option<String>,
    pub message: String,
}

pub fn execute_review_node(
    ledger: &mut AgentRunLedger,
    reviews: &mut ReviewBridgeStore,
    run_id: String,
    patches: Vec<PatchProposalView>,
    tests: Vec<TestArtifactView>,
) -> Result<AgentReviewExecutionResult, String> {
    validate_run_id(&run_id)?;
    let node = ledger
        .append_node(&run_id, "diff_review", "Review patch and test artifacts")
        .map_err(agent_error)?;
    ledger
        .append_event(
            &run_id,
            "review.started",
            "Review node started from persisted artifacts.",
        )
        .map_err(agent_error)?;
    match create_review_record(reviews, review_request(&run_id, patches, tests)) {
        Ok(report) => finish_review_node(ledger, &run_id, &node.id, report),
        Err(error) => fail_with_node(ledger, &run_id, &node.id, &error),
    }
}

fn finish_review_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    node_id: &str,
    report: ReviewReportView,
) -> Result<AgentReviewExecutionResult, String> {
    mark_node(ledger, run_id, node_id, AgentRunStatus::Completed)?;
    ledger
        .record_artifact(run_id, "review_report", &report.id)
        .map_err(agent_error)?;
    ledger
        .append_event(
            run_id,
            "review.completed",
            &format!(
                "Review report {} captured with {} finding(s).",
                report.id,
                report.findings.len()
            ),
        )
        .map_err(agent_error)?;
    Ok(AgentReviewExecutionResult {
        message: format!(
            "Review report {} captured with {} finding(s).",
            report.id,
            report.findings.len()
        ),
        review_report_id: Some(report.id),
        run_id: run_id.to_string(),
        status: AgentReviewExecutionStatus::Completed,
    })
}

fn fail_with_node(
    ledger: &mut AgentRunLedger,
    run_id: &str,
    node_id: &str,
    message: &str,
) -> Result<AgentReviewExecutionResult, String> {
    mark_node(ledger, run_id, node_id, AgentRunStatus::Failed)?;
    let _ = ledger.append_event(run_id, "agent_executor.failed", message);
    ledger.fail_run(run_id, message).map_err(agent_error)?;
    Ok(AgentReviewExecutionResult {
        message: message.to_string(),
        review_report_id: None,
        run_id: run_id.to_string(),
        status: AgentReviewExecutionStatus::Failed,
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

fn review_request(
    run_id: &str,
    patches: Vec<PatchProposalView>,
    tests: Vec<TestArtifactView>,
) -> ReviewCreateRequest {
    ReviewCreateRequest {
        patches: patches.into_iter().map(patch_review_input).collect(),
        run_id: run_id.to_string(),
        tests: tests.into_iter().map(test_review_input).collect(),
    }
}

fn patch_review_input(patch: PatchProposalView) -> PatchReviewInput {
    PatchReviewInput {
        approval_id: patch.approval_id,
        files: patch
            .files
            .into_iter()
            .map(|file| PatchFileReviewInput {
                diff: file.diff.into_iter().map(diff_review_input).collect(),
                path: file.path,
            })
            .collect(),
        id: patch.id,
        run_id: patch.run_id,
        status: patch.status,
    }
}

fn diff_review_input(line: DiffLineView) -> DiffLineReviewInput {
    DiffLineReviewInput {
        kind: line.kind,
        text: line.text,
    }
}

fn test_review_input(test: TestArtifactView) -> TestReviewInput {
    TestReviewInput {
        approval_id: test.approval_id,
        command: test.command,
        cwd: test.cwd,
        duration_ms: test.duration_ms,
        exit_code: test.exit_code,
        failure_summary: test.failure_summary,
        id: test.id,
        run_id: test.run_id,
        status: Some(test.status),
        stderr: test.stderr,
        stdout: test.stdout,
    }
}

fn validate_run_id(run_id: &str) -> Result<(), String> {
    if run_id.trim().is_empty() {
        return Err("Review execution requires a run ID.".to_string());
    }
    Ok(())
}

fn agent_error(error: AgentRunError) -> String {
    format!("{error:?}")
}
