//! Attachment risk classification and the approval gate that turns proposals
//! into durable records.

use crate::attachment::{
    AttachmentProposal, AttachmentProposalStatus, AttachmentRecord, AttachmentRisk,
};
use crate::attachment_kind::{AttachmentKind, AttachmentSourceKind};
use crate::project::ApprovalPolicyRecord;

/// The approval gate: decide whether a proposal may become a durable record.
/// Risky proposals (requires_approval) need a non-empty `approval_id`; safe ones
/// pass with no friction. Denied/expired proposals can never be accepted.
pub fn accept_proposal(
    proposal: &AttachmentProposal,
    approval_id: Option<&str>,
) -> Result<AttachmentRecord, String> {
    match proposal.status {
        AttachmentProposalStatus::Denied => {
            return Err("This attachment was denied and cannot be added.".to_string());
        }
        AttachmentProposalStatus::Expired => {
            return Err("This attachment's approval expired; re-propose it.".to_string());
        }
        _ => {}
    }
    let has_approval = approval_id
        .map(str::trim)
        .map(|value| !value.is_empty())
        .unwrap_or(false);
    if proposal.requires_approval && !has_approval {
        return Err("This attachment needs approval before it can be added.".to_string());
    }
    Ok(AttachmentRecord::from_proposal(proposal, approval_id))
}

/// Inputs to risk classification, independent of storage so it is unit-testable.
pub struct ClassifyInput<'a> {
    pub source_kind: AttachmentSourceKind,
    pub detected_kind: AttachmentKind,
    pub estimated_bytes: Option<u64>,
    pub estimated_file_count: Option<u32>,
    /// `Some(false)` means the locator is a local path outside any allowed read
    /// scope; `None` means scope is not applicable (e.g. clipboard) or unknown.
    pub in_read_scope: Option<bool>,
    pub policy: &'a ApprovalPolicyRecord,
}

pub struct Classification {
    pub requires_approval: bool,
    pub risk: AttachmentRisk,
    pub reason: Option<String>,
}

/// Decide whether ingesting a source needs an approval, and how risky it is.
/// This is the PR2 baseline; PR4 turns `requires_approval` into a real
/// ActionProposal. Keep it conservative — default to requiring approval when in
/// doubt about external or oversized sources.
pub fn classify(input: ClassifyInput) -> Classification {
    // Folders and archives always need a look before Delyx reads them.
    if matches!(input.source_kind, AttachmentSourceKind::LocalFolder)
        || matches!(input.detected_kind, AttachmentKind::Folder)
    {
        return require(
            AttachmentRisk::High,
            "Folder import: review which files Delyx will read.",
        );
    }
    if matches!(input.detected_kind, AttachmentKind::Archive) {
        return require(
            AttachmentRisk::High,
            "Archives must be approved before extraction.",
        );
    }
    if input.source_kind.is_external() {
        return require(
            AttachmentRisk::Medium,
            "External source: data comes from outside the project.",
        );
    }
    if let Some(count) = input.estimated_file_count {
        if count > input.policy.folder_file_count {
            return require(
                AttachmentRisk::Medium,
                "Many files: this import exceeds the project's file-count threshold.",
            );
        }
    }
    if let Some(bytes) = input.estimated_bytes {
        if bytes > input.policy.large_file_bytes {
            return require(
                AttachmentRisk::Medium,
                "Large file: exceeds the project's large-file threshold.",
            );
        }
    }
    if input.in_read_scope == Some(false) && input.policy.require_approval_outside_scope {
        return require(
            AttachmentRisk::Medium,
            "Outside read scope: this path is not in an allowed project read scope.",
        );
    }
    Classification {
        requires_approval: false,
        risk: AttachmentRisk::Low,
        reason: None,
    }
}

fn require(risk: AttachmentRisk, reason: &str) -> Classification {
    Classification {
        requires_approval: true,
        risk,
        reason: Some(reason.to_string()),
    }
}

/// Deterministic proposal id from (project, thread, locator) so re-proposing the
/// same source in the same thread updates one row instead of piling up dupes.
pub fn stable_proposal_id(
    project_id: &str,
    thread_id: Option<&str>,
    source_locator: &str,
) -> String {
    let mut hash: u64 = 1469598103934665603; // FNV-1a offset basis
    for part in [project_id, thread_id.unwrap_or(""), source_locator] {
        for byte in part.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(1099511628211);
        }
        hash ^= 0x1f; // separator so ("ab","c") != ("a","bc")
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("attach-{hash:016x}")
}
