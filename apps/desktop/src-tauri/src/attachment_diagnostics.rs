//! Metadata-only attachment diagnostics for support bundles and the diagnostics
//! panel. Crucially it carries NO file contents, chunk text, excerpts, or raw
//! paths — only status metadata and a human failure hint — so a support bundle
//! can't leak attached file contents or secrets by default.

use crate::attachment::AttachmentRecord;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentReportEntry {
    pub id: String,
    /// Redacted display name (secret-looking names are masked).
    pub display_name: String,
    pub source_kind: String,
    pub detected_kind: String,
    pub parse_status: String,
    pub index_status: String,
    pub bytes: Option<u64>,
    pub has_approval: bool,
    /// Why an attachment isn't usable, in plain language. None when healthy.
    pub failure_hint: Option<String>,
}

/// Build a redacted, content-free report from attachment records.
pub fn attachment_report(records: &[AttachmentRecord]) -> Vec<AttachmentReportEntry> {
    records.iter().map(entry_for).collect()
}

fn entry_for(record: &AttachmentRecord) -> AttachmentReportEntry {
    AttachmentReportEntry {
        id: record.id.clone(),
        display_name: redact(&record.display_name),
        source_kind: record.source_kind.as_str().to_string(),
        detected_kind: record.detected_kind.as_str().to_string(),
        parse_status: record.parse_status.as_str().to_string(),
        index_status: record.index_status.as_str().to_string(),
        bytes: record.bytes,
        has_approval: record.approval_id.is_some(),
        failure_hint: failure_hint(record),
    }
}

fn failure_hint(record: &AttachmentRecord) -> Option<String> {
    use crate::attachment::{AttachmentIndexStatus, AttachmentParseStatus};
    match record.parse_status {
        AttachmentParseStatus::Failed => Some(
            "Could not read the file — it may be missing, binary, or not valid UTF-8 text."
                .to_string(),
        ),
        AttachmentParseStatus::Unsupported => Some(
            "This type is not parsed into text (e.g. image/binary); it's stored but not interpreted."
                .to_string(),
        ),
        AttachmentParseStatus::Partial => Some(
            "Only part of the content was processed (truncated at the size cap or partial extraction)."
                .to_string(),
        ),
        _ => match record.index_status {
            AttachmentIndexStatus::Failed => Some("Indexing failed for this attachment.".to_string()),
            _ => None,
        },
    }
}

/// Mask values that look like secrets or contain a path so a bundle stays clean.
pub fn redact(value: &str) -> String {
    if looks_secret(value) {
        return "[redacted]".to_string();
    }
    value.to_string()
}

fn looks_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    // Distinctive keyword substrings — safe to over-match in a support bundle.
    if [
        "api_key",
        "apikey",
        "password",
        "secret",
        "credential",
        "private_key",
        "authorization",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return true;
    }
    if value.trim_start().starts_with("-----BEGIN ") {
        return true;
    }
    // Token-prefix patterns checked per whitespace token, so a real secret like
    // "sk-LIVE..." is caught without redacting innocent names like "task-1.txt".
    value.split_whitespace().any(|token| {
        let lower = token.to_ascii_lowercase();
        lower.starts_with("sk-")
            || lower.starts_with("bearer")
            || lower.starts_with("xoxb-")
            || token.starts_with("ghp_")
            || token.starts_with("github_pat_")
            || token.starts_with("AKIA")
    })
}
