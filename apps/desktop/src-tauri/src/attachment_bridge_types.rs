//! Request/response payloads for the attachment bridge commands.

use crate::attachment::{
    AttachmentKind, AttachmentProposal, AttachmentRecord, AttachmentSourceKind,
};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentParsePdfRequest {
    pub attachment_id: String,
    /// Already-extracted page texts (e.g. from a webview-side PDF extractor),
    /// one string per page. Empty pages are skipped.
    pub pages: Vec<String>,
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
