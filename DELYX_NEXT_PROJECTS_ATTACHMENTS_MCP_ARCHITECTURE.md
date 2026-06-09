# Delyx Next Projects, Attachments, Drag-Drop, and MCP Architecture

Last updated: 2026-06-09

## Decision

MCP is **not** the core mechanism for Delyx Next projects or chat file upload.

Delyx Next should own projects, attachments, context packs, evidence, approvals, and run history as native first-class product state. MCP should be an optional integration boundary for external resources and tools.

In short:

```text
Delyx Project = native core state
MCP Server    = optional external provider / connector / tool bridge
```

Do not make a Delyx project "an MCP server." A project is the workbench's durable local state. MCP can attach things to that project, but it should not define the project.

## Product Direction

Delyx Next is a local-first, UI-first AI workbench. The user should be able to:

- create or open a project rooted at a local workspace path
- understand exactly what files Delyx is allowed to read
- drag/drop files into chat
- attach folders, screenshots, URLs, clipboard content, and connector resources
- approve risky reads or large imports before Delyx processes them
- see pending, approved, denied, failed, partial, and indexed attachment states
- use attached files as scoped context for one thread or the whole project
- cite attached content through EvidenceRecords when the assistant makes claims
- optionally connect MCP providers beside the native project system

## Architecture Summary

```text
Project
  -> Threads
    -> Messages
    -> Attachments
    -> Context Packs
    -> Runs
    -> Diffs
    -> Test Artifacts
    -> Evidence Records

FilePicker / DragDrop / Clipboard / URL / Connector / MCP
  -> AttachmentProposal
  -> Approval if needed
  -> AttachmentRecord
  -> Parser
  -> Context Pack
  -> Evidence Records
  -> Optional Index
```

MCP should sit on the edge:

```text
Native Delyx Project
  -> optional MCP resource provider
  -> optional MCP tool provider
  -> optional MCP external source index
```

MCP can provide resources or tools, but Delyx still decides:

- what project the resource belongs to
- what thread can use it
- whether reading it requires approval
- how much of it becomes model context
- what evidence records support claims
- whether any tool execution is safe

## Core Rule

Build native Projects, Attachments, Drag-Drop, Context Packs, and Evidence first.

Then expose/import/export/connect through MCP where it makes sense.

MCP should extend the workbench, not define the workbench.

---

# Native Project Model

## Project Domain Model

```ts
export type ProjectTrustLevel = "local" | "restricted" | "external";

export interface ProjectView {
  id: string;
  name: string;
  rootPath: string;
  trustLevel: ProjectTrustLevel;
  allowedFileScopes: FileScopeView[];
  defaultApprovalPolicy: ApprovalPolicyView;
  modelPermissions: ProjectModelPermissionsView;
  toolPermissions: ProjectToolPermissionsView;
  memoryScope: ProjectMemoryScopeView;
  evidenceIndexStatus: EvidenceIndexStatusView;
  diagnostics: ProjectDiagnosticsView;
  createdAt: string;
  updatedAt: string;
}
```

## Rust Model

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub trust_level: ProjectTrustLevel,
    pub allowed_file_scopes: Vec<FileScopeRecord>,
    pub approval_policy: ApprovalPolicyRecord,
    pub model_permissions: ProjectModelPermissionsRecord,
    pub tool_permissions: ProjectToolPermissionsRecord,
    pub memory_scope: ProjectMemoryScopeRecord,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectTrustLevel {
    Local,
    Restricted,
    External,
}
```

## Allowed File Scopes

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileScopeRecord {
    pub path: String,
    pub recursive: bool,
    pub can_read: bool,
    pub can_write: bool,
    pub reason: String,
}
```

Rules:

- Files outside an allowed read scope require a new approval.
- Folder imports above a configured size threshold require approval.
- Archive extraction always requires approval.
- Binary files should not be sent to a model directly.
- File writes remain separate from file reads and must use existing approval gates.

---

# Attachment Pipeline

## User Flows

The `+` menu and drag/drop should share the same ingestion path.

The `+` menu should include:

- Add file
- Add folder
- Add project file
- Add screenshot/image
- Add clipboard
- Add URL/source
- Add from connector
- Add from MCP provider

Drag/drop should support:

- individual files
- multiple files
- folders, where supported by the platform
- images/screenshots
- archives, with explicit approval before extraction

Clipboard should support:

- pasted text
- pasted image
- pasted file path
- copied files, where supported

## Attachment Lifecycle

```text
pending
  -> needs_approval
  -> approved
  -> reading
  -> parsed
  -> indexed
  -> attached
```

Failure states:

```text
rejected
failed
unsupported
too_large
approval_denied
approval_expired
partial
```

## Attachment Proposal

An AttachmentProposal is a preview of what Delyx wants to ingest.

```ts
export interface AttachmentProposalView {
  id: string;
  projectId: string;
  threadId?: string;
  sourceKind: AttachmentSourceKind;
  displayName: string;
  sourceLocator: string;
  proposedScope: AttachmentScopeView;
  detectedKind: AttachmentKind;
  estimatedBytes?: number;
  estimatedFileCount?: number;
  requiresApproval: boolean;
  approvalReason?: string;
  risk: "low" | "medium" | "high";
  status: AttachmentProposalStatus;
  createdAt: string;
}

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

export type AttachmentProposalStatus =
  | "pending"
  | "needs_approval"
  | "approved"
  | "denied"
  | "expired"
  | "failed";
```

## Attachment Record

An AttachmentRecord is the durable approved or accepted attachment.

```ts
export interface AttachmentRecordView {
  id: string;
  projectId: string;
  threadId?: string;
  messageId?: string;
  runId?: string;
  sourceKind: AttachmentSourceKind;
  detectedKind: AttachmentKind;
  displayName: string;
  originalLocator: string;
  localReferencePath?: string;
  contentHash?: string;
  bytes?: number;
  parseStatus: AttachmentParseStatus;
  indexStatus: AttachmentIndexStatus;
  approvalId?: string;
  createdAt: string;
  updatedAt: string;
}

export type AttachmentParseStatus =
  | "not_started"
  | "reading"
  | "parsed"
  | "partial"
  | "unsupported"
  | "failed";

export type AttachmentIndexStatus =
  | "not_indexed"
  | "queued"
  | "indexed"
  | "partial"
  | "failed";
```

## Local Copy vs Reference

Delyx should support both modes:

```text
reference = keep path/locator and read from source when approved
copy      = copy file into Delyx attachment storage
```

Recommended default:

- project files: reference
- one-off external files: copy
- screenshots/clipboard: copy
- URLs/connectors/MCP resources: store fetched snapshot + source locator
- large folders: reference + index selected files only

## Attachment Storage

Recommended local paths:

```text
%LOCALAPPDATA%/Delyx Next/attachments/<project_id>/<attachment_id>/original
%LOCALAPPDATA%/Delyx Next/attachments/<project_id>/<attachment_id>/parsed.json
%LOCALAPPDATA%/Delyx Next/attachments/<project_id>/<attachment_id>/thumb.png
```

Never store secrets inside attachment metadata. Redact known secret patterns from previews and support bundles.

---

# Context Packs

A Context Pack is the scoped model-ready subset of attachments and project files.

Do not dump full attachments into prompts by default.

```ts
export interface ContextPackView {
  id: string;
  projectId: string;
  threadId: string;
  runId?: string;
  sourceAttachmentIds: string[];
  sourceEvidenceRecordIds: string[];
  budgetTokens: number;
  usedTokens: number;
  strategy: ContextPackStrategy;
  status: "ready" | "partial" | "failed";
  createdAt: string;
}

export type ContextPackStrategy =
  | "direct_excerpt"
  | "summarized"
  | "retrieval"
  | "manual_pin"
  | "mixed";
```

Context pack rules:

- Chat should see only the selected context pack, not every file in the project.
- Large attachments should be chunked and summarized or retrieved.
- EvidenceRecords should be created for chunks that support claims.
- Context pack creation should record what was included and what was excluded.

---

# Evidence Records

Every claim based on attached files should be backed by EvidenceRecords.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentEvidenceRecord {
    pub id: String,
    pub project_id: String,
    pub thread_id: Option<String>,
    pub run_id: Option<String>,
    pub attachment_id: String,
    pub source_kind: String,
    pub title: String,
    pub locator: String,
    pub excerpt: String,
    pub content_hash: Option<String>,
    pub retrieved_at: String,
    pub relevance_score: Option<u32>,
    pub relevance_reason: Option<String>,
}
```

Evidence locators should be human-readable:

```text
file:src/app/AppShell.tsx#L61-L79
pdf:uploaded-spec.pdf#page=12
image:panel-photo.png#region=breaker-label
url:https://example.com/spec#section=4
mcp:github:owner/repo:path/to/file#L10-L30
```

---

# MCP Boundary

## What MCP Is Good For

Use MCP for optional external integration:

- attach GitHub repo resources
- query issue trackers
- query docs/source systems
- connect filesystem-like providers
- expose external tool/resource servers
- let Delyx call trusted tool endpoints with approval gates
- import source snippets into native AttachmentRecords

## What MCP Should Not Own

Do not use MCP as the core mechanism for:

- Delyx project identity
- Delyx thread state
- message persistence
- approval policy
- file attachment lifecycle
- local evidence records
- local memory scope
- run history
- test/diff artifacts
- model routing

## MCP Attachment Flow

```text
MCP resource listed
  -> user chooses resource
  -> Delyx creates AttachmentProposal(sourceKind = mcp_resource)
  -> approval if needed
  -> Delyx fetches resource snapshot
  -> Delyx creates AttachmentRecord
  -> Delyx parses/indexes into ContextPack/EvidenceRecords
```

## MCP Tool Flow

```text
Model or user requests tool
  -> Delyx validates project/tool permission
  -> Delyx creates ActionProposal if risky
  -> user approves
  -> Delyx calls MCP tool
  -> result becomes ToolArtifact / EvidenceRecord / AttachmentRecord
```

MCP tools should never bypass Delyx approvals.

---

# Database Schema

Add these tables to the SQLite migration.

## Projects

```sql
CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  trust_level TEXT NOT NULL,
  approval_policy_json TEXT NOT NULL,
  model_permissions_json TEXT NOT NULL,
  tool_permissions_json TEXT NOT NULL,
  memory_scope_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS project_file_scopes (
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  scope_index INTEGER NOT NULL,
  path TEXT NOT NULL,
  recursive INTEGER NOT NULL DEFAULT 0,
  can_read INTEGER NOT NULL DEFAULT 1,
  can_write INTEGER NOT NULL DEFAULT 0,
  reason TEXT NOT NULL,
  PRIMARY KEY (project_id, scope_index)
);
```

## Attachment Proposals

```sql
CREATE TABLE IF NOT EXISTS attachment_proposals (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  thread_id TEXT,
  source_kind TEXT NOT NULL,
  detected_kind TEXT NOT NULL,
  display_name TEXT NOT NULL,
  source_locator TEXT NOT NULL,
  proposed_scope_json TEXT NOT NULL,
  estimated_bytes INTEGER,
  estimated_file_count INTEGER,
  requires_approval INTEGER NOT NULL DEFAULT 0,
  approval_reason TEXT,
  risk TEXT NOT NULL,
  status TEXT NOT NULL,
  approval_id TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## Attachments

```sql
CREATE TABLE IF NOT EXISTS attachments (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  thread_id TEXT,
  message_id TEXT,
  run_id TEXT,
  source_kind TEXT NOT NULL,
  detected_kind TEXT NOT NULL,
  display_name TEXT NOT NULL,
  original_locator TEXT NOT NULL,
  local_reference_path TEXT,
  content_hash TEXT,
  bytes INTEGER,
  parse_status TEXT NOT NULL,
  index_status TEXT NOT NULL,
  approval_id TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_attachments_project
  ON attachments(project_id);

CREATE INDEX IF NOT EXISTS idx_attachments_thread
  ON attachments(thread_id);
```

## Parsed Chunks

```sql
CREATE TABLE IF NOT EXISTS attachment_chunks (
  attachment_id TEXT NOT NULL REFERENCES attachments(id) ON DELETE CASCADE,
  chunk_index INTEGER NOT NULL,
  kind TEXT NOT NULL,
  title TEXT NOT NULL,
  locator TEXT NOT NULL,
  text TEXT NOT NULL,
  token_estimate INTEGER NOT NULL DEFAULT 0,
  content_hash TEXT,
  PRIMARY KEY (attachment_id, chunk_index)
);
```

## Context Packs

```sql
CREATE TABLE IF NOT EXISTS context_packs (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  thread_id TEXT NOT NULL,
  run_id TEXT,
  strategy TEXT NOT NULL,
  budget_tokens INTEGER NOT NULL,
  used_tokens INTEGER NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS context_pack_items (
  context_pack_id TEXT NOT NULL REFERENCES context_packs(id) ON DELETE CASCADE,
  item_index INTEGER NOT NULL,
  attachment_id TEXT,
  evidence_record_id TEXT,
  locator TEXT NOT NULL,
  text TEXT NOT NULL,
  token_estimate INTEGER NOT NULL,
  inclusion_reason TEXT NOT NULL,
  PRIMARY KEY (context_pack_id, item_index)
);
```

## Attachment Evidence

```sql
CREATE TABLE IF NOT EXISTS attachment_evidence_records (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  thread_id TEXT,
  run_id TEXT,
  attachment_id TEXT NOT NULL REFERENCES attachments(id) ON DELETE CASCADE,
  source_kind TEXT NOT NULL,
  title TEXT NOT NULL,
  locator TEXT NOT NULL,
  excerpt TEXT NOT NULL,
  content_hash TEXT,
  retrieved_at TEXT NOT NULL,
  relevance_score INTEGER,
  relevance_reason TEXT
);
```

---

# Tauri Commands

Add a native attachment bridge. Keep the API boring and typed.

```rust
#[tauri::command]
pub fn attachment_propose(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentProposeRequest,
) -> Result<AttachmentProposalView, String>;

#[tauri::command]
pub fn attachment_approve(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentApproveRequest,
) -> Result<AttachmentRecordView, String>;

#[tauri::command]
pub fn attachment_parse(
    state: tauri::State<AttachmentBridgeState>,
    request: AttachmentParseRequest,
) -> Result<AttachmentParseResultView, String>;

#[tauri::command]
pub fn attachment_snapshot(
    state: tauri::State<AttachmentBridgeState>,
    project_id: String,
    thread_id: Option<String>,
) -> Result<AttachmentSnapshotView, String>;

#[tauri::command]
pub fn context_pack_create(
    state: tauri::State<AttachmentBridgeState>,
    request: ContextPackCreateRequest,
) -> Result<ContextPackView, String>;
```

## Frontend Client

```ts
export async function proposeAttachment(request: AttachmentProposeRequest) {
  return invoke<AttachmentProposalView>("attachment_propose", { request });
}

export async function approveAttachment(request: AttachmentApproveRequest) {
  return invoke<AttachmentRecordView>("attachment_approve", { request });
}

export async function loadAttachmentSnapshot(projectId: string, threadId?: string) {
  return invoke<AttachmentSnapshotView>("attachment_snapshot", { projectId, threadId });
}
```

---

# Parser Strategy

## Text / Markdown / Code

- read with size cap
- detect encoding
- normalize line endings
- create chunks by heading/function/line ranges
- emit EvidenceRecords with line locators

## PDF

- extract text where available
- retain page numbers
- create page-based chunks
- if text extraction fails, mark `partial` and show user-visible warning
- OCR is optional future work, not first milestone

## Image / Screenshot

- store original
- create thumbnail
- do not claim contents unless vision model or user-provided description exists
- future: local vision model or external vision adapter through explicit provider permission

## Archive

- require approval before extraction
- show file count and total bytes
- reject path traversal entries
- extract only into Delyx attachment storage
- never write archive contents into project workspace without separate file-write approval

## URL

- fetch snapshot only after approval when required
- store original URL, retrieval time, response metadata, and extracted text
- create EvidenceRecords with URL locators

## Connector / MCP Resource

- Delyx fetches a snapshot
- snapshot becomes an AttachmentRecord
- connector/MCP remains the source locator, not the project itself

---

# UI Requirements

## Composer

Add a `+` button beside the composer.

Menu items:

```text
Add file
Add folder
Add project file
Add screenshot/image
Add clipboard
Add URL/source
Add from connector
Add from MCP provider
```

## Pending Attachment Chips

Before send:

```text
[file.ts] pending
[spec.pdf] needs approval
[screenshot.png] ready
```

Each chip opens an inspector showing:

- source
- detected type
- estimated bytes
- whether approval is required
- what Delyx will read
- whether it will copy or reference
- parsing/indexing state

## Attachment Inspector States

Must render:

- empty
- pending
- needs approval
- approval denied
- approval expired
- reading
- parsed
- indexed
- partial
- failed
- unsupported

## Thread View

Messages with attachments should show:

```text
User: review this
Attachments: 3
- src/main.rs indexed
- spec.pdf partial, pages 1-8
- screenshot.png stored, not interpreted
```

## Evidence UI

When assistant cites attached content, show:

```text
Evidence
- src/main.rs L42-L68
- spec.pdf page 7
```

No evidence means no source-backed claim.

---

# Approval Policy

## No Approval Required

Generally safe:

- small file inside approved project read scope
- pasted text under size cap
- user-selected one-off text file under size cap
- screenshot/image stored but not interpreted

## Approval Required

Require ActionProposal for:

- folder imports
- recursive scans
- files outside project read scope
- archives
- large files
- many files
- URL fetches if network policy requires approval
- connector resources with external account data
- MCP resources from external systems
- binary files where metadata or extraction is requested

## Approval Proposal Text

Example:

```text
Delyx wants to read 18 files from C:\Projects\my-app\src.
Reason: Build a context pack for this thread.
Risk: Medium. This may include proprietary source code.
Scope: Read-only. No file writes. No terminal commands.
Rollback: Attachment records can be removed; no workspace files will be changed.
Expires: 15 minutes.
```

---

# MCP Implementation Later

After native attachments work, add:

```text
mcp_resource_list
mcp_resource_attach
mcp_tool_propose
mcp_tool_execute_approved
```

Do not let the model call MCP directly. Delyx mediates all calls.

## MCP Resource Record

```ts
export interface McpResourceAttachmentSource {
  serverId: string;
  resourceUri: string;
  displayName: string;
  mimeType?: string;
}
```

MCP resource attach creates a normal AttachmentProposal:

```ts
{
  sourceKind: "mcp_resource",
  sourceLocator: `mcp:${serverId}:${resourceUri}`,
  detectedKind: inferredKind,
  requiresApproval: true
}
```

---

# Implementation PR Plan

## PR 1 — Project Domain Hardening

Goal: make Project the native root state.

Tasks:

- add/extend `ProjectRecord`
- add project approval policy fields
- add allowed file scopes
- persist projects/scopes in SQLite
- expose `project_snapshot` and `project_save` bridge commands
- add tests for load/save and scope validation

Done when:

- Delyx can persist a project with root path and allowed scopes
- UI can display project trust/scope state
- no MCP concept is required for local project identity

## PR 2 — Attachment Domain Model

Goal: add typed proposals and records without parsing yet.

Tasks:

- create `attachment.rs`
- create `attachment_persistence.rs`
- create `attachment_bridge.rs`
- add `attachment_proposals` and `attachments` tables
- add proposal creation for local files and clipboard text
- add deterministic tests

Done when:

- attachments can be proposed and persisted
- approval-required state is represented
- denied/expired proposal states remain visible

## PR 3 — Composer `+` Menu and Drag-Drop Shell

Goal: one UI path for menu and drag/drop.

Tasks:

- add `AttachmentMenu`
- add `PendingAttachmentTray`
- add drag/drop handlers to composer area
- call `attachment_propose`
- render pending chips and failure states
- no parsing yet

Done when:

- user can pick/drop a file and see a truthful pending/needs-approval chip
- unsupported files show unsupported state
- tests cover menu and drag/drop path using fake file objects where possible

## PR 4 — Approval-Gated Attachment Acceptance

Goal: approvals are enforced before risky reads/imports.

Tasks:

- classify risk by file count/size/scope/source kind
- create ActionProposal when needed
- add `attachment_approve`
- convert approved proposals into AttachmentRecords
- link approval IDs to attachment records
- tests for no approval, required approval, denied approval, expired approval

Done when:

- large folder/import cannot become an AttachmentRecord without approval
- small safe file can become an AttachmentRecord without unnecessary friction
- all states render in UI

## PR 5 — Text/Code/Markdown Parsing

Goal: parse simple local text/code into chunks.

Tasks:

- add `attachment_parser.rs`
- add `attachment_chunks` table
- support text, markdown, code
- add line-range locators
- cap bytes per file
- mark partial when truncated
- tests for line locators and truncation

Done when:

- attached code/text can become chunks with stable locators
- large files become partial, not fake-complete

## PR 6 — Context Packs

Goal: attachments become scoped model context.

Tasks:

- add `context_pack.rs`
- add `context_pack_create`
- add context pack tables
- select chunks by pinned/manual first, then budgeted chunks
- create ContextPack view in thread inspector
- tests for budget and inclusion reasons

Done when:

- model calls can receive selected attachment context
- UI shows what was included/excluded

## PR 7 — Evidence from Attachments

Goal: claims can cite attached files.

Tasks:

- create `attachment_evidence_records`
- generate EvidenceRecords from context pack chunks
- wire evidence IDs into AgentRun receipts where relevant
- UI shows evidence from attachments
- tests for locator integrity

Done when:

- final answers can cite attached file chunks
- no unsupported source-backed claim is shown without evidence

## PR 8 — PDF/Image/Archive Handling

Goal: broaden file support safely.

Tasks:

- PDF text extraction with page locators
- image storage/thumbnail only
- archive preview + approval + safe extraction into attachment storage
- tests for archive path traversal rejection
- tests for PDF partial/failure states

Done when:

- PDFs become page chunks when extractable
- images do not produce claims unless interpreted later
- archives cannot write into workspace or escape extraction root

## PR 9 — URL/Connector Attachment Sources

Goal: external source snapshots become native AttachmentRecords.

Tasks:

- add URL proposal and fetch snapshot path
- connector source abstraction
- approval where network/external data applies
- EvidenceRecords with URL/connector locators
- tests for fetch failure and snapshot persistence

Done when:

- external resources enter Delyx through native attachment records
- source locator and retrieval time are preserved

## PR 10 — MCP Resource Attachments

Goal: MCP extends native attachments.

Tasks:

- list MCP resources from configured server
- attach selected MCP resource as AttachmentProposal
- approval required by default
- fetch snapshot through Delyx bridge
- parse/index like other attachments
- tests with fake MCP resource provider

Done when:

- MCP resources become AttachmentRecords
- MCP does not own project/thread state
- MCP cannot bypass approvals

## PR 11 — MCP Tool Approval Boundary

Goal: MCP tools are optional, approval-gated external tools.

Tasks:

- register MCP tools as external tool capabilities
- propose tool call before execution when risky
- execute only approved calls
- store ToolArtifact and optional EvidenceRecords
- tests for denied/expired approvals

Done when:

- model/tool requests cannot call MCP directly
- Delyx approval engine mediates all MCP tool execution

## PR 12 — Polish, Diagnostics, and Support Bundle

Goal: make it debuggable and supportable.

Tasks:

- attachment diagnostics panel
- parser/indexer error details with redaction
- support bundle redacts attachment contents by default
- export metadata-only attachment report
- tests for no secret leakage

Done when:

- user can understand why an attachment failed
- support bundles do not leak file contents/secrets by default

---

# File/Module Map

Recommended Rust files:

```text
apps/desktop/src-tauri/src/project.rs
apps/desktop/src-tauri/src/project_bridge.rs
apps/desktop/src-tauri/src/project_persistence.rs
apps/desktop/src-tauri/src/attachment.rs
apps/desktop/src-tauri/src/attachment_bridge.rs
apps/desktop/src-tauri/src/attachment_persistence.rs
apps/desktop/src-tauri/src/attachment_parser.rs
apps/desktop/src-tauri/src/attachment_context_pack.rs
apps/desktop/src-tauri/src/attachment_evidence.rs
apps/desktop/src-tauri/src/mcp_attachment.rs
apps/desktop/src-tauri/src/mcp_tool_bridge.rs
```

Recommended frontend files:

```text
apps/desktop/src/features/attachments/attachmentTypes.ts
apps/desktop/src/features/attachments/attachmentClient.ts
apps/desktop/src/features/attachments/AttachmentMenu.tsx
apps/desktop/src/features/attachments/PendingAttachmentTray.tsx
apps/desktop/src/features/attachments/AttachmentInspector.tsx
apps/desktop/src/features/attachments/attachmentReducer.ts
apps/desktop/src/features/attachments/attachmentFormat.ts
```

Keep files small. Split before files become broad god-modules.

---

# Testing Matrix

## Rust Unit Tests

Must cover:

- project scope validation
- safe file proposal
- risky folder proposal
- approval required classification
- approval denied blocks record creation
- approval expired blocks record creation
- text chunk line locators
- truncation produces partial status
- archive path traversal rejection
- context pack token budget
- EvidenceRecord locator generation
- MCP resource attach cannot bypass approval

## Frontend Tests

Must cover:

- `+` menu renders supported attachment choices
- drag/drop and menu use same proposal path
- pending chips render status
- failed/unsupported states stay visible
- attachment inspector displays read scope/risk
- model send includes context pack only after attachment is ready
- no fake indexed state appears before parse/index completes

## Smoke Tests

Add scripts or extend existing smoke checks to assert:

- no hardcoded fake attachments in shipped UI
- no MCP-only project model language
- attachment states include failed/partial/denied/expired
- risky import copy says approval required

---

# Non-Goals for First Milestone

Do not build these first:

- full vector database for every project file
- automatic whole-drive indexing
- background recursive scanning without approval
- model-initiated direct MCP calls
- OCR as a required path
- image understanding claims without a vision model/evidence path
- auto-extracting archives into project folders
- syncing project state into an MCP server

---

# Definition of Done

This feature is 100% done when:

- Projects are native Delyx records with path, scopes, policy, and diagnostics.
- Chat supports `+` menu attachments.
- Drag/drop uses the same attachment proposal pipeline as the `+` menu.
- Risky reads/imports require ActionProposal approval.
- Approved attachments persist as AttachmentRecords.
- Text/code/markdown/PDF attachments can become scoped Context Packs.
- Assistant claims from attachments can cite EvidenceRecords.
- Failed, partial, unsupported, denied, expired, loading, and ready states are visible.
- MCP resources can be attached through the same native pipeline.
- MCP tools are approval-gated and cannot bypass Delyx safety policy.
- No shipped UI invents fake attachment/index/evidence state.
- Tests prove the core states and safety gates.

## Final Architecture Statement

Delyx Next should treat projects and attachments as core native product primitives.

MCP should be an optional integration layer for external resources and tools.

The project is Delyx's local trust boundary.

Attachments are Delyx's context boundary.

EvidenceRecords are Delyx's truth boundary.

Approvals are Delyx's safety boundary.

MCP belongs outside those boundaries unless Delyx explicitly imports or executes through them.
