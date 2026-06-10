//! Attachment domain model: typed proposals (a preview of what Delyx wants to
//! ingest) and durable records. PR2 covers proposals + risk classification; the
//! parser/index/context-pack stages land in later PRs.

use crate::project::ApprovalPolicyRecord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentSourceKind {
    LocalFile,
    LocalFolder,
    ProjectFile,
    Clipboard,
    Url,
    Screenshot,
    Connector,
    McpResource,
}

impl AttachmentSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AttachmentSourceKind::LocalFile => "local_file",
            AttachmentSourceKind::LocalFolder => "local_folder",
            AttachmentSourceKind::ProjectFile => "project_file",
            AttachmentSourceKind::Clipboard => "clipboard",
            AttachmentSourceKind::Url => "url",
            AttachmentSourceKind::Screenshot => "screenshot",
            AttachmentSourceKind::Connector => "connector",
            AttachmentSourceKind::McpResource => "mcp_resource",
        }
    }

    pub fn from_str(value: &str) -> Option<AttachmentSourceKind> {
        Some(match value {
            "local_file" => AttachmentSourceKind::LocalFile,
            "local_folder" => AttachmentSourceKind::LocalFolder,
            "project_file" => AttachmentSourceKind::ProjectFile,
            "clipboard" => AttachmentSourceKind::Clipboard,
            "url" => AttachmentSourceKind::Url,
            "screenshot" => AttachmentSourceKind::Screenshot,
            "connector" => AttachmentSourceKind::Connector,
            "mcp_resource" => AttachmentSourceKind::McpResource,
            _ => return None,
        })
    }

    /// External sources carry data from outside the project trust boundary.
    pub fn is_external(self) -> bool {
        matches!(
            self,
            AttachmentSourceKind::Url
                | AttachmentSourceKind::Connector
                | AttachmentSourceKind::McpResource
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentKind {
    Text,
    Code,
    Markdown,
    Pdf,
    Image,
    Archive,
    Binary,
    Folder,
    Url,
    Unknown,
}

impl AttachmentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AttachmentKind::Text => "text",
            AttachmentKind::Code => "code",
            AttachmentKind::Markdown => "markdown",
            AttachmentKind::Pdf => "pdf",
            AttachmentKind::Image => "image",
            AttachmentKind::Archive => "archive",
            AttachmentKind::Binary => "binary",
            AttachmentKind::Folder => "folder",
            AttachmentKind::Url => "url",
            AttachmentKind::Unknown => "unknown",
        }
    }

    pub fn from_str(value: &str) -> AttachmentKind {
        match value {
            "text" => AttachmentKind::Text,
            "code" => AttachmentKind::Code,
            "markdown" => AttachmentKind::Markdown,
            "pdf" => AttachmentKind::Pdf,
            "image" => AttachmentKind::Image,
            "archive" => AttachmentKind::Archive,
            "binary" => AttachmentKind::Binary,
            "folder" => AttachmentKind::Folder,
            "url" => AttachmentKind::Url,
            _ => AttachmentKind::Unknown,
        }
    }
}

/// Infer a kind from a locator's file extension. Conservative: unknown stays
/// unknown rather than guessing text.
pub fn infer_kind(locator: &str) -> AttachmentKind {
    let lower = locator.to_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or("");
    match ext {
        "md" | "markdown" => AttachmentKind::Markdown,
        "txt" | "log" | "csv" | "json" | "yaml" | "yml" | "toml" => AttachmentKind::Text,
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "c" | "h" | "cpp" | "hpp"
        | "cs" | "rb" | "php" | "swift" | "kt" | "sql" | "sh" => AttachmentKind::Code,
        "pdf" => AttachmentKind::Pdf,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" => AttachmentKind::Image,
        "zip" | "tar" | "gz" | "tgz" | "rar" | "7z" => AttachmentKind::Archive,
        "exe" | "dll" | "bin" | "so" | "dylib" | "o" => AttachmentKind::Binary,
        _ => AttachmentKind::Unknown,
    }
}

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
