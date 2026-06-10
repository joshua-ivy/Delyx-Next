//! Attachment domain model: typed proposals (a preview of what Delyx wants to
//! ingest) and durable records. PR2 covers proposals + risk classification; the
//! parser/index/context-pack stages land in later PRs.

use serde::{Deserialize, Serialize};

pub use crate::attachment_kind::{infer_kind, AttachmentKind, AttachmentSourceKind};
pub use crate::attachment_policy::{
    accept_proposal, classify, stable_proposal_id, Classification, ClassifyInput,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentRisk {
    Low,
    Medium,
    High,
}

impl AttachmentRisk {
    pub fn as_str(self) -> &'static str {
        match self {
            AttachmentRisk::Low => "low",
            AttachmentRisk::Medium => "medium",
            AttachmentRisk::High => "high",
        }
    }

    pub fn from_str(value: &str) -> AttachmentRisk {
        match value {
            "high" => AttachmentRisk::High,
            "medium" => AttachmentRisk::Medium,
            _ => AttachmentRisk::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentProposalStatus {
    Pending,
    NeedsApproval,
    Approved,
    Denied,
    Expired,
    Failed,
}

impl AttachmentProposalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AttachmentProposalStatus::Pending => "pending",
            AttachmentProposalStatus::NeedsApproval => "needs_approval",
            AttachmentProposalStatus::Approved => "approved",
            AttachmentProposalStatus::Denied => "denied",
            AttachmentProposalStatus::Expired => "expired",
            AttachmentProposalStatus::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> AttachmentProposalStatus {
        match value {
            "needs_approval" => AttachmentProposalStatus::NeedsApproval,
            "approved" => AttachmentProposalStatus::Approved,
            "denied" => AttachmentProposalStatus::Denied,
            "expired" => AttachmentProposalStatus::Expired,
            "failed" => AttachmentProposalStatus::Failed,
            _ => AttachmentProposalStatus::Pending,
        }
    }
}

/// Whether an attachment is scoped to a single thread or the whole project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentScope {
    /// "thread" (default) or "project".
    pub mode: String,
}

impl Default for AttachmentScope {
    fn default() -> Self {
        Self {
            mode: "thread".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentProposal {
    pub id: String,
    pub project_id: String,
    pub thread_id: Option<String>,
    pub source_kind: AttachmentSourceKind,
    pub detected_kind: AttachmentKind,
    pub display_name: String,
    pub source_locator: String,
    pub proposed_scope: AttachmentScope,
    pub estimated_bytes: Option<u64>,
    pub estimated_file_count: Option<u32>,
    pub requires_approval: bool,
    pub approval_reason: Option<String>,
    pub risk: AttachmentRisk,
    pub status: AttachmentProposalStatus,
    pub approval_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentParseStatus {
    NotStarted,
    Reading,
    Parsed,
    Partial,
    Unsupported,
    Failed,
}

impl AttachmentParseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AttachmentParseStatus::NotStarted => "not_started",
            AttachmentParseStatus::Reading => "reading",
            AttachmentParseStatus::Parsed => "parsed",
            AttachmentParseStatus::Partial => "partial",
            AttachmentParseStatus::Unsupported => "unsupported",
            AttachmentParseStatus::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> AttachmentParseStatus {
        match value {
            "reading" => AttachmentParseStatus::Reading,
            "parsed" => AttachmentParseStatus::Parsed,
            "partial" => AttachmentParseStatus::Partial,
            "unsupported" => AttachmentParseStatus::Unsupported,
            "failed" => AttachmentParseStatus::Failed,
            _ => AttachmentParseStatus::NotStarted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentIndexStatus {
    NotIndexed,
    Queued,
    Indexed,
    Partial,
    Failed,
}

impl AttachmentIndexStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AttachmentIndexStatus::NotIndexed => "not_indexed",
            AttachmentIndexStatus::Queued => "queued",
            AttachmentIndexStatus::Indexed => "indexed",
            AttachmentIndexStatus::Partial => "partial",
            AttachmentIndexStatus::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> AttachmentIndexStatus {
        match value {
            "queued" => AttachmentIndexStatus::Queued,
            "indexed" => AttachmentIndexStatus::Indexed,
            "partial" => AttachmentIndexStatus::Partial,
            "failed" => AttachmentIndexStatus::Failed,
            _ => AttachmentIndexStatus::NotIndexed,
        }
    }
}

/// A durable, accepted attachment. Created only after a proposal clears its
/// approval gate. Parsing/indexing happen in later PRs, so a fresh record starts
/// not-started / not-indexed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentRecord {
    pub id: String,
    pub project_id: String,
    pub thread_id: Option<String>,
    pub message_id: Option<String>,
    pub run_id: Option<String>,
    pub source_kind: AttachmentSourceKind,
    pub detected_kind: AttachmentKind,
    pub display_name: String,
    pub original_locator: String,
    pub local_reference_path: Option<String>,
    pub content_hash: Option<String>,
    pub bytes: Option<u64>,
    pub parse_status: AttachmentParseStatus,
    pub index_status: AttachmentIndexStatus,
    pub approval_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl AttachmentRecord {
    pub fn from_proposal(
        proposal: &AttachmentProposal,
        approval_id: Option<&str>,
    ) -> AttachmentRecord {
        AttachmentRecord {
            id: proposal.id.clone(),
            project_id: proposal.project_id.clone(),
            thread_id: proposal.thread_id.clone(),
            message_id: None,
            run_id: None,
            source_kind: proposal.source_kind,
            detected_kind: proposal.detected_kind,
            display_name: proposal.display_name.clone(),
            original_locator: proposal.source_locator.clone(),
            local_reference_path: None,
            content_hash: None,
            bytes: proposal.estimated_bytes,
            parse_status: AttachmentParseStatus::NotStarted,
            index_status: AttachmentIndexStatus::NotIndexed,
            approval_id: approval_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string()),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}
