use crate::patch::{DiffLine, DiffLineKind, PatchFile, PatchProposal};
use crate::review::{ReviewAgent, ReviewReport};
use crate::review_bridge_keys::{decision_key, mode_key, patch_status, priority_key, test_status};
use crate::test_runner::TestArtifact;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct ReviewBridgeState {
    store: Mutex<ReviewBridgeStore>,
}

#[derive(Default)]
pub struct ReviewBridgeStore {
    agent: ReviewAgent,
    reports: Vec<ReviewReportView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCreateRequest {
    pub run_id: String,
    pub patches: Vec<PatchReviewInput>,
    pub tests: Vec<TestReviewInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchReviewInput {
    pub id: String,
    pub run_id: String,
    pub approval_id: String,
    pub status: String,
    pub files: Vec<PatchFileReviewInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchFileReviewInput {
    pub path: String,
    pub diff: Vec<DiffLineReviewInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLineReviewInput {
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestReviewInput {
    pub id: String,
    pub run_id: String,
    pub approval_id: Option<String>,
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub status: Option<String>,
    pub failure_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewReportView {
    pub id: String,
    pub run_id: String,
    pub mode: String,
    pub decision: String,
    pub findings: Vec<ReviewFindingView>,
    pub risk_summary: String,
    pub test_summary: String,
    pub evidence_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFindingView {
    pub id: String,
    pub priority: String,
    pub title: String,
    pub detail: String,
    pub risk_label: String,
    pub suggested_fix: String,
    pub file_path: String,
    pub hunk_label: String,
}

#[tauri::command]
pub fn review_create(
    state: tauri::State<ReviewBridgeState>,
    request: ReviewCreateRequest,
) -> Result<ReviewReportView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "Review bridge lock failed.".to_string())?;
    create_review_record(&mut store, request)
}

#[tauri::command]
pub fn review_snapshot(
    state: tauri::State<ReviewBridgeState>,
    run_id: String,
) -> Result<Vec<ReviewReportView>, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "Review bridge lock failed.".to_string())?;
    Ok(review_snapshot_from_store(&store, &run_id))
}

pub fn create_review_record(
    store: &mut ReviewBridgeStore,
    request: ReviewCreateRequest,
) -> Result<ReviewReportView, String> {
    if request.run_id.trim().is_empty() {
        return Err("Review requires a run ID.".to_string());
    }
    if request
        .patches
        .iter()
        .any(|patch| patch.run_id != request.run_id)
    {
        return Err("Review patch artifacts must belong to the requested run.".to_string());
    }
    if request
        .tests
        .iter()
        .any(|test| test.run_id != request.run_id)
    {
        return Err("Review test artifacts must belong to the requested run.".to_string());
    }
    let patches = request
        .patches
        .into_iter()
        .map(patch_input)
        .collect::<Result<Vec<_>, _>>()?;
    let tests = request
        .tests
        .into_iter()
        .map(test_input)
        .collect::<Result<Vec<_>, _>>()?;
    let report = store.agent.review(&request.run_id, &patches, &tests);
    let view = report_view(&report);
    store.reports.push(view.clone());
    Ok(view)
}

pub fn review_snapshot_from_store(
    store: &ReviewBridgeStore,
    run_id: &str,
) -> Vec<ReviewReportView> {
    store
        .reports
        .iter()
        .filter(|report| report.run_id == run_id)
        .cloned()
        .collect()
}

fn patch_input(input: PatchReviewInput) -> Result<PatchProposal, String> {
    Ok(PatchProposal {
        approval_id: input.approval_id,
        checkpoint_id: None,
        files: input
            .files
            .into_iter()
            .map(patch_file_input)
            .collect::<Result<Vec<_>, _>>()?,
        id: input.id,
        run_id: input.run_id,
        status: patch_status(&input.status)?,
    })
}

fn patch_file_input(input: PatchFileReviewInput) -> Result<PatchFile, String> {
    let after = input
        .diff
        .iter()
        .filter(|line| line.kind == "added")
        .map(|line| line.text.clone())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(PatchFile {
        after,
        before: String::new(),
        diff: input
            .diff
            .into_iter()
            .map(diff_input)
            .collect::<Result<Vec<_>, _>>()?,
        path: PathBuf::from(input.path),
    })
}

fn diff_input(input: DiffLineReviewInput) -> Result<DiffLine, String> {
    Ok(DiffLine {
        kind: match input.kind.as_str() {
            "added" => DiffLineKind::Added,
            "context" => DiffLineKind::Context,
            "removed" => DiffLineKind::Removed,
            _ => return Err("Unsupported diff line kind.".to_string()),
        },
        text: input.text,
    })
}

fn test_input(input: TestReviewInput) -> Result<TestArtifact, String> {
    Ok(TestArtifact {
        approval_id: input.approval_id.unwrap_or_default(),
        command: input.command,
        created_at: 0,
        duration_ms: input.duration_ms,
        exit_code: input.exit_code,
        failure_summary: input.failure_summary,
        id: input.id,
        exec_events: Vec::new(),
        output_truncated: false,
        run_id: input.run_id,
        status: test_status(input.status.as_deref(), input.exit_code)?,
        stderr: input.stderr,
        stdout: input.stdout,
        working_directory: PathBuf::from(input.cwd),
    })
}

fn report_view(report: &ReviewReport) -> ReviewReportView {
    ReviewReportView {
        decision: decision_key(report.decision).to_string(),
        evidence_summary: report.evidence_summary.clone(),
        findings: report.findings.iter().map(finding_view).collect(),
        id: report.id.clone(),
        mode: mode_key(report.mode).to_string(),
        risk_summary: report.risk_summary.clone(),
        run_id: report.run_id.clone(),
        test_summary: report.test_summary.clone(),
    }
}

fn finding_view(finding: &crate::review::ReviewFinding) -> ReviewFindingView {
    ReviewFindingView {
        detail: finding.detail.clone(),
        file_path: finding.hunk.file_path.clone(),
        hunk_label: format!("{}:{}", finding.hunk.patch_id, finding.hunk.line_index),
        id: finding.id.clone(),
        priority: priority_key(finding.priority).to_string(),
        risk_label: finding.risk_label.clone(),
        suggested_fix: finding.suggested_fix.clone(),
        title: finding.title.clone(),
    }
}
