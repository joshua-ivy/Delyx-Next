use crate::approval::ApprovalEngine;
use crate::approval_bridge::ApprovalBridgeState;
use crate::test_runner::{TestCommandInput, TestRunner, TestStatus};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct TestRunnerBridgeState {
    store: Mutex<TestRunnerBridgeStore>,
}

#[derive(Default)]
pub struct TestRunnerBridgeStore {
    artifacts: Vec<TestArtifactView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRunRequest {
    pub run_id: String,
    pub approval_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub approved_roots: Vec<String>,
    pub timeout_ms: u64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestArtifactView {
    pub id: String,
    pub run_id: String,
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub parsed_failures: Option<Vec<ParsedFailureView>>,
    pub started_at: String,
    pub completed_at: String,
    pub approval_id: Option<String>,
    pub status: String,
    pub failure_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedFailureView {
    pub id: String,
    pub message: String,
}

#[tauri::command]
pub fn test_run_approved(
    state: tauri::State<TestRunnerBridgeState>,
    approvals: tauri::State<ApprovalBridgeState>,
    request: TestRunRequest,
) -> Result<TestArtifactView, String> {
    approvals.with_engine(|engine| {
        let mut store = state.store.lock().map_err(|_| "Test bridge lock failed.".to_string())?;
        run_test_record(&mut store, engine, request)
    })?
}

#[tauri::command]
pub fn test_snapshot(
    state: tauri::State<TestRunnerBridgeState>,
    run_id: String,
) -> Result<Vec<TestArtifactView>, String> {
    let store = state.store.lock().map_err(|_| "Test bridge lock failed.".to_string())?;
    Ok(test_snapshot_from_store(&store, &run_id))
}

pub fn run_test_record(
    store: &mut TestRunnerBridgeStore,
    approvals: &ApprovalEngine,
    request: TestRunRequest,
) -> Result<TestArtifactView, String> {
    validate_request(&request)?;
    let roots = request.approved_roots.iter().map(PathBuf::from).collect::<Vec<_>>();
    let mut runner = TestRunner::new(roots).map_err(|error| format!("{error:?}"))?;
    let completed_at = request.completed_at.clone().unwrap_or_else(|| request.started_at.clone());
    let artifact = runner
        .run_approved_test(
            TestCommandInput {
                approval_id: request.approval_id,
                args: request.args,
                program: request.program,
                run_id: request.run_id,
                timeout_ms: request.timeout_ms,
                working_directory: PathBuf::from(request.working_directory),
            },
            request.created_at_ms,
            approvals,
        )
        .map_err(|error| format!("{error:?}"))?;
    let view = artifact_view(&artifact, request.started_at, completed_at);
    store.artifacts.push(view.clone());
    Ok(view)
}

pub fn test_snapshot_from_store(store: &TestRunnerBridgeStore, run_id: &str) -> Vec<TestArtifactView> {
    store.artifacts.iter().filter(|artifact| artifact.run_id == run_id).cloned().collect()
}

fn validate_request(request: &TestRunRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty() || request.approval_id.trim().is_empty() {
        return Err("Test run requires run and approval IDs.".to_string());
    }
    if request.approved_roots.is_empty() {
        return Err("Test run requires at least one approved root.".to_string());
    }
    Ok(())
}

fn artifact_view(
    artifact: &crate::test_runner::TestArtifact,
    started_at: String,
    completed_at: String,
) -> TestArtifactView {
    let parsed_failures = artifact.failure_summary.as_ref().map(|message| vec![ParsedFailureView {
        id: format!("{}-failure-1", artifact.id),
        message: message.clone(),
    }]);
    TestArtifactView {
        approval_id: Some(artifact.approval_id.clone()),
        command: artifact.command.clone(),
        completed_at,
        cwd: artifact.working_directory.display().to_string(),
        duration_ms: artifact.duration_ms,
        exit_code: artifact.exit_code,
        failure_summary: artifact.failure_summary.clone(),
        id: artifact.id.clone(),
        parsed_failures,
        run_id: artifact.run_id.clone(),
        started_at,
        status: status_key(artifact.status).to_string(),
        stderr: artifact.stderr.clone(),
        stdout: artifact.stdout.clone(),
    }
}

fn status_key(status: TestStatus) -> &'static str {
    match status {
        TestStatus::Failed => "failed",
        TestStatus::Passed => "passed",
    }
}
