use crate::agent_run::{AgentRunError, AgentRunStatus};
use crate::review_bridge::{ReviewBridgeState, ReviewBridgeStore};
use crate::thread_run_bridge::{ThreadRunBridgeState, ThreadRunStore};
use crate::threads::ThreadStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewRevisionRequest {
    pub run_id: String,
    pub review_report_id: String,
    pub finding_id: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewRevisionBridgeView {
    pub status: String,
    pub run_id: String,
    pub review_report_id: String,
    pub finding_id: String,
    pub next_flow: Vec<String>,
    pub message: String,
}

#[tauri::command]
pub fn agent_request_review_revision(
    threads: tauri::State<ThreadRunBridgeState>,
    reviews: tauri::State<ReviewBridgeState>,
    request: AgentReviewRevisionRequest,
) -> Result<AgentReviewRevisionBridgeView, String> {
    let mut thread_store = threads
        .store
        .lock()
        .map_err(|_| "Thread bridge lock failed.".to_string())?;
    let mut review_store = reviews
        .store
        .lock()
        .map_err(|_| "Review bridge lock failed.".to_string())?;
    let view = request_review_revision_record(&mut thread_store, &mut review_store, request)?;
    threads.persist(&thread_store)?;
    reviews.save_if_persistent(&review_store)?;
    Ok(view)
}

pub fn request_review_revision_record(
    thread_store: &mut ThreadRunStore,
    review_store: &mut ReviewBridgeStore,
    request: AgentReviewRevisionRequest,
) -> Result<AgentReviewRevisionBridgeView, String> {
    validate_request(&request)?;
    let report_index = report_index(review_store, &request)?;
    ensure_finding(review_store, report_index, &request.finding_id)?;
    let thread_id = thread_id_for_run(thread_store, &request.run_id)?;
    ensure_run_is_repairable(thread_store, &request.run_id)?;
    thread_store
        .manager
        .set_status(&thread_id, ThreadStatus::Building)
        .map_err(|error| format!("{error:?}"))?;
    update_record_timestamp(thread_store, &request.run_id, &request.updated_at)?;

    let node = thread_store
        .ledger
        .append_node(
            &request.run_id,
            "repair",
            "Repair requested from review finding",
        )
        .map_err(agent_error)?;
    review_store.reports[report_index].decision = "revise_requested".to_string();
    mark_node(
        &mut thread_store.ledger,
        &request.run_id,
        &node.id,
        AgentRunStatus::Completed,
    )?;
    thread_store
        .ledger
        .record_artifact(
            &request.run_id,
            "review_revision",
            &format!("{}/{}", request.review_report_id, request.finding_id),
        )
        .map_err(agent_error)?;
    thread_store
        .ledger
        .append_event(
            &request.run_id,
            "repair.requested",
            &format!(
                "Repair requested from review {} finding {}.",
                request.review_report_id, request.finding_id
            ),
        )
        .map_err(agent_error)?;

    Ok(AgentReviewRevisionBridgeView {
        finding_id: request.finding_id,
        message: "Repair requested from review finding; next flow is plan -> build.".to_string(),
        next_flow: vec!["plan".to_string(), "build".to_string()],
        review_report_id: request.review_report_id,
        run_id: request.run_id,
        status: "revise_requested".to_string(),
    })
}

fn validate_request(request: &AgentReviewRevisionRequest) -> Result<(), String> {
    if request.run_id.trim().is_empty() {
        return Err("Review revision requires a run ID.".to_string());
    }
    if request.review_report_id.trim().is_empty() {
        return Err("Review revision requires a review report ID.".to_string());
    }
    if request.finding_id.trim().is_empty() {
        return Err("Review revision requires a finding ID.".to_string());
    }
    Ok(())
}

fn report_index(
    store: &ReviewBridgeStore,
    request: &AgentReviewRevisionRequest,
) -> Result<usize, String> {
    store
        .reports
        .iter()
        .position(|report| report.id == request.review_report_id && report.run_id == request.run_id)
        .ok_or_else(|| "Review report was not found for this run.".to_string())
}

fn ensure_finding(
    store: &ReviewBridgeStore,
    report_index: usize,
    finding_id: &str,
) -> Result<(), String> {
    store.reports[report_index]
        .findings
        .iter()
        .any(|finding| finding.id == finding_id)
        .then_some(())
        .ok_or_else(|| "Review finding was not found on this report.".to_string())
}

fn thread_id_for_run(store: &ThreadRunStore, run_id: &str) -> Result<String, String> {
    store
        .records
        .iter()
        .find(|record| record.run_id == run_id)
        .map(|record| record.thread_id.clone())
        .ok_or_else(|| "Thread record was not found for this run.".to_string())
}

fn ensure_run_is_repairable(store: &ThreadRunStore, run_id: &str) -> Result<(), String> {
    let run = store.ledger.get_run(run_id).map_err(agent_error)?;
    (run.status == AgentRunStatus::Running)
        .then_some(())
        .ok_or_else(|| "Only a running AgentRun can request repair.".to_string())
}

fn update_record_timestamp(
    store: &mut ThreadRunStore,
    run_id: &str,
    updated_at: &str,
) -> Result<(), String> {
    let record = store
        .records
        .iter_mut()
        .find(|record| record.run_id == run_id)
        .ok_or_else(|| "Thread record was not found for this run.".to_string())?;
    record.updated_at = updated_at.to_string();
    Ok(())
}

fn mark_node(
    ledger: &mut crate::agent_run::AgentRunLedger,
    run_id: &str,
    node_id: &str,
    status: AgentRunStatus,
) -> Result<(), String> {
    let run = ledger.run_mut(run_id).map_err(agent_error)?;
    let node = run
        .nodes
        .iter_mut()
        .find(|node| node.id == node_id)
        .ok_or_else(|| "Agent repair node not found.".to_string())?;
    node.status = status;
    Ok(())
}

fn agent_error(error: AgentRunError) -> String {
    format!("{error:?}")
}
