# Delyx Next No-BS Build Checklist

Last updated: 2026-06-08.

## Independent Verification

Audited against the local repo on 2026-06-07. Every marked-off Phase 1 item was
confirmed accurate; no checkbox was overclaimed. Evidence:

- `cargo fmt --check` and `cargo test --workspace`: 212 passed, 0 failed.
- `npm run typecheck`, `npm test` (smoke/source-contract), `npm run build`, `npm run smoke:ui`, and `npm run smoke:tauri`: pass.
- Browser visual checks passed for the no-thread cockpit at 1280x720 and 390x844 before the Focus port: no fake progress/diff/terminal/metric blocks, no inspector, no horizontal overflow. Focus UI browser checks must stay current after each visual pass.
- Partial SQLite state, missing execution engine, Ollama-only live model path,
  OpenAI-compatible stub, single Rust crate, string-rendered cockpit, and
  grep-style frontend verifiers all match the "Current Reality" claims below.
- Functional islands confirmed to do real I/O: `apply_approved_patch` performs
  approval-gated `fs::write` + checkpoint, and the generic terminal worker does a
  real `Command::spawn` with timeout and output capture.

## Core Product

- Delyx Next is a local-first, UI-first AI workbench.
- Default workflow: Project -> Thread -> Explore -> Plan -> Approve -> Build -> Diff -> Test -> Review.
- The UI is the trust layer, not decoration.
- No fake runtime data in the shipped UI.
- Every runtime concept needs a visible state: empty, loading, waiting, blocked, failed, denied, expired, partial, success.
- Risky actions require approval before execution.
- Code-change claims need diff artifacts.
- Tested-code claims need execution artifacts.
- Source-backed claims need EvidenceRecords.
- Local-first is the default; cloud integrations are optional and explicit.
- Source files target 300 lines or fewer; split before 400; 500 is a hard cap unless generated/config/documented.

## Completion Meaning

- Skeleton complete means typed model or policy shape + bridge/client slice + honest empty UI + deterministic slice checks.
- Functionally complete means it drives real work end-to-end, persists in the intended store, and has behavior tests that prove the user workflow.
- Most completed PRs below are skeleton complete, not functionally complete.

## Current Reality

- PR 1-18.1 breadth is skeleton-complete.
- SQLite is partially implemented. AgentRun save/load, Tauri thread/run session state, approval bridge state, recent workspace project snapshots, model role routes, memory governance, skill registry, automation engine, release/support-bundle state, approved test artifacts, patch proposal diffs, review reports, external-agent run artifacts, research EvidenceStore receipts, AgentRun EvidenceRecords, and AgentOutcome support ID links now use a real SQLite database and migration. Memory, skills, automation contracts, and release/support-bundle state have narrow persisted mutation bridges; automatic final-answer support synthesis plus remaining action bridges are still missing.
- There is no full execution engine: no scheduler, executor, resume engine, repair loop, or hook runner.
- The default Explore -> Plan -> Approve -> Build -> Diff -> Test -> Review loop is not autonomous.
- Ollama is the only real live model execution path.
- OpenAI-compatible providers are health/config stubs only.
- Codex CLI has an approval-gated read-only launch bridge with captured terminal output and UI artifacts.
- Codex write-capable launch still needs real checkpoint/worktree isolation and diff depth before it is usable.
- Claude external agent support is detection/contract-preview only.
- Generic terminal worker execution exists behind external-agent and terminal-command approvals.
- Frontend tests are smoke/source-contract verifier scripts, not behavioral React/component tests.
- The default workbench is now a React Focus shell ported from the provided Focus prototype. Legacy `buildCockpitMarkup` string-rendered cockpit files remain in source as an older implementation and smoke-contract reference, but they are no longer the mounted primary workbench.
- The repo intentionally has one Rust crate today: `apps/desktop/src-tauri`.
- `openai/codex` was audited at commit `e093d81` as a reference/salvage pool, not a repo to blindly copy.
- A small Codex-inspired PowerShell UTF-8 terminal-capture polish is wired for approved external worker commands.
- A Codex-inspired typed command execution artifact now backs approved test commands and external terminal workers with output caps, stdout/stderr events, status, duration, approval IDs, and deterministic tests.
- Codex CLI launch is now wired only through Delyx approvals and captured command artifacts; it is not an autonomous build loop.
- The Focus workbench now uses a centered first-run composer, rail navigation, command palette, thread switcher, model picker, settings surface, and artifact-driven active thread view. The UI renders real project/thread/model/run/approval/diff/test state only; no prototype thread/model/diff mock content ships.

## Phase 1 Skeleton Checklist

- [x] PR 1 - Product Direction and Planning Docs: skeleton complete.
- [x] PR 2 - App Shell + Design System + Mock UI Prototype: skeleton complete, later stripped of shipped fake data.
- [x] PR 3 - Workspace Manager Wired to UI: skeleton complete.
- [x] PR 3.1 - Tauri Workspace Snapshot MVP: skeleton complete.
- [x] PR 3.2 - Remove Simulated Workspace State Controls: complete.
- [x] PR 3.3 - Read-Only Git Dirty Count: skeleton complete.
- [x] PR 4 - Thread Manager Wired to UI: skeleton complete.
- [x] PR 4.5 - Remove Demo Data From UI: complete.
- [x] PR 4.6 - Remove Simulated Thread Empty-State Controls: complete.
- [x] PR 4.7 - Remove Legacy Decorative Thread Controls: complete.
- [x] PR 5 - Typed AgentRun Ledger: skeleton complete; AgentRun file persistence now uses SQLite, broader local store still missing.
- [x] PR 5.1 - Tauri Thread/Run Session Bridge: skeleton complete.
- [x] PR 5.2 - Thread Session Status and Archive Bridge: skeleton complete.
- [x] PR 6 - Explore and Plan Modes: skeleton complete.
- [x] PR 7 - Approval Engine: narrow functional island.
- [x] PR 7.1 - Tauri Approval Session Bridge: narrow functional island.
- [x] PR 8 - Patch Proposal and Checkpoints: narrow functional island.
- [x] PR 8.1 - Remove Unimplemented Approval and Diff Controls: complete.
- [x] PR 8.2 - Tauri Patch Proposal Bridge: skeleton complete; build-flow wiring still missing.
- [x] PR 9 - Test Runner Artifacts: narrow functional island.
- [x] PR 9.1 - Tauri Test Runner Bridge: narrow functional island for approved test commands.
- [x] PR 10 - Review Mode: skeleton complete.
- [x] PR 10.1 - Tauri Review Report Bridge: skeleton complete.
- [x] PR 11 - Model Provider Abstraction: skeleton complete.
- [x] PR 11.1 - Ollama Composer MVP: functional for local Ollama composer replies.
- [x] PR 11.2 - Tauri Runtime Bridge MVP: skeleton complete.
- [x] PR 11.3 - Ollama PlanAgent MVP: narrow functional island for read-only plan drafts.
- [x] PR 11.4 - Remove Frontend Mock Model Route: complete.
- [x] PR 11.5 - Remove Runtime Mock Route: complete.
- [x] PR 11.6 - Tauri Ollama Runtime Detection: functional for local detection.
- [x] PR 11.7 - Renderer Runtime Model Sync: skeleton complete.
- [x] PR 11.8 - Tauri Ollama Chat Bridge: functional for local Ollama chat.
- [x] PR 11.9 - Remove Frontend Mock Provider Kind: complete.
- [x] PR 11.10 - Ollama Agent Session Bridge: skeleton complete.
- [x] PR 12 - External Agent Bridge Prototype: skeleton complete; generic terminal worker is the only executing adapter.
- [x] PR 12.1 - Truthful External Agent Detection: skeleton complete.
- [x] PR 12.2 - External Agent Command Contracts: skeleton complete.
- [x] PR 12.3 - External Agent Contract UI State: skeleton complete.
- [x] PR 12.4 - External Agent Contract Preview Command: skeleton complete.
- [x] PR 12.5 - External Agent Command Array Rendering: complete.
- [x] PR 12.6 - External Agent Adapter Status Bridge: skeleton complete.
- [x] PR 13 - Source-Backed Research MVP: skeleton complete.
- [x] PR 13.1 - Active Run Evidence Inspector Wiring: skeleton complete.
- [x] PR 14 - Memory Governance: skeleton complete.
- [x] PR 14.1 - Active Run Memory Inspector Wiring: skeleton complete.
- [x] PR 15 - Skills: skeleton complete.
- [x] PR 15.1 - Imported Skills Inspector Wiring: skeleton complete.
- [x] PR 16 - Automations / Mission Contracts: skeleton complete.
- [x] PR 16.1 - Automation Inspector Wiring: skeleton complete.
- [x] PR 17 - Mobile Companion: skeleton complete.
- [x] PR 17.1 - Mobile Companion Inspector Wiring: skeleton complete.
- [x] PR 18 - Packaging and Release: skeleton complete.
- [x] PR 18.1 - Release Readiness Inspector Wiring: skeleton complete.
- [x] PR 23.1 - Approval Taxonomy Snapshot Bridge: skeleton complete.

## Phase 2 Functional Depth Checklist

- [ ] D1 - Real SQLite Local Store
  - Added `rusqlite` with bundled SQLite for local Windows-safe storage.
  - AgentRun `save_to_path` / `load_from_path` now use the SQLite migration instead of a tab-separated text helper.
  - Tauri thread/run bridge state now saves threads, messages, run links, and AgentRun rows to SQLite and reloads them on desktop startup.
  - Tauri approval bridge state now saves proposals, scope, status, decisions, and decision notes to SQLite and reloads them on desktop startup.
  - Recent workspace project snapshots now save project metadata, rules files, approved roots, Git metadata, and indexed file names to SQLite.
  - Model role routes now save to SQLite; runtime status reloads a valid saved coding route before falling back to the first ready Ollama model.
  - The Rust memory governance store now saves candidates, promoted records, suppression state, and ID continuity to SQLite; a read-only Tauri snapshot bridge feeds the cockpit.
  - Memory candidate propose, approved promotion, candidate suppression, and record suppression now run through a Tauri bridge backed by the persisted MemoryStore. Durable memory promotion still requires a matching approved `DurableMemorySave` proposal and a completed source run.
  - The Rust skill registry now saves imported skills, trust, activation status, permissions, source hashes, and ID continuity to SQLite; a read-only Tauri snapshot bridge feeds the cockpit.
  - Skill import, activation with explicit permissions, disable, and suppression now run through a Tauri bridge backed by the persisted SkillRegistry. Activation changes local registry state only; it does not execute scripts, edit files, or use network by itself.
  - The Rust automation engine now saves mission contracts, active/paused status, schedules, allowed tools, scheduled run records, approval links, and ID continuity to SQLite; a read-only Tauri snapshot bridge feeds the cockpit.
  - Automation contract create, approved activation, and pause now run through a Tauri bridge backed by the persisted AutomationEngine. Contract activation still requires a matching approved `ScheduledRiskyAction` proposal; scheduled-run creation remains internal until generated approval cards are persisted and UI-visible through the approval bridge.
  - Release profile and latest redacted support bundle now save to SQLite; a Tauri bridge feeds the cockpit.
  - Release profile save and latest support-bundle export now run through a Tauri bridge backed by SQLite. Support-bundle config/log entries are redacted by the release domain before persistence; signing execution, update publishing, and support-bundle file export are still not implemented.
  - Approved test artifact bridge records now save command, cwd, approval/run IDs, exit status, stdout/stderr, parsed failures, output truncation, and command exec events to SQLite and reload on desktop startup.
  - Patch proposal bridge records now save proposal IDs, approval/run IDs, status, checkpoint ID, file paths, and diff lines to SQLite and reload on desktop startup.
  - Review report bridge records now save report IDs, decisions, summaries, finding order, hunk labels, priorities, and suggested fixes to SQLite and reload on desktop startup.
  - External-agent run bridge records now save adapter IDs, status, scope summary, terminal output, diff summary, review-required state, ordered transcript events, and linked test artifact IDs to SQLite and reload on desktop startup.
  - Research EvidenceStore records now save run IDs, source kind, title, locator, excerpt, stance, normalized claim keys, and post-reload ID continuity to SQLite.
  - AgentRun EvidenceRecords now save source IDs, URIs, quotes, hashes, retrieval timestamps, and relevance metadata to SQLite; thread/run snapshots expose those receipts back to the UI instead of returning an empty evidence array.
  - AgentOutcome now saves linked evidence record IDs and test artifact IDs to SQLite; thread/run snapshots expose those support links to the UI.
  - SQLite tests prove migration tables, foreign keys, child records, run reload, thread/run session reload, approval reload, workspace snapshot reload, model route reload, memory reload, memory mutation reload, skill reload, skill mutation reload, automation reload, automation contract mutation reload, release reload, release mutation reload, support-bundle reload, test artifact reload, patch proposal reload, review report reload, external-agent run reload, research evidence reload, AgentOutcome support reload, UI-ready bridge snapshots, and SQLite file format.
  - Persist remaining automatic final-answer support records, scheduled-run approval card wiring, and action bridges that still live only inside detached runtime state.
  - Next: add mutation/action bridges for the remaining persisted governance stores only when the matching approval gates and UI states are ready, then split remaining artifact/evidence stores only where AgentRun persistence is not enough.
  - Add migration/repository tests that prove data survives reload.

- [ ] D2 - AgentRun Execution Engine
  - Add executor, scheduler, node runner, resume, repair, and hook modules.
  - Make AgentRun the real execution graph, not only an inspector artifact.
  - Added a shared `CommandExecArtifact` primitive for approved command receipts; it now feeds the test runner and external terminal worker.
  - Model calls now emit visible `model_call.started` events so the UI can show real in-flight local model work without fake chain-of-thought.
  - Drive Explore -> Plan -> Approve -> Build -> Diff -> Test -> Review through runtime state.
  - Use Codex thread/start vs turn/start and command/exec protocol shapes as reference.
  - Keep all risky actions approval-gated.

- [ ] D3 - Behavioral Frontend Tests
  - Add Vitest + React Testing Library or Playwright interaction tests.
  - Cover project creation, thread creation, planning, approval, diff, test artifact, review, evidence, error, blocked, expired, and empty states.
  - Keep grep/source verifiers only as smoke guards.
  - Stop using source-substring checks as proof of UI behavior.

- [ ] D4 - UI Architecture Decision
  - Default workbench migrated to focused React components using the provided Focus prototype as visual source.
  - Legacy string-rendered cockpit files still need cleanup or formal deprecation.
  - Add behavior tests around the Focus component tree and direct action callbacks.
  - Reconcile the plan with Radix/TanStack/Zustand targets.
  - Completed now: no-thread cockpit hides unbacked progress, diff, terminal, inspector, and metric-card furniture; Focus home centers the real composer and keeps setup nudges tied to real repo/model state.

- [ ] D5 - Functional Build Flow
  - Convert approved plans into patch proposals through the runtime engine.
  - Apply approved patches only after approval and checkpointing.
  - Evaluate Codex `apply-patch` parser/delta model before deepening the local patch engine.
  - Surface real diffs and rollback state in the UI.
  - Connect build outputs to test and review steps.

- [ ] D6 - Functional Test/Review Loop
  - Run approved tests from the agent loop.
  - Attach test artifacts to the active run automatically.
  - Generate review reports from actual patch/test artifacts.
  - Prevent final "tested" claims unless linked artifacts exist.

- [ ] D7 - Model Integration Depth
  - Decide whether OpenAI-compatible providers are in scope.
  - If yes, implement real calls, keyring-backed secret handling, health checks, and tests.
  - If no, mark them out of scope and remove misleading provider surfaces.
  - Add Ollama version/readiness and optional pull-progress UI only when backed by real local state.

- [ ] D8 - External Agent Integration Depth
  - Codex CLI read-only launch is wired behind external-agent and terminal approvals with captured terminal output and UI artifacts.
  - External-agent run receipts now survive restart through SQLite, including transcript and linked test IDs.
  - Add real checkpoint/worktree creation before enabling write-capable Codex launch from the UI.
  - Add real changed-file/diff capture from external-agent runs instead of review placeholders.
  - Decide whether Claude launch is in scope beyond detection and contract preview.
  - If no, keep them detection/contract-preview only and label them that way in UI.

- [ ] D9 - Evidence and Final Answer Receipts
  - Build final-answer support records from files read, commands run, tests executed, diffs produced, model calls, approvals, and evidence.
  - Make unsupported, insufficient, partial, and untested final-answer states visible.

- [ ] D10 - Architecture Reconciliation
  - Record the current single-crate Rust decision in `docs/ARCHITECTURE.md`.
  - Split crates only when real pressure justifies it.
  - Keep target architecture as an extraction map, not a fake repo shape.

- [ ] D11 - Codex Reference Salvage Track
  - Use `docs/CODEX_REFERENCE_AUDIT.md` as the pick list.
  - Pull only pieces that reduce risk or save real implementation time.
  - Candidate direct/adapt pieces: exec policy decisions, command exec artifacts, thread/turn protocol shape, apply-patch deltas, keyring store, Ollama readiness, Git baseline/diff helpers, sandbox capability detection.
  - Pulled/adapted: PowerShell UTF-8 command prep, read-only Codex CLI launch contract, and typed command execution receipts.
  - Avoid importing Codex core, generated protocol macros, cloud auth, or broad parser stacks until a PR proves the need.
  - Every Codex-derived change needs tests, UI-visible state, approval gates, and dependency justification.

## Validation Gates

Run relevant gates after each PR:

```powershell
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test
cargo test --workspace
.\.tools\npm.cmd run build
.\.tools\npm.cmd run smoke:ui
.\.tools\npm.cmd run smoke:tauri
git diff --check
```

Current warning: `npm test` is not a real frontend behavior suite yet. It is a smoke/source-contract gate until D3 lands.

For eval work:

```powershell
.\.tools\npm.cmd run eval:response
.\.tools\npm.cmd run eval:agentic
```

## Definition of Done

- Code compiles.
- Tests prove the behavior at the right level.
- UI states are truthful.
- Docs match current reality.
- Risky actions remain approval-gated.
- No fake data is shipped to make the app look finished.
- No source-backed, tested, or completed claims are made without artifacts.
- Source files stay inside the line budget.
