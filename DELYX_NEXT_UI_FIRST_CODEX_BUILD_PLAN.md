# Delyx Next No-BS Build Checklist

Last updated: 2026-06-08.

## Independent Verification

Audited against the local repo on 2026-06-07. Every marked-off Phase 1 item was
confirmed accurate; no checkbox was overclaimed. Evidence:

- `cargo fmt --check` and `cargo test --workspace`: 212 passed, 0 failed.
- `npm run typecheck`, `npm test` (smoke/source-contract plus focused Vitest component tests), `npm run build`, `npm run smoke:ui`, and `npm run smoke:tauri`: pass.
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
- SQLite is partially implemented. AgentRun save/load, Tauri thread/run session state, approval bridge state, recent workspace project snapshots, model role routes, memory governance, skill registry, automation engine, release/support-bundle state and file-export receipts, approved test artifacts, patch proposal/apply/restore receipts, review reports, external-agent run artifacts, research EvidenceStore receipts, AgentRun EvidenceRecords, and AgentOutcome support ID links now use a real SQLite database and migration. Memory, skills, automation contracts/scheduled runs, release/support-bundle state, support-bundle file export, patch apply/restore, and final-answer support synthesis have narrow persisted mutation bridges; remaining action bridges are still missing.
- There is no full execution engine: no scheduler, multi-node executor, repair loop, or hook runner. Narrow AgentRun executor nodes can now run approval-gated patch proposal/apply/restore, approved test-command work, and read-only review work while recording run events/artifacts/evidence.
- The default Explore -> Plan -> Approve -> Build -> Diff -> Test -> Review loop is not autonomous.
- Ollama is the only real live model execution path.
- OpenAI-compatible providers are health/config stubs only.
- Codex CLI has an approval-gated read-only launch bridge with captured terminal output and UI artifacts.
- Codex write-capable launch still needs real checkpoint/worktree isolation and diff depth before it is usable.
- Claude external agent support is detection/contract-preview only.
- Generic terminal worker execution exists behind external-agent and terminal-command approvals.
- Frontend coverage now includes a narrow Vitest/React Testing Library path plus older smoke/source-contract verifier scripts. Broad behavior coverage is still missing.
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
  - Automation contract create, approved activation, pause, and due-run scheduling now run through a Tauri bridge backed by the persisted AutomationEngine. Contract activation still requires a matching approved `ScheduledRiskyAction` proposal. Risky scheduled runs generate persisted, UI-visible approval cards through the approval bridge before execution.
  - Release profile and latest redacted support bundle now save to SQLite; a Tauri bridge feeds the cockpit.
  - Release profile save, release smoke capture, latest redacted support-bundle export, and approval-gated support-bundle file export now run through Tauri bridges backed by SQLite. Support-bundle config/log entries are redacted by the release domain before persistence; file export requires a matching approved `FileWrite` proposal and an approved root. Signing execution and update publishing are still not implemented.
  - Approved test artifact bridge records now save command, cwd, approval/run IDs, exit status, stdout/stderr, parsed failures, output truncation, and command exec events to SQLite and reload on desktop startup.
  - Patch proposal, apply, and restore bridge records now save proposal IDs, approval/run IDs, status, checkpoint IDs, restore approval IDs, file paths, before/after text, diff lines, and checkpoint file receipts to SQLite and reload on desktop startup. `patch_apply_approved` requires a matching approved `FileWrite` proposal, approved root, unchanged file contents since proposal, and then writes through the checkpointing PatchEngine. `patch_restore_approved` requires a matching approved `FileWrite` proposal, approved root, unchanged file contents since apply, and then restores/removes checkpointed files.
  - Review report bridge records now save report IDs, decisions, summaries, finding order, hunk labels, priorities, and suggested fixes to SQLite and reload on desktop startup.
  - External-agent run bridge records now save adapter IDs, status, scope summary, terminal output, diff summary, review-required state, ordered transcript events, and linked test artifact IDs to SQLite and reload on desktop startup.
  - Research EvidenceStore records now save run IDs, source kind, title, locator, excerpt, stance, normalized claim keys, and post-reload ID continuity to SQLite.
  - AgentRun EvidenceRecords now save source IDs, URIs, quotes, hashes, retrieval timestamps, and relevance metadata to SQLite; thread/run snapshots expose those receipts back to the UI instead of returning an empty evidence array.
  - AgentOutcome now saves linked evidence record IDs and test artifact IDs to SQLite; thread/run snapshots expose those support links to the UI.
  - Final-answer support synthesis now has a narrow Tauri bridge that links existing AgentRun EvidenceRecord IDs and passed persisted test artifact IDs into AgentOutcome, records a visible `final_answer.support_synthesized` event, marks the thread done, and saves the result to SQLite. It does not generate prose, infer unsupported claims, or make the full agent loop autonomous.
  - SQLite tests prove migration tables, foreign keys, child records, run reload, thread/run session reload, approval reload, workspace snapshot reload, model route reload, memory reload, memory mutation reload, skill reload, skill mutation reload, automation reload, automation contract/scheduled-run mutation reload, release reload, release mutation reload, release smoke reload, support-bundle reload, support-bundle file-export receipt reload, test artifact reload, patch proposal/apply/restore reload, review report reload, external-agent run reload, research evidence reload, AgentOutcome support reload, final-answer support synthesis reload, UI-ready bridge snapshots, legacy migration upgrades, and SQLite file format.
  - Persist remaining action bridges that still live only inside detached runtime state.
  - Next: add mutation/action bridges for the remaining persisted governance stores only when the matching approval gates and UI states are ready, then split remaining artifact/evidence stores only where AgentRun persistence is not enough.
  - Add migration/repository tests that prove data survives reload.

- [ ] D2 - AgentRun Execution Engine
  - Add executor, scheduler, node runner, resume, repair, and hook modules.
  - Make AgentRun the real execution graph, not only an inspector artifact.
  - Added a shared `CommandExecArtifact` primitive for approved command receipts; it now feeds the test runner and external terminal worker.
  - Added a narrow `agent_execute_patch_proposal` bridge that waits on pending `FileWrite` approvals, runs an approved patch-proposal node through AgentRun, persists the patch proposal, and records node events, artifact IDs, and diff evidence receipts.
  - Added a narrow `agent_execute_patch_apply` bridge that waits on pending `FileWrite` approvals, applies an existing PatchProposal through the stale-file/checkpoint PatchEngine path, writes only after approval, and records AgentRun node events, patch-apply artifacts, and diff evidence receipts.
  - Added a narrow `agent_execute_patch_restore` bridge that requires a separate executable `FileWrite` approval, restores/removes checkpointed files only when current contents still match the applied patch, and records AgentRun rollback events, artifacts, and evidence receipts.
  - Added a narrow `agent_execute_test_run` bridge that waits on pending `TerminalCommand` approvals, runs only commands accepted by the TestRunner, captures the persisted TestArtifact, and records AgentRun test-execution events, artifacts, and evidence receipts.
  - Added a narrow `agent_execute_review` bridge that reads persisted PatchProposal and TestArtifact records for the run, creates a read-only ReviewReport, and records AgentRun review events and report artifacts.
  - Model calls now emit visible `model_call.started` events so the UI can show real in-flight local model work without fake chain-of-thought.
  - Drive Explore -> Plan -> Approve -> Build -> Diff -> Test -> Review through runtime state.
  - Use Codex thread/start vs turn/start and command/exec protocol shapes as reference.
  - Keep all risky actions approval-gated.

- [ ] D3 - Behavioral Frontend Tests
  - Add Vitest + React Testing Library or Playwright interaction tests.
  - Added Vitest + React Testing Library as a real component-test path, separate from the existing smoke/source-contract verifiers.
  - First behavior test covers FocusThread patch apply visibility/click behavior: Apply appears only for proposed patches with matching approved approvals.
  - Added a MarkdownMessage component test proving headings, lists, bold, inline code, and fenced code render as elements instead of raw markdown text.
  - Added FocusShell behavior coverage for the home composer send path and keyboard-opened Settings desktop-shell state.
  - Added FocusThread behavior coverage for live run placement: latest user message, single running activity line, then assistant reply.
  - Added FocusThread empty-artifact coverage so plan/diff/test/review placeholder blocks stay hidden until real artifacts exist.
  - Cover project creation, thread creation, planning, approval, diff, test artifact, review, evidence, error, blocked, expired, and empty states.
  - Keep grep/source verifiers only as smoke guards.
  - Stop using source-substring checks as proof of UI behavior.

- [ ] D4 - UI Architecture Decision
  - Default workbench migrated to focused React components using the provided Focus prototype as visual source.
  - Legacy string-rendered cockpit files are formally deprecated as mounted UI and retained only as older smoke-contract/reference code until they can be safely deleted.
  - `docs/ARCHITECTURE.md` records FocusShell as the live workbench and legacy cockpit as non-mounted reference code.
  - Add behavior tests around the Focus component tree and direct action callbacks.
  - Reconcile the plan with Radix/TanStack/Zustand targets.
  - Completed now: no-thread cockpit hides unbacked progress, diff, terminal, inspector, and metric-card furniture; Focus home centers the real composer and keeps setup nudges tied to real repo/model state.

- [ ] D5 - Functional Build Flow
  - Convert approved plans into patch proposals through the runtime engine.
  - Runtime can now convert an explicit approved patch-proposal request into a persisted AgentRun patch node; it is not yet wired from the UI plan/build flow and does not generate patch content by itself.
  - Runtime can now execute an explicit approved PatchProposal apply node through AgentRun. This is real file I/O through the existing stale-file and checkpoint gates, but it is not yet automatically chained from plan approval.
  - Runtime can now execute an explicit approved PatchProposal restore node through AgentRun. This is real rollback I/O through the existing restore approval, stale-after, and checkpoint receipt gates, but it is not yet automatically chained from review rejection.
  - Focus UI now loads persisted patch snapshots for the active run instead of passing a static empty patch array, so real PatchProposal diffs can appear when the runtime creates them.
  - Focus approval decisions no longer mark the thread as `building` by themselves; approval returns to the next-step state or keeps waiting when more approvals are pending until an actual executor/tool action starts.
  - Focus state now loads persisted approval proposals/decisions for the active run instead of relying only on the current renderer session, which is required before safe patch/test action buttons can reason about approval status.
  - Focus diff UI can now call the AgentRun patch-apply bridge for a proposed patch only when its matching approval is visibly approved; Rust still enforces approval, approved root, stale-file, and checkpoint gates before any write.
  - Patch apply and restore now have persisted approval-gated bridges with stale-file protection and checkpoint receipts; the runtime engine still needs to call them automatically from the build flow.
  - Evaluate Codex `apply-patch` parser/delta model before deepening the local patch engine.
  - Surface real diffs and rollback state in the UI.
  - Connect build outputs to test and review steps.

- [ ] D6 - Functional Test/Review Loop
  - Run approved tests from the agent loop.
  - Runtime can now execute an explicit approved test command through AgentRun. It reuses the existing TestRunner approval, cwd, command-shape, timeout, output-capture, and artifact persistence gates, but it is not yet automatically chained from patch apply.
  - Focus thread UI now loads persisted test artifacts for the active run instead of passing a static empty test array, so real TestRunner receipts can appear when the runtime creates them.
  - Focus thread UI can now show a manual `Run tests` action after an applied patch when the active plan contains a supported direct test command. If terminal approval is missing, it queues a visible approval first; execution uses the AgentRun test bridge only after approval.
  - Plan command discovery now prefers the bundled `.tools\npm.cmd test` wrapper when the project index proves it exists, avoiding fake PATH assumptions on Windows.
  - Generate review reports from actual patch/test artifacts.
  - Runtime can now execute an explicit read-only review node through AgentRun. The bridge gathers persisted patch and test artifacts by run ID before creating the ReviewReport, so review input is actual stored receipt data rather than caller-supplied mock state.
  - Focus thread UI can now run that read-only review action when the active run has real patch or test artifacts, reload persisted ReviewReports, and display the resulting review receipt inline.
  - Prevent final "tested" claims unless linked artifacts exist.

- [ ] D7 - Model Integration Depth
  - OpenAI-compatible providers are out of live scope for now. The frontend maps the typed backend stub to an unavailable/not-wired UI state instead of suggesting a missing API key would make it usable.
  - Revisit only with real calls, keyring-backed secret handling, health checks, and tests.
  - Runtime status now optionally probes real local Ollama `/api/version` and surfaces the version in Settings when available; missing version data does not override the model-readiness probe.
  - Add Ollama version/readiness and optional pull-progress UI only when backed by real local state.

- [ ] D8 - External Agent Integration Depth
  - Codex CLI read-only launch is wired behind external-agent and terminal approvals with captured terminal output and UI artifacts.
  - External-agent run receipts now survive restart through SQLite, including transcript and linked test IDs.
  - Add real checkpoint/worktree creation before enabling write-capable Codex launch from the UI.
  - Add real changed-file/diff capture from external-agent runs instead of review placeholders.
  - Decide whether Claude launch is in scope beyond detection and contract preview.
  - If no, keep them detection/contract-preview only and label them that way in UI.

- [ ] D9 - Evidence and Final Answer Receipts
  - Added a narrow final-answer support synthesis bridge for existing AgentRun evidence and passed persisted test artifacts.
  - Focus thread UI now shows final support receipt counts when AgentOutcome exists.
  - Focus can record final support from an existing assistant message only; it links existing AgentRun evidence and passed persisted tests through the Tauri bridge and does not generate new prose or infer claims.
  - Build final-answer support records from files read, commands run, tests executed, diffs produced, model calls, approvals, and evidence.
  - Make unsupported, insufficient, partial, and untested final-answer states visible.

- [x] D10 - Architecture Reconciliation
  - Recorded the current single-crate Rust decision in `docs/ARCHITECTURE.md`.
  - Split crates only when real pressure justifies it.
  - Keep target architecture as an extraction map, not a fake repo shape.

- [ ] D11 - Codex Reference Salvage Track
  - Use `docs/CODEX_REFERENCE_AUDIT.md` as the pick list.
  - Pull only pieces that reduce risk or save real implementation time.
  - Candidate direct/adapt pieces: exec policy decisions, command exec artifacts, thread/turn protocol shape, apply-patch deltas, keyring store, Ollama readiness, Git baseline/diff helpers, sandbox capability detection.
  - Pulled/adapted: PowerShell UTF-8 command prep, read-only Codex CLI launch contract, and typed command execution receipts.
  - Avoid importing Codex core, generated protocol macros, cloud auth, or broad parser stacks until a PR proves the need.
  - Every Codex-derived change needs tests, UI-visible state, approval gates, and dependency justification.

- [ ] D12 - Refined Windows Desktop App
  - Current truth: Delyx Next has a usable Tauri Windows desktop shell and NSIS package path, but it is still an unsigned dev product without updater/signing polish.
  - Added explicit `dev:desktop` scripts for the Tauri Windows shell; `dev` remains the browser/Vite preview.
  - Tauri config now declares the stable main window label, centered native decorated window behavior, bundle publisher/descriptions, and app/installer icon paths.
  - Release smoke now checks desktop launch script wiring, primary window basics, bundle metadata, and NSIS icon configuration.
  - Added the official Tauri single-instance plugin so launching Delyx Next again focuses the existing main window instead of creating a second desktop session.
  - Runtime status now exposes desktop shell policy to the UI: main window label, renderer-command menu policy, startup focus behavior, single-instance reopen behavior, and unsigned dev signing status.
  - Settings now shows the real Windows shell state when the Rust bridge is available.
  - Packaged Windows verification passed on current head: `.\.tools\npm.cmd run package:windows` produced `target\release\bundle\nsis\Delyx Next_0.0.0_x64-setup.exe`.
  - Next desktop depth: signing, updater policy, install/upgrade smoke, native file associations/deep links only if the product needs them.
  - Keep the desktop shell tied to real local runtime state; do not use packaging polish to hide missing agent behavior.

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

Current warning: `npm test` now includes focused real component behavior tests, but broad frontend behavior coverage is still missing until D3 is expanded.

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
