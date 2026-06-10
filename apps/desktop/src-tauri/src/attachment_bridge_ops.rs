//! Database-path-level operations behind the attachment bridge commands.

use crate::attachment::{
    classify, infer_kind, stable_proposal_id, AttachmentParseStatus, AttachmentProposal,
    AttachmentProposalStatus, AttachmentScope, AttachmentSourceKind, ClassifyInput,
};
use crate::attachment_bridge_types::{
    AttachmentParseRequest, AttachmentParseResultView, AttachmentProposeRequest,
    ContextPackCreateRequest,
};
use crate::attachment_parser::{is_text_like, parse_attachment_text};
use crate::attachment_persistence::{
    list_chunks_from_path, list_records_from_path, load_record_from_path, save_chunks_to_path,
    save_proposal_to_path, set_record_parse_status_to_path,
};
use crate::context_pack::{save_context_pack_to_path, select_context, ChunkCandidate, ContextPack};
use crate::project::ApprovalPolicyRecord;
use std::collections::HashSet;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn propose_attachment(
    database_path: &Path,
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
    let project =
        crate::project_persistence::load_project_from_path(database_path, &request.project_id)
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
    save_proposal_to_path(database_path, &proposal)
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

pub fn parse_attachment(
    database_path: &Path,
    request: AttachmentParseRequest,
) -> Result<AttachmentParseResultView, String> {
    let record = load_record_from_path(database_path, &request.attachment_id)?
        .ok_or_else(|| format!("Attachment `{}` was not found.", request.attachment_id))?;

    if !is_text_like(record.detected_kind) {
        let updated = set_record_parse_status_to_path(
            database_path,
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
                    database_path,
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
    save_chunks_to_path(database_path, &record.id, &output.chunks)?;
    let status = if output.partial {
        AttachmentParseStatus::Partial
    } else {
        AttachmentParseStatus::Parsed
    };
    let updated = set_record_parse_status_to_path(database_path, &record.id, status)?;
    Ok(AttachmentParseResultView {
        attachment_id: updated.id,
        parse_status: updated.parse_status.as_str().to_string(),
        chunk_count: output.chunks.len(),
        partial: output.partial,
    })
}

const DEFAULT_CONTEXT_BUDGET_TOKENS: u32 = 4_000;

pub fn create_context_pack(
    database_path: &Path,
    request: ContextPackCreateRequest,
) -> Result<ContextPack, String> {
    let records =
        list_records_from_path(database_path, &request.project_id, Some(&request.thread_id))?;
    let mut candidates = Vec::new();
    for record in records {
        if !matches!(
            record.parse_status,
            AttachmentParseStatus::Parsed | AttachmentParseStatus::Partial
        ) {
            continue;
        }
        for chunk in list_chunks_from_path(database_path, &record.id)? {
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
    save_context_pack_to_path(database_path, &pack)
}

fn unique_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0)
}
