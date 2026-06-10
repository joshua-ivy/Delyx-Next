//! Tauri bridge for attachment proposals: propose a source for ingestion and
//! snapshot the current proposals for a project/thread. Approval, parsing, and
//! context packs arrive in later PRs; PR2 stops at typed, persisted proposals.

use crate::attachment::AttachmentParseStatus;
use crate::attachment::{
    accept_proposal, classify, infer_kind, stable_proposal_id, AttachmentKind, AttachmentProposal,
    AttachmentProposalStatus, AttachmentRecord, AttachmentScope, AttachmentSourceKind,
    ClassifyInput,
};
use crate::attachment_diagnostics::{attachment_report, AttachmentReportEntry};
use crate::attachment_evidence::{
    evidence_from_pack, list_evidence_for_thread, save_evidence_batch_to_path,
    AttachmentEvidenceRecord,
};
use crate::attachment_external::external_snapshot_chunks;
use crate::attachment_media::chunk_pdf_pages;
use crate::attachment_parser::{is_text_like, parse_attachment_text};
use crate::attachment_persistence::{
    list_chunks_from_path, list_proposals_from_path, list_records_from_path,
    load_proposal_from_path, load_record_from_path, save_chunks_to_path, save_proposal_to_path,
    save_record_to_path, set_proposal_status_to_path, set_record_parse_status_to_path,
};
use crate::context_pack::{
    load_context_pack_from_path, save_context_pack_to_path, select_context, ChunkCandidate,
    ContextPack,
};
use crate::project::ApprovalPolicyRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AttachmentBridgeState {
    database_path: PathBuf,
}

impl AttachmentBridgeState {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentProposeRequest {
    pub project_id: String,
    #[serde(default)]
    pub thread_id: Option<String>,
    pub source_kind: AttachmentSourceKind,
    pub display_name: String,
    pub source_locator: String,
    #[serde(default)]
    pub scope_mode: Option<String>,
    #[serde(default)]
    pub detected_kind: Option<AttachmentKind>,
    #[serde(default)]
    pub estimated_bytes: Option<u64>,
    #[serde(default)]
    pub estimated_file_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentSnapshotView {
    pub project_id: String,
    pub thread_id: Option<String>,
    pub proposals: Vec<AttachmentProposal>,
    pub records: Vec<AttachmentRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentApproveRequest {
    pub proposal_id: String,
    /// The id of the approval that cleared this attachment (links record→approval).
    #[serde(default)]
    pub approval_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentProposalIdRequest {
    pub proposal_id: String,
}

#[tauri::command]
pub fn attachment_propose(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentProposeRequest,
) -> Result<AttachmentProposal, String> {
    if request.project_id.trim().is_empty() {
        return Err("Attachment proposal requires a project.".to_string());
    }
    if request.source_locator.trim().is_empty() {
        return Err("Attachment proposal requires a source locator.".to_string());
    }

    let detected_kind = request
        .detected_kind
        .unwrap_or_else(|| infer_kind(&request.source_locator));

    // Pull the project's approval policy + read scopes if it exists; otherwise
    // fall back to safe defaults so a proposal can still be classified.
    // `project_id` is already the stable id, so look it up directly.
    let project = crate::project_persistence::load_project_from_path(
        &state.database_path,
        &request.project_id,
    )
    .ok()
    .flatten();
    let policy = project
        .as_ref()
        .map(|p| p.approval_policy.clone())
        .unwrap_or_default();
    let in_read_scope = in_read_scope(&request, project.as_ref(), &policy);

    let classification = classify(ClassifyInput {
        source_kind: request.source_kind,
        detected_kind,
        estimated_bytes: request.estimated_bytes,
        estimated_file_count: request.estimated_file_count,
        in_read_scope,
        policy: &policy,
    });

    let status = if classification.requires_approval {
        AttachmentProposalStatus::NeedsApproval
    } else {
        AttachmentProposalStatus::Pending
    };

    let proposal = AttachmentProposal {
        id: stable_proposal_id(
            &request.project_id,
            request.thread_id.as_deref(),
            &request.source_locator,
        ),
        project_id: request.project_id,
        thread_id: request.thread_id,
        source_kind: request.source_kind,
        detected_kind,
        display_name: request.display_name,
        source_locator: request.source_locator,
        proposed_scope: AttachmentScope {
            mode: request.scope_mode.unwrap_or_else(|| "thread".to_string()),
        },
        estimated_bytes: request.estimated_bytes,
        estimated_file_count: request.estimated_file_count,
        requires_approval: classification.requires_approval,
        approval_reason: classification.reason,
        risk: classification.risk,
        status,
        approval_id: None,
        created_at: String::new(),
        updated_at: String::new(),
    };
    save_proposal_to_path(&state.database_path, &proposal)
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentParseRequest {
    pub attachment_id: String,
    /// Optional inline content (e.g. read by the frontend via FileReader). When
    /// absent, the record's `original_locator` is read as a local file path.
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentParseResultView {
    pub attachment_id: String,
    pub parse_status: String,
    pub chunk_count: usize,
    pub partial: bool,
}

/// Parse an accepted attachment's text into chunks with line-range locators.
/// Non-text kinds are marked unsupported; oversized content yields `partial`.
#[tauri::command]
pub fn attachment_parse(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentParseRequest,
) -> Result<AttachmentParseResultView, String> {
    let record = load_record_from_path(&state.database_path, &request.attachment_id)?
        .ok_or_else(|| format!("Attachment `{}` was not found.", request.attachment_id))?;

    if !is_text_like(record.detected_kind) {
        let updated = set_record_parse_status_to_path(
            &state.database_path,
            &record.id,
            AttachmentParseStatus::Unsupported,
        )?;
        return Ok(AttachmentParseResultView {
            attachment_id: updated.id,
            parse_status: updated.parse_status.as_str().to_string(),
            chunk_count: 0,
            partial: false,
        });
    }

    let content = match request.content {
        Some(content) => content,
        None => match std::fs::read_to_string(&record.original_locator) {
            Ok(content) => content,
            Err(_) => {
                let updated = set_record_parse_status_to_path(
                    &state.database_path,
                    &record.id,
                    AttachmentParseStatus::Failed,
                )?;
                return Ok(AttachmentParseResultView {
                    attachment_id: updated.id,
                    parse_status: updated.parse_status.as_str().to_string(),
                    chunk_count: 0,
                    partial: false,
                });
            }
        },
    };

    let output = parse_attachment_text(&record.display_name, record.detected_kind, &content);
    save_chunks_to_path(&state.database_path, &record.id, &output.chunks)?;
    let status = if output.partial {
        AttachmentParseStatus::Partial
    } else {
        AttachmentParseStatus::Parsed
    };
    let updated = set_record_parse_status_to_path(&state.database_path, &record.id, status)?;
    Ok(AttachmentParseResultView {
        attachment_id: updated.id,
        parse_status: updated.parse_status.as_str().to_string(),
        chunk_count: output.chunks.len(),
        partial: output.partial,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPackCreateRequest {
    pub project_id: String,
    pub thread_id: String,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub budget_tokens: Option<u32>,
    #[serde(default)]
    pub pinned_locators: Option<Vec<String>>,
}

const DEFAULT_CONTEXT_BUDGET_TOKENS: u32 = 4_000;

/// Build a scoped context pack from the thread's parsed attachment chunks. Only
/// parsed/partial attachments contribute; pinned locators are always included.
#[tauri::command]
pub fn context_pack_create(
    state: tauri::State<AttachmentBridgeState>,
    request: ContextPackCreateRequest,
) -> Result<ContextPack, String> {
    let records = list_records_from_path(
        &state.database_path,
        &request.project_id,
        Some(&request.thread_id),
    )?;
    let mut candidates = Vec::new();
    for record in records {
        if !matches!(
            record.parse_status,
            AttachmentParseStatus::Parsed | AttachmentParseStatus::Partial
        ) {
            continue;
        }
        for chunk in list_chunks_from_path(&state.database_path, &record.id)? {
            candidates.push(ChunkCandidate {
                attachment_id: record.id.clone(),
                locator: chunk.locator,
                text: chunk.text,
                token_estimate: chunk.token_estimate,
            });
        }
    }

    let pinned: HashSet<String> = request
        .pinned_locators
        .unwrap_or_default()
        .into_iter()
        .collect();
    let budget = request
        .budget_tokens
        .unwrap_or(DEFAULT_CONTEXT_BUDGET_TOKENS);
    let selection = select_context(candidates, budget, &pinned);

    let pack = ContextPack {
        id: format!("pack-{:x}", unique_stamp()),
        project_id: request.project_id,
        thread_id: request.thread_id,
        run_id: request.run_id,
        strategy: selection.strategy,
        budget_tokens: budget,
        used_tokens: selection.used_tokens,
        status: selection.status,
        items: selection.items,
        created_at: String::new(),
        excluded_count: selection.excluded_count,
    };
    save_context_pack_to_path(&state.database_path, &pack)
}

fn unique_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0)
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentParsePdfRequest {
    pub attachment_id: String,
    /// Already-extracted page texts (e.g. from a webview-side PDF extractor),
    /// one string per page. Empty pages are skipped.
    pub pages: Vec<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentExternalSnapshotRequest {
    pub attachment_id: String,
    /// Text fetched by the webview for this URL/connector resource.
    pub content: String,
    #[serde(default)]
    pub retrieved_at_ms: Option<u64>,
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

fn in_read_scope(
    request: &AttachmentProposeRequest,
    project: Option<&crate::project::ProjectRecord>,
    _policy: &ApprovalPolicyRecord,
) -> Option<bool> {
    // Scope only applies to local file paths.
    let is_local = matches!(
        request.source_kind,
        AttachmentSourceKind::LocalFile
            | AttachmentSourceKind::LocalFolder
            | AttachmentSourceKind::ProjectFile
    );
    if !is_local {
        return None;
    }
    project.map(|p| p.can_read_path(&request.source_locator))
}
