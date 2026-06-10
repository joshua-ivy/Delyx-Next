//! Tauri bridge for attachment proposals: propose a source for ingestion and
//! snapshot the current proposals for a project/thread. Approval, parsing, and
//! context packs arrive in later PRs; PR2 stops at typed, persisted proposals.

use crate::attachment::{
    accept_proposal, AttachmentParseStatus, AttachmentProposal, AttachmentProposalStatus,
    AttachmentRecord,
};
use crate::attachment_bridge_ops::{create_context_pack, parse_attachment, propose_attachment};
use crate::attachment_diagnostics::{attachment_report, AttachmentReportEntry};
use crate::attachment_evidence::{
    evidence_from_pack, list_evidence_for_thread, save_evidence_batch_to_path,
    AttachmentEvidenceRecord,
};
use crate::attachment_external::external_snapshot_chunks;
use crate::attachment_media::chunk_pdf_pages;
use crate::attachment_persistence::{
    list_proposals_from_path, list_records_from_path, load_proposal_from_path,
    load_record_from_path, save_chunks_to_path, save_record_to_path, set_proposal_status_to_path,
    set_record_parse_status_to_path,
};
use crate::context_pack::{load_context_pack_from_path, ContextPack};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub use crate::attachment_bridge_types::{
    AttachmentApproveRequest, AttachmentExternalSnapshotRequest, AttachmentParsePdfRequest,
    AttachmentParseRequest, AttachmentParseResultView, AttachmentProposalIdRequest,
    AttachmentProposeRequest, AttachmentSnapshotView, ContextPackCreateRequest,
};

pub struct AttachmentBridgeState {
    database_path: PathBuf,
}

impl AttachmentBridgeState {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

#[tauri::command]
pub fn attachment_propose(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentProposeRequest,
) -> Result<AttachmentProposal, String> {
    propose_attachment(&state.database_path, request)
}

#[tauri::command]
pub fn attachment_snapshot(
    state: tauri::State<AttachmentBridgeState>,
    project_id: String,
    thread_id: Option<String>,
) -> Result<AttachmentSnapshotView, String> {
    let proposals =
        list_proposals_from_path(&state.database_path, &project_id, thread_id.as_deref())?;
    let records = list_records_from_path(&state.database_path, &project_id, thread_id.as_deref())?;
    Ok(AttachmentSnapshotView {
        project_id,
        thread_id,
        proposals,
        records,
    })
}

/// Accept a proposal into a durable AttachmentRecord. Risky proposals
/// (`requires_approval`) must carry an `approval_id`; safe ones pass freely. The
/// proposal is then marked approved and linked to the approval.
#[tauri::command]
pub fn attachment_approve(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentApproveRequest,
) -> Result<AttachmentRecord, String> {
    let proposal = load_proposal_from_path(&state.database_path, &request.proposal_id)?
        .ok_or_else(|| {
            format!(
                "Attachment proposal `{}` was not found.",
                request.proposal_id
            )
        })?;
    let record = accept_proposal(&proposal, request.approval_id.as_deref())?;
    let saved = save_record_to_path(&state.database_path, &record)?;
    set_proposal_status_to_path(
        &state.database_path,
        &proposal.id,
        AttachmentProposalStatus::Approved,
        request.approval_id.as_deref(),
    )?;
    Ok(saved)
}

/// Parse an accepted attachment's text into chunks with line-range locators.
/// Non-text kinds are marked unsupported; oversized content yields `partial`.
#[tauri::command]
pub fn attachment_parse(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentParseRequest,
) -> Result<AttachmentParseResultView, String> {
    parse_attachment(&state.database_path, request)
}

/// Build a scoped context pack from the thread's parsed attachment chunks. Only
/// parsed/partial attachments contribute; pinned locators are always included.
#[tauri::command]
pub fn context_pack_create(
    state: tauri::State<AttachmentBridgeState>,
    request: ContextPackCreateRequest,
) -> Result<ContextPack, String> {
    create_context_pack(&state.database_path, request)
}

fn now_millis_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

/// Generate (and persist) evidence records from a context pack so the assistant
/// can cite exact attachment chunk ranges. Returns the evidence it created.
#[tauri::command]
pub fn attachment_evidence_from_pack(
    state: tauri::State<AttachmentBridgeState>,
    pack_id: String,
) -> Result<Vec<AttachmentEvidenceRecord>, String> {
    let pack = load_context_pack_from_path(&state.database_path, &pack_id)?
        .ok_or_else(|| format!("Context pack `{pack_id}` was not found."))?;
    let records = evidence_from_pack(&pack, &now_millis_string());
    save_evidence_batch_to_path(&state.database_path, &records)?;
    Ok(records)
}

/// Redacted, content-free attachment report for the diagnostics panel and
/// support bundles. Never includes chunk text, excerpts, or raw paths.
#[tauri::command]
pub fn attachment_report_snapshot(
    state: tauri::State<AttachmentBridgeState>,
    project_id: String,
    thread_id: Option<String>,
) -> Result<Vec<AttachmentReportEntry>, String> {
    let records = list_records_from_path(&state.database_path, &project_id, thread_id.as_deref())?;
    Ok(attachment_report(&records))
}

#[tauri::command]
pub fn attachment_evidence_snapshot(
    state: tauri::State<AttachmentBridgeState>,
    project_id: String,
    thread_id: String,
) -> Result<Vec<AttachmentEvidenceRecord>, String> {
    list_evidence_for_thread(&state.database_path, &project_id, &thread_id)
}

/// Store per-page chunks for a PDF whose text was extracted elsewhere. No
/// extractable text → `partial` (never a fake "parsed").
#[tauri::command]
pub fn attachment_parse_pdf(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentParsePdfRequest,
) -> Result<AttachmentParseResultView, String> {
    let record = load_record_from_path(&state.database_path, &request.attachment_id)?
        .ok_or_else(|| format!("Attachment `{}` was not found.", request.attachment_id))?;
    let chunks = chunk_pdf_pages(&record.display_name, &request.pages);
    save_chunks_to_path(&state.database_path, &record.id, &chunks)?;
    let partial = chunks.is_empty();
    let status = if partial {
        AttachmentParseStatus::Partial
    } else {
        AttachmentParseStatus::Parsed
    };
    let updated = set_record_parse_status_to_path(&state.database_path, &record.id, status)?;
    Ok(AttachmentParseResultView {
        attachment_id: updated.id,
        parse_status: updated.parse_status.as_str().to_string(),
        chunk_count: chunks.len(),
        partial,
    })
}

/// Store a fetched URL/connector snapshot on an approved external attachment.
/// Empty content = failed fetch (record marked failed, no chunks). The source
/// locator (record's `original_locator`) and retrieval time are preserved.
#[tauri::command]
pub fn attachment_external_snapshot(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentExternalSnapshotRequest,
) -> Result<AttachmentParseResultView, String> {
    let record = load_record_from_path(&state.database_path, &request.attachment_id)?
        .ok_or_else(|| format!("Attachment `{}` was not found.", request.attachment_id))?;
    let retrieved_at = request
        .retrieved_at_ms
        .map(|value| value.to_string())
        .unwrap_or_else(now_millis_string);
    let chunks =
        external_snapshot_chunks(&record.original_locator, &request.content, &retrieved_at);
    save_chunks_to_path(&state.database_path, &record.id, &chunks)?;
    let failed = chunks.is_empty();
    let status = if failed {
        AttachmentParseStatus::Failed
    } else {
        AttachmentParseStatus::Parsed
    };
    let updated = set_record_parse_status_to_path(&state.database_path, &record.id, status)?;
    Ok(AttachmentParseResultView {
        attachment_id: updated.id,
        parse_status: updated.parse_status.as_str().to_string(),
        chunk_count: chunks.len(),
        partial: false,
    })
}

#[tauri::command]
pub fn attachment_deny(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentProposalIdRequest,
) -> Result<AttachmentProposal, String> {
    set_proposal_status_to_path(
        &state.database_path,
        &request.proposal_id,
        AttachmentProposalStatus::Denied,
        None,
    )
}

#[tauri::command]
pub fn attachment_expire(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentProposalIdRequest,
) -> Result<AttachmentProposal, String> {
    set_proposal_status_to_path(
        &state.database_path,
        &request.proposal_id,
        AttachmentProposalStatus::Expired,
        None,
    )
}
