// Attachment domain, mirroring the Rust `AttachmentProposal` (serde camelCase).
// PR3 only deals with proposals (preview of what Delyx wants to ingest); parsing
// and records arrive in later PRs.

export type AttachmentSourceKind =
  | "local_file"
  | "local_folder"
  | "project_file"
  | "clipboard"
  | "url"
  | "screenshot"
  | "connector"
  | "mcp_resource";

export type AttachmentKind =
  | "text"
  | "code"
  | "markdown"
  | "pdf"
  | "image"
  | "archive"
  | "binary"
  | "folder"
  | "url"
  | "unknown";

export type AttachmentRisk = "low" | "medium" | "high";

export type AttachmentProposalStatus =
  | "pending"
  | "needs_approval"
  | "approved"
  | "denied"
  | "expired"
  | "failed";

export interface AttachmentScope {
  mode: string;
}

export interface AttachmentProposal {
  id: string;
  projectId: string;
  threadId?: string | null;
  sourceKind: AttachmentSourceKind;
  detectedKind: AttachmentKind;
  displayName: string;
  sourceLocator: string;
  proposedScope: AttachmentScope;
  estimatedBytes?: number | null;
  estimatedFileCount?: number | null;
  requiresApproval: boolean;
  approvalReason?: string | null;
  risk: AttachmentRisk;
  status: AttachmentProposalStatus;
  approvalId?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface AttachmentProposeRequest {
  projectId: string;
  threadId?: string;
  sourceKind: AttachmentSourceKind;
  displayName: string;
  sourceLocator: string;
  scopeMode?: string;
  detectedKind?: AttachmentKind;
  estimatedBytes?: number;
  estimatedFileCount?: number;
}

export type AttachmentParseStatus =
  | "not_started"
  | "reading"
  | "parsed"
  | "partial"
  | "unsupported"
  | "failed";

export type AttachmentIndexStatus = "not_indexed" | "queued" | "indexed" | "partial" | "failed";

export interface AttachmentRecord {
  id: string;
  projectId: string;
  threadId?: string | null;
  messageId?: string | null;
  runId?: string | null;
  sourceKind: AttachmentSourceKind;
  detectedKind: AttachmentKind;
  displayName: string;
  originalLocator: string;
  localReferencePath?: string | null;
  contentHash?: string | null;
  bytes?: number | null;
  parseStatus: AttachmentParseStatus;
  indexStatus: AttachmentIndexStatus;
  approvalId?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface AttachmentSnapshotView {
  projectId: string;
  threadId?: string | null;
  proposals: AttachmentProposal[];
  records: AttachmentRecord[];
}

export interface AttachmentParseResult {
  attachmentId: string;
  parseStatus: AttachmentParseStatus;
  chunkCount: number;
  partial: boolean;
}

export interface ContextPackItem {
  attachmentId?: string | null;
  evidenceRecordId?: string | null;
  locator: string;
  text: string;
  tokenEstimate: number;
  inclusionReason: string;
}

export interface ContextPack {
  id: string;
  projectId: string;
  threadId: string;
  runId?: string | null;
  strategy: string;
  budgetTokens: number;
  usedTokens: number;
  status: string;
  items: ContextPackItem[];
  createdAt: string;
  excludedCount: number;
}

/** A proposal draft built by the UI before the project/thread ids are attached. */
export type AttachmentDraft = Omit<AttachmentProposeRequest, "projectId" | "threadId">;
