use crate::agent_drive::AgentDriveContext;
use crate::agent_scheduler_patch_context::repair_relative_path;
use crate::approval_bridge::{
    propose_approval_record, ApprovalBridgeStore, ApprovalProposalRequest, PermissionScopeView,
};
use crate::patch_bridge::PatchBridgeStore;
use crate::review_bridge::ReviewBridgeStore;
use crate::thread_run_bridge::ThreadRunStore;
use std::path::Path;

/// Renderer-supplied expiry for a driver-created approval card. The driver has no
/// clock library, so the calling command provides both the display ISO string and
/// the millisecond deadline the approval gate enforces.
#[derive(Debug, Clone)]
pub struct ApprovalExpiry {
    pub iso: String,
    pub ms: u64,
}

/// Driver entry point: create the pending apply approval card when the command
/// supplied an expiry, otherwise yield to the renderer (no-op).
pub(crate) fn create_apply_approval(
    context: &mut AgentDriveContext<'_>,
    proposal_id: Option<&str>,
) -> Result<(), String> {
    let (Some(expiry), Some(proposal_id)) = (context.approval_expiry.clone(), proposal_id) else {
        return Ok(());
    };
    ensure_patch_apply_approval(
        context.approvals,
        context.patches,
        &context.run_id,
        proposal_id,
        &expiry,
    )
}

/// Driver entry point: create the pending repair approval card when the command
/// supplied an expiry, otherwise yield to the renderer (no-op).
pub(crate) fn create_repair_approval(
    context: &mut AgentDriveContext<'_>,
    report_id: Option<&str>,
    finding_id: Option<&str>,
) -> Result<(), String> {
    let (Some(expiry), Some(report_id), Some(finding_id)) =
        (context.approval_expiry.clone(), report_id, finding_id)
    else {
        return Ok(());
    };
    ensure_repair_approval(
        context.approvals,
        context.reviews,
        context.threads,
        context.plan_db,
        &context.run_id,
        report_id,
        finding_id,
        &expiry,
    )
}

/// Idempotently create the pending FileWrite approval for a proposed patch apply,
/// matching the scheduler's `{run}-patch-apply-{proposal}` node so granting it
/// makes the scheduler return `run_patch_apply`. Creates a pending card only; it
/// never grants the approval.
pub(crate) fn ensure_patch_apply_approval(
    approvals: &mut ApprovalBridgeStore,
    patches: &PatchBridgeStore,
    run_id: &str,
    proposal_id: &str,
    expiry: &ApprovalExpiry,
) -> Result<(), String> {
    let proposal = patches
        .records
        .iter()
        .find(|record| record.id == proposal_id)
        .ok_or_else(|| "Patch proposal not found for apply approval.".to_string())?;
    let paths = proposal
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let request = file_write_request(
        format!("drive-apply-{proposal_id}"),
        run_id,
        format!("{run_id}-patch-apply-{proposal_id}"),
        "Apply the proposed patch to the approved files.",
        paths,
        expiry,
    );
    propose_approval_record(approvals, request).map(|_| ())
}

/// Idempotently create the pending FileWrite approval for a scoped repair draft,
/// matching the scheduler's `{run}-repair-{report}-{finding}` node and finding
/// path. Creates a pending card only; it never grants the approval or writes.
pub(crate) fn ensure_repair_approval(
    approvals: &mut ApprovalBridgeStore,
    reviews: &ReviewBridgeStore,
    threads: &ThreadRunStore,
    plan_db: &Path,
    run_id: &str,
    report_id: &str,
    finding_id: &str,
    expiry: &ApprovalExpiry,
) -> Result<(), String> {
    let report = reviews
        .reports
        .iter()
        .find(|report| report.id == report_id && report.run_id == run_id)
        .ok_or_else(|| "Review report not found for repair approval.".to_string())?;
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.id == finding_id)
        .ok_or_else(|| "Review finding not found for repair approval.".to_string())?;
    let record = threads
        .records
        .iter()
        .find(|record| record.run_id == run_id)
        .ok_or_else(|| "Thread run record not found for repair approval.".to_string())?;
    let project = crate::workspace_persistence::load_project_by_id(plan_db, &record.project_id)?
        .ok_or_else(|| "Project not found for repair approval.".to_string())?;
    let relative = repair_relative_path(&finding.file_path, &project.path)
        .ok_or_else(|| "Repair finding path is outside the project root.".to_string())?;
    let request = file_write_request(
        format!("drive-repair-{report_id}-{finding_id}"),
        run_id,
        format!("{run_id}-repair-{report_id}-{finding_id}"),
        "Draft a repair patch for the review finding file.",
        vec![relative],
        expiry,
    );
    propose_approval_record(approvals, request).map(|_| ())
}

fn file_write_request(
    client_id: String,
    run_id: &str,
    node_id: String,
    summary: &str,
    paths: Vec<String>,
    expiry: &ApprovalExpiry,
) -> ApprovalProposalRequest {
    ApprovalProposalRequest {
        action_type: "edit_file".to_string(),
        client_id,
        expected_result: summary.to_string(),
        expires_at: expiry.iso.clone(),
        expires_at_ms: expiry.ms,
        node_id,
        rationale: summary.to_string(),
        required_permission: "write_file".to_string(),
        risk_label: "high".to_string(),
        rollback_plan: Some("Use checkpoint receipts to roll back.".to_string()),
        run_id: run_id.to_string(),
        scope: PermissionScopeView {
            commands: None,
            connector_id: None,
            kind: "file".to_string(),
            paths: Some(paths),
            project_id: None,
            root: None,
            summary: summary.to_string(),
        },
    }
}
