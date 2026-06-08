# Delyx Next Architecture

Delyx Next should be local-first, UI-first, typed, and evidence-oriented.

Keep architecture proportional to the current milestone. Avoid empty module structures that exist only because the target architecture is large.

## Stack

Default stack:

- Tauri v2
- React
- TypeScript
- Rust
- SQLite (Phase 2 target; AgentRun, AgentOutcome support links, thread/run, approval, recent workspace, model-route, memory-store, skill-registry, automation-engine, release-state, test-artifact, patch-proposal, review-report, external-agent-run, and research-evidence persistence are wired first)
- Vite
- CSS variables for design tokens
- Radix UI primitives where useful
- Lucide icons

Use old Delyx's Tauri/React/TypeScript/Rust/SQLite direction unless a decision record proves otherwise.

## Current Implementation Reality

As of 2026-06-08, the repo is Phase 1 skeleton-complete, not functionally
complete:

- Rust is a single crate in `apps/desktop/src-tauri`; the multi-crate target is
  an extraction map, not current state.
- SQLite is partially implemented. `agent_run_persistence.rs` now uses
  `rusqlite` and the migration artifact for AgentRun save/load, including
  nodes, events, artifacts, evidence, outcome summary, and outcome support ID
  links. The Tauri
  thread/run bridge also persists task threads, messages, run links, and
  AgentRun rows to the local SQLite database. The approval bridge persists
  proposals, UI scope, status, and decisions. Recent workspace project snapshots
  persist project metadata, approved roots, rules files, Git metadata, and
  indexed file names. Model role routes persist to SQLite and are validated
  against current provider health before becoming active. The Rust memory
  governance store persists candidates, durable records, suppression state, and
  ID continuity. Its Tauri bridge now exposes persisted candidate propose,
  approved promote, candidate suppress, record suppress, and snapshot commands.
  Durable memory promotion still requires a matching approved memory-save
  proposal and a completed source run. The Rust skill registry persists imported
  manifests, trust, permissions, status, source hashes, and ID continuity. Its
  Tauri bridge exposes persisted import, activate, disable, suppress, and
  snapshot commands; activation changes registry state only and does not execute
  any skill capability by itself.
  The Rust automation engine persists mission contracts and scheduled-run
  records. Its Tauri bridge exposes persisted contract create, approved
  activate, pause, due-run schedule, and snapshot commands. Activation still
  requires a matching approved scheduled-action proposal. Risky scheduled runs
  generate persisted approval cards through the approval bridge before execution.
  Release profile, release smoke, redacted support-bundle state, and
  support-bundle file export receipts persist to
  SQLite. The release Tauri bridge exposes persisted profile save, smoke
  capture, latest support-bundle export, approval-gated file export, and
  snapshot commands. Support-bundle config/log entries are redacted before
  persistence. Approved test artifacts,
  proposed patch diffs, review reports, external-agent run artifacts, and
  research EvidenceStore receipts persist receipt data to SQLite and reload
  with ID continuity. AgentRun EvidenceRecords persist source IDs, locators,
  quotes, hashes, retrieval timestamps, and relevance metadata. AgentOutcome
  evidence/test support ID links also persist and the thread/run snapshot
  bridge exposes receipts and outcome support links to the UI. A narrow
  final-answer support bridge now links existing AgentRun EvidenceRecord IDs and
  passed persisted test artifact IDs into AgentOutcome and saves the result to
  SQLite. Narrow patch executor bridges can now wait on pending `FileWrite`
  approvals, run approved AgentRun patch-proposal, patch-apply, and
  patch-restore nodes, approved test-execution nodes, and read-only review
  nodes. Patch nodes persist patch state, write files only through the
  stale-file/checkpoint PatchEngine path, and record node events, artifact IDs,
  and diff evidence receipts. Test nodes reuse the TestRunner command-shape,
  approval, cwd, timeout, output capture, and persistence path before recording
  AgentRun test receipts. Review nodes gather persisted PatchProposal and
  TestArtifact records by run ID, then create ReviewReport receipts without
  write capability. A narrow AgentScheduler bridge now reads the persisted
  AgentRun, approval, patch, test, and review stores to choose conservative
  next-step decisions: wait, single-approval resume, patch apply, tests, review,
  final-support readiness, terminal, complete, or blocked. The Focus UI can
  render that decision as a next-action line without inventing runtime state.
  Resume actions forward the active plan's supported test-command signal back
  into the scheduler bridge, so resume decisions use the same real plan context
  as the visible scheduler line. After the non-risky resume transition, the
  bridge returns a post-resume scheduler decision when persisted patch, test, or
  review work is ready; otherwise it falls back to the visible resume decision.
  Approval decisions can also trigger that non-risky resume transition when the
  approved proposal is the last pending approval for the run. A focused
  renderer dispatcher can then run post-resume scheduler-selected actions:
  approved patch apply, approved or approval-queued tests, read-only review, or
  final-support recording. After each dispatched action it asks the Rust
  scheduler for a bounded next decision, stopping on passive/repeated states or
  the depth limit. A separate `agent_execute_patch_draft` bridge can perform the
  approved plan-file read, local Ollama PatchDraftAgent call, Rust JSON parse,
  model-call receipt recording, and patch-proposal capture path. That bridge is
  still a narrow renderer-invoked command, not the full executor/repair loop.
  After the bridge persists a proposed diff, the renderer reloads patch/run
  receipts and re-enters the scheduler dispatcher with the fresh patch list so
  the next apply/test/review decision can be queued or continued through normal
  approval boundaries. It does not apply generated patches, bypass approvals, or
  run arbitrary tools. Applied patch cards also expose checkpoint rollback state
  and can queue or execute a separate restore approval through the existing
  AgentRun restore bridge.
  Remaining governance/action bridges are still not live.
- There is no full AgentRun multi-node autonomous executor, repair loop, or hook
  runner yet.
- Frontend checks now include a narrow Vitest/React Testing Library component
  test path, starting with FocusThread's approval-gated patch apply behavior.
  MarkdownMessage rendering is also covered so assistant replies do not regress
  to raw markdown. The older smoke/source-contract verifiers remain as
  guardrails, not proof of behavior.
- The default workbench now uses a React Focus shell ported from the provided
  Focus prototype: centered first-run composer, rail navigation, command
  palette, thread switcher, model picker, settings, and artifact-driven active
  thread view. Legacy `buildCockpitMarkup` string-rendered files remain in
  source as the older cockpit implementation and smoke-contract reference, but
  they are not the mounted primary workbench. The Focus UI avoids shipped fake
  data and renders real project, thread, model, run, approval, diff, and test
  state only.

## Codex Reference Integration

`openai/codex` is a reference and salvage pool, not the Delyx architecture.
The audited reference commit is `e093d81`; details live in
`docs/CODEX_REFERENCE_AUDIT.md`.

Rules:

- import or adapt only pieces that reduce risk or save real implementation time
- keep Delyx approval gates and UI-visible runtime state around every imported behavior
- document dependency weight before adding broad crates
- prefer small local adapters over wholesale crate-graph imports
- keep cloud/auth paths optional and explicit

Current Codex-inspired local adaptation:

- `terminal_command_prep` applies best-effort PowerShell UTF-8 output setup for
  approved external worker commands without changing the visible approved
  command label.
- `command_exec` adapts Codex app-server command execution protocol ideas into
  a small local `CommandExecArtifact`: command label, cwd, approval/run IDs,
  timeout, exit status, duration, capped stdout/stderr, truncation flags, and
  timeline events. It is used only after Delyx approval gates have already
  authorized the test command or external terminal worker.

## Source File Size Budget

Keep source files small enough to review comfortably:

- target: 300 lines or fewer
- review/split threshold: 400 lines
- hard cap: 500 lines

Exceptions are allowed for generated code, declarative config, snapshots/fixtures, or a documented architecture decision.

When a source file approaches the threshold, split by responsibility:

- React components
- hooks
- view-model builders
- Rust services
- repository methods

Rust source is formatted with `rustfmt`, and `npm test` enforces the 300-line
source budget for `.rs`, `.ts`, `.tsx`, and `.css` files.
- policy evaluators
- type definitions

## Target Runtime Shape

```text
Delyx Next
  Workspace Manager
  Thread Manager
  Agent Runtime
  Permission Engine
  Tool Layer
  Model Layer
  Evidence Layer
  UI
```

## Workspace Manager

Owns:

- projects
- approved workspace roots
- Git state
- read-only file index/search
- rules file detection
- checkpoints
- worktrees later

Early rules:

- reads outside approved roots fail
- writes are not implemented before approval engine and patch system
- workspace scope is visible in UI
- Tauri `workspace_snapshot` exposes approved-root metadata, rules files, and indexed file names only
- Tauri `workspace_snapshot` stores the latest real workspace snapshot in SQLite so the UI can reopen with recent project state
- Git dirty counts come from read-only Git index metadata when available; otherwise they stay unknown

## Thread Manager

Owns:

- task threads
- conversation state
- active mode
- thread status
- linked AgentRuns
- archive/update flow

Threads are scoped to projects.

## Agent Runtime

Owns:

- AgentRun graph
- AgentNode execution
- AgentEvent log
- mode-specific agents
- lifecycle hooks
- resume/repair later

Core agents:

- ExploreAgent
- PlanAgent
- BuildAgent
- ReviewAgent
- TestAgent
- ResearchAgent

The AgentRun graph is the future execution/resume engine, not just an inspector artifact.
Current state: the graph is still primarily an inspection and bridge artifact,
with narrow scheduling and execution paths. `AgentScheduler` can identify
approval waits, resume exactly one ready approval, select approved proposed
patches for apply, require supported test-command evidence after applied
patches, select review from stored patch/test artifacts, and report
final-support readiness after a stored review. `agent_schedule_next` exposes
that decision to the UI, and `agent_resume_waiting_run` persists the non-risky
resume transition when exactly one approval is executable. It does not yet
dispatch the whole loop by itself. The `agent_execute_patch_proposal` bridge can
advance a run from a pending approval boundary into an approved patch-proposal
node, persist the patch proposal, and record node events, artifact IDs, and diff
evidence receipts. The `agent_execute_patch_draft` bridge owns the approved
generated-patch slice before that proposal node: it reads only scoped approved
plan files, calls local Ollama, records model-call receipts, rejects truncated,
unapproved, empty, duplicate, or unchanged generated file contents, and then
feeds the approved proposal bridge. It remains a narrow command invoked by the
renderer after approval, not an autonomous repair-loop node. The
`agent_execute_patch_apply` bridge can advance an
approved existing PatchProposal into a patch-apply node, write files through the
stale-file/checkpoint PatchEngine path, and record patch-apply receipts. The
`agent_execute_patch_restore` bridge can advance an approved existing applied
PatchProposal into a patch-restore node, require a separate executable
`FileWrite` approval, verify current file contents still match the applied
patch, restore/remove checkpointed files, and record rollback receipts. The
`agent_execute_test_run` bridge can advance an approved test command into a
test-execution node, run only commands accepted by the TestRunner, persist the
TestArtifact, and record test evidence. The `agent_execute_review` bridge can
advance persisted patch/test artifacts into a read-only review node and
ReviewReport artifact. The Focus thread view loads persisted patch, test, and
review receipts for the active run; it shows scheduler-selected next actions
from the Rust bridge, can resume a single ready approval, and can trigger
existing patch/test/review/final-support actions only from real receipts.
PatchDraft success also reloads real receipts before dispatching the next
scheduler decision, preventing generated patches from relying on stale renderer
state. Other runtime islands execute real work
outside the full graph: Ollama chat/plan calls, the generic terminal-worker
bridge, and Codex CLI read-only launches. The full Explore -> Plan -> Approve
-> Build -> Diff -> Test -> Review execution loop remains Phase 2 work. AgentRun
save/load, Tauri
thread/run bridge session reload, approval bridge reload, recent workspace
project reload, test artifact bridge reload, patch proposal bridge reload,
review report reload, and external-agent run artifact reload now use SQLite.
Research EvidenceStore receipts also persist to SQLite, including source kind,
locator, excerpt, stance, and normalized claim keys. AgentRun EvidenceRecords
also persist non-lossy receipt metadata, and AgentOutcome support ID links are
serialized through desktop thread/run snapshots.

## Permission Engine

Owns:

- read policy
- write policy
- terminal policy
- network policy
- memory policy
- automation policy
- approval proposal creation
- approval decision enforcement

Risky actions must produce an ActionProposal before execution.
Desktop approval proposals and approve/deny decisions are mirrored through a
Tauri approval bridge backed by the Rust ApprovalEngine. The bridge returns the
same UI-ready proposal shape as the renderer, rejects unsupported non-risky
approval actions, deduplicates repeated plan-approval requests by client ID, and
keeps expired decisions visible instead of moving a run forward. Web preview
retains the local renderer fallback.
Persistent desktop runs restore approval proposals and decisions from SQLite so
pending, approved, denied, and expired cards remain inspectable after restart.
The `approval_taxonomy` command exposes the Rust risk taxonomy as a read-only
UI snapshot. The renderer merges that snapshot into its web-preview fallback so
approval policy labels come from the same minimum-risk floors used by execution
gates when Tauri is available. Focus state also reloads persisted approval
proposals and decisions for the active run so patch/test controls can reason
from real approval status instead of renderer-only memory.

## Tool Layer

Owns:

- file read/search
- patch proposal
- patch apply
- terminal command runner
- test runner
- Git tools
- Python sandbox
- external agent bridge

Tool output should become artifacts or evidence records when relevant.
The Tauri `patch_propose` bridge exposes proposal-only diffs from explicit
approved-root file content requests. It uses the Rust PatchEngine to read
before contents and build UI-ready diff records. Proposed patch records persist
proposal IDs, approval/run IDs, status, checkpoint IDs, file paths, before/after
text, and diff lines to SQLite so diff receipts survive restart. The
`patch_apply_approved` bridge applies an existing proposal only when the
matching `FileWrite` approval is executable, the file still matches the proposed
before text, and the target stays inside approved roots. It writes through the
checkpointing PatchEngine and persists checkpoint file receipts for review and
rollback. The `patch_restore_approved` bridge requires its own executable
`FileWrite` approval, verifies the file still matches the applied `after` text,
stays inside approved roots, and then writes or removes files from the persisted
checkpoint receipts while recording the restore approval ID.
The Focus UI loads `patch_snapshot` for the active run so persisted patch
receipts can surface as real diffs; it does not synthesize placeholder diffs
when the runtime has not produced a PatchProposal. Focus diff controls may call
the AgentRun patch-apply bridge only when the matching approval is visibly
approved in active Focus state. The Rust bridge still owns final approval,
root, stale-file, and checkpoint enforcement before any file write. Applied
patch cards show checkpoint state and can request a separate restore approval;
only an approved restore proposal ID is sent to the AgentRun patch-restore
bridge, which still owns stale-after and approved-root enforcement before any
rollback write. The same Focus diff receipt now shows checkpoint file paths,
the restore approval ID after rollback, stale restore failure messages from the
AgentRun event stream, and post-restore review guidance without inventing
rollback state.
Focus approval decisions do not mark a thread as building by themselves. They
return to the next-step state, or keep waiting when more approvals are pending,
until a concrete executor or tool action starts.
The Tauri `test_run_approved` bridge exposes approved test-command execution
through the Rust TestRunner. It reads the same Rust ApprovalEngine owned by the
approval bridge, rejects pending or mismatched approvals, captures stdout,
stderr, exit code, duration, output truncation state, command execution events,
and failure summary, and stores UI-ready TestArtifactView records by run. The
desktop bridge persists those test artifacts, parsed failures, and command exec
events to SQLite so test receipts survive restart.
The Tauri `review_create` bridge exposes the read-only Rust ReviewAgent for
real patch and test artifacts. It returns UI-ready ReviewReportView findings
without write capability and rejects not-run test artifacts so review cannot
turn missing execution into tested claims.
Review reports persist to SQLite, including summaries, decisions, ordered
findings, hunk labels, priorities, and suggested fixes. This preserves what
Delyx reviewed across restart; it does not make the full build/test/review loop
autonomous yet.
Codex CLI and Claude Code adapter detection reads PATH only. Their typed
command contracts produce visible `codex exec` and `claude -p` command arrays
with explicit permission mode, transcript format, working directory, and Delyx
tool requirements. Codex CLI read-only launch now flows through external-agent
approval, terminal-command approval, captured output, and UI-visible artifacts.
Those external-agent run artifacts persist to SQLite with status, scope,
terminal output, diff summary, review-required state, ordered transcript events,
and linked test artifact IDs.
Approved external terminal workers use the shared command execution artifact so
their transcript events, stdout/stderr, exit status, and duration follow the
same receipt shape as tests.
Codex write-capable launch and any diff-capturing external run still require a
real checkpoint or isolated worktree. Claude launch remains preview-only.

## Memory Governance

Owns:

- memory candidates
- approval-gated promotion
- durable memory records
- suppression and supersession
- source run/thread receipts

The Rust `MemoryStore` persists candidates and records to SQLite, including
candidate status, record suppression, source run/thread IDs, and post-reload ID
continuity. The Tauri memory bridge exposes UI-ready snapshots plus persisted
candidate propose, approved promote, candidate suppress, and record suppress
commands. Promotion uses the shared approval engine and rejects unapproved,
mismatched, expired, wrong-action, or non-completed-source-run saves. Frontend
controls for those mutations are still shallow; the bridge and snapshots are the
current source of truth.

## Skills

Owns:

- imported skill manifests
- source hashes
- local vs third-party trust
- activation, disabled, and suppressed states
- explicit skill permissions

The Rust `SkillRegistry` persists imported manifests to SQLite, including trust,
status, permissions, source hashes, and post-reload ID continuity. The Tauri
skill bridge exposes UI-ready snapshots plus persisted import, activate,
disable, and suppress commands. Activation requires explicit permission flags
and only changes local registry state; separate execution paths must still check
those permissions before running scripts, editing files, or using network.
Frontend controls for those mutations are still shallow; the bridge and
snapshots are the current source of truth.

## Automations

Owns:

- mission contracts
- approval-gated activation
- active hours and delivery targets
- allowed tool scope
- workspace drift blocking
- scheduled-run records

The Rust `AutomationEngine` persists mission contracts and scheduled runs to
SQLite, including status, schedule shape, allowed tools, approval links,
workspace fingerprints, and post-reload ID continuity. The Tauri automation
bridge exposes UI-ready snapshots plus persisted contract create, approved
activate, pause, and due-run schedule commands. Activation uses the shared
approval engine and rejects unapproved, mismatched, expired, or wrong-action
approvals. Scheduled-run creation for risky tools creates a persisted approval
proposal and matching approval bridge record so the UI can show the pending
approval before execution.

## Release

Owns:

- Windows dev release profile
- signing readiness
- update metadata status
- support bundle export shape
- secret redaction policy

Release profile, latest release smoke receipt, latest redacted support bundle,
and latest support-bundle file export receipt persist to SQLite, including
signing inputs, update channel/published state, installer smoke status,
installer path, smoke command, captured timestamp, redacted config summary,
redacted logs, support-bundle metadata, exported file path, approval ID,
export timestamp, and bytes written. The Tauri release bridge exposes UI-ready
snapshots plus persisted profile save, smoke capture, latest support-bundle
export, and approval-gated support-bundle file export commands. Support-bundle
config/log entries are redacted before they are stored, and file export requires
a matching approved `FileWrite` proposal plus an approved root. Signing
execution and update publishing still need explicit bridges before this becomes
an end-to-end release workflow.

## Model Layer

Owns:

- mock provider
- Ollama provider
- OpenAI-compatible provider
- role routing
- model health checks
- missing provider/API-key states

The implementation keeps a deterministic mock provider for fixtures and backend
provider tests, but the frontend does not select it as the live user-facing
route. Local Ollama is the first real route for composer calls and read-only
PlanAgent drafts when `127.0.0.1:11434` is reachable. Ollama plans must parse
into typed PlanView JSON before appearing in the UI, and each successful or
failed model call is recorded in the AgentRun ledger.
Role routing may only save routes to providers whose health is ready; missing-key,
unconfigured, or unreachable providers remain visible but unusable.
Model role routes are stored locally in SQLite. Runtime status reloads valid
routes first, then saves the first ready Ollama coding route only when no valid
coding route exists.
The read-only Tauri `runtime_status` command exposes app identity and provider
status to the UI without executing tools or storing secrets. It probes local
Ollama through loopback `/api/tags`, promotes discovered local models into the
runtime coding route, parses tags with `serde_json`, and reports unreachable
or empty states truthfully. The renderer applies that desktop runtime status
to model settings on first load and uses direct browser Ollama probing only
when the Tauri bridge is unavailable. It does not expose the deterministic mock
provider or a mock coding route as live runtime state.
The Tauri `ollama_chat` command owns desktop `/api/chat` model calls so local
agent replies and read-only PlanAgent drafts flow through the runtime bridge
instead of relying on renderer-only networking. The command validates the
selected model and message roles before the loopback request, returns a
provider/model/text artifact shape, and preserves the renderer HTTP path only
for web preview where no Tauri bridge exists.
Composer and PlanAgent thread messages are also mirrored through the Tauri
thread/run bridge in desktop mode, so real Ollama user, assistant, and system
messages are visible in bridge snapshots instead of living only in renderer
state. The bridge validates roles, non-empty bodies, linked thread records, and
typed status transitions; it does not grant model calls any file, terminal,
connector, memory, scheduled-work, or external-agent authority.
The frontend model view type does not include a live mock provider kind; unknown
or unsupported runtime provider kinds map to an unavailable UI state.

The Windows desktop shell is explicit runtime state, not hidden packaging
metadata. Tauri owns the stable `main` window, focuses it on startup, and uses
the official single-instance plugin to bring the existing window forward when
the app is launched again. Native menu policy is intentionally
`renderer_command_ui` for now because command palette/settings controls are the
primary UI surface. Dev packaging remains unsigned and is reported as
`unsigned_dev_build`.

Model roles:

- answer
- helper
- deepResearch
- maxReasoning
- coding
- embedding
- scoring

Secrets must not be stored in the repo.
OpenAI-compatible providers are currently represented by health/config states,
not real chat/completion execution. The frontend maps that backend stub to an
unavailable/not-wired UI state instead of suggesting that adding an API key would
make it usable. Ollama is the only live model execution path.

## Evidence Layer

Owns:

- diffs
- test output
- terminal logs
- source receipts
- file hashes
- model-call records
- external agent transcripts
- final answer support

Evidence records should make claim support inspectable.
The Rust `EvidenceStore` now persists source-backed research receipts to
SQLite, including run IDs, source kind, title, locator, excerpt, stance, and
normalized claim keys. This preserves research receipts across restart, but it
does not yet build final-answer support records automatically from every file,
diff, command, approval, model call, or external-agent artifact. The
`thread_final_answer_record` bridge can finish a thread by linking current
AgentRun evidence IDs and passed persisted test artifact IDs, emitting a
visible support-synthesis event, and persisting those AgentOutcome support
links. It does not infer claim support from prose or make unsupported/tested
states complete by itself.

## Data Models

### Project

```ts
interface Project {
  id: string;
  name: string;
  path: string;
  approvedRoots: string[];
  git?: {
    isRepo: boolean;
    branch?: string;
    hasUncommittedChanges: boolean;
    remoteUrl?: string;
  };
  rulesFiles: ProjectRulesFile[];
  modelProfileId?: string;
  createdAt: string;
  updatedAt: string;
}
```

### TaskThread

```ts
interface TaskThread {
  id: string;
  projectId: string;
  title: string;
  goal: string;
  status: TaskStatus;
  mode: AgentMode;
  activeRunId?: string;
  runIds: string[];
  createdAt: string;
  updatedAt: string;
}
```

### AgentRun

```ts
interface AgentRun {
  id: string;
  projectId?: string;
  threadId?: string;
  parentRunId?: string;
  goal: string;
  mode: AgentMode;
  status:
    | "created"
    | "running"
    | "waiting_for_approval"
    | "blocked"
    | "repairing"
    | "succeeded"
    | "failed"
    | "cancelled";
  nodes: AgentNode[];
  events: AgentEvent[];
  artifacts: Artifact[];
  evidence: EvidenceRecord[];
  metrics: RunMetrics;
  outcome?: AgentOutcome;
  createdAt: string;
  updatedAt: string;
}
```

### ActionProposal

```ts
interface ActionProposal {
  id: string;
  runId: string;
  nodeId: string;
  actionType:
    | "read_file"
    | "write_file"
    | "edit_file"
    | "run_terminal"
    | "install_dependency"
    | "save_memory"
    | "use_connector"
    | "schedule_work"
    | "external_send"
    | "external_agent";
  riskLabel: "low" | "medium" | "high" | "dangerous";
  requiredPermission: string;
  rationale: string;
  expectedResult: string;
  rollbackPlan?: string;
  scope: PermissionScope;
  expiresAt: string;
  status: "pending" | "approved" | "denied" | "expired";
}
```

### EvidenceRecord

```ts
interface EvidenceRecord {
  id: string;
  runId: string;
  sourceKind:
    | "local_file"
    | "repo_symbol"
    | "terminal"
    | "test"
    | "diff"
    | "web"
    | "memory"
    | "external_agent"
    | "model_call";
  sourceId: string;
  title?: string;
  uri?: string;
  quote?: string;
  hash?: string;
  retrievedAt: string;
  relevance?: {
    relationship:
      | "direct_implementation"
      | "caller"
      | "test"
      | "config"
      | "doc"
      | "name_only"
      | "unknown";
    score: number;
    reason: string;
  };
}
```

### AgentOutcome

```ts
interface AgentOutcome {
  status: "succeeded" | "failed" | "blocked";
  summary: string;
  evidenceRecordIds: string[];
  testArtifactIds: string[];
}
```

## Decision Records

Record architecture decisions here until a dedicated ADR folder exists.

### ADR-0001: Clean Rebuild With Old Delyx As Baseline

Decision: Delyx Next is a clean rebuild. Old Delyx is used as reference, spec, eval source, safety-policy source, and salvage pool.

Reason: The new product direction requires a UI-first project/thread/diff workflow and should avoid copying old cockpit-first complexity.

### ADR-0002: Separate App Identity

Decision: Use `com.geaux.delyxnext` unless later packaging work changes it.

Reason: Old Delyx and Delyx Next should coexist during development and migration.

### ADR-0003: CSS Variables First

Decision: Use CSS variables and shared React components for PR 2 design tokens. Do not add Tailwind unless a later decision record justifies it.

Reason: The app needs stable product-specific tokens and a small, controlled design system before broad styling utilities.

### ADR-0004: Windows-First Unsigned Dev Packaging

Decision: PR 18 targets Windows NSIS packaging first. Dev builds are explicitly unsigned until certificate, digest, timestamp, and sign command inputs are configured together.

Reason: Packaging should be testable without pretending production signing exists. Support bundles redact logs and config summaries before export, and update metadata remains a disabled local placeholder.

### ADR-0005: Separate Web Preview And Desktop Dev Commands

Decision: `npm run dev` starts the local Vite web preview, and `npm run dev:desktop` starts the Tauri Windows desktop shell.

Reason: Delyx Next needs a fast browser preview for UI iteration and a clear native desktop path for app-shell QA. The two paths should not be confused when judging Windows desktop polish.

### ADR-0006: Windows Single-Instance Desktop Shell

Decision: Delyx Next uses Tauri's official single-instance plugin and keeps native app menus disabled in favor of the renderer command UI.

Reason: A refined Windows desktop app should not create duplicate local-agent sessions when launched twice. The dependency is narrow and official, and the renderer command palette remains the product's visible trust/control surface.

### ADR-0007: Focus Shell Is The Mounted Workbench

Decision: `FocusShell` is the live mounted workbench. Legacy string-rendered
cockpit modules, including `buildCockpitMarkup`, are deprecated as product UI
and remain only as older smoke-contract/reference code until a focused cleanup
can delete or replace their verifier coverage.

Reason: The product direction is a simple Codex-like project/thread workspace
with real runtime receipts. Keeping the old cockpit mounted would preserve the
confusing dashboard furniture the UI cleanup removed, while deleting every old
contract at once would erase useful guardrails before equivalent Focus behavior
tests exist.

### ADR-0008: Single Rust Crate Until Extraction Pressure

Decision: Delyx Next keeps one Rust crate at `apps/desktop/src-tauri` for the
current Phase 2 work. The target runtime map names domains and future extraction
points, not crates that must exist before the code has earned them.

Reason: Splitting crates now would mostly create architecture theater. The
current risk is execution depth, persistence, approvals, receipts, and UI truth;
crate extraction should happen only when compile boundaries, ownership, or test
isolation start paying for the added workspace shape.

### ADR-0009: PatchDraftAgent Proposes, It Does Not Apply

Decision: Approved plan/build approval can trigger a narrow
`agent_execute_patch_draft` PatchDraftAgent bridge. The bridge reads only scoped
approved plan files, asks local Ollama for structured complete replacement
contents, records model-call receipts, parses and validates the returned JSON in
Rust, and records a proposed diff through the AgentRun patch proposal bridge.
Applying that generated proposal requires a separate apply approval ID in the
patch apply request; the proposal approval is not accepted as write
authorization by the renderer action or the Rust apply bridge.

Reason: The current approval copy scopes this action to proposing a patch. File
writes must remain visible through the existing patch apply/checkpoint gates, so
generated content and disk mutation stay separate trust boundaries. The
scheduler may still surface a proposed patch as the next step, but the action
queues or requires the apply-specific approval before any file write occurs.
The bridge removes renderer-side PatchDraft parsing/orchestration, but it is not
yet the full autonomous executor/repair loop.
