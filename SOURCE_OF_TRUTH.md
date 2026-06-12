# Delyx Next — SOURCE OF TRUTH

Last updated: 2026-06-11.

This is the **single canonical document** for Delyx Next. It replaces and
supersedes the retired planning docs (`ROADMAP.md`,
`DELYX_NEXT_CAMPAIGN_MODE_ARCHITECTURE.md`, and everything that lived under
`docs/` — all recoverable from git history). If this file and any other prose
disagree, this file wins.

The only other markdown that matters:

- `AGENTS.md` — runtime rules file. It is **read by the agent at runtime**
  (codebase-awareness injects AGENTS.md/CLAUDE.md into the local model's
  system prompt) and by external Claude/Codex CLI workers. Keep it short,
  imperative, and current.
- `README.md` — quickstart: build, run, validate, package.
- `apps/desktop/src-tauri/packs/*/lore.md` — Campaign Mode era content
  (runtime data, not documentation).

---

## 1. Mission

Delyx Next is a **local-first, UI-first AI workbench** — a desktop coding
agent plus a war-RPG campaign engine, both running against local models.

```text
Delyx Next =
  local-first safety, approvals, evidence receipts, model routing
  + Codex-style project/thread/diff workflow
  + Claude-Code-style Explore/Plan/Build/Test/Review agent flow
  + a first-class desktop UI as the trust layer
```

**North star: be as good as — or better than — Claude Code and Codex CLI as a
coding agent**, judged purely as a harness (model quality and datacenters
excluded). We cannot out-spend Anthropic/OpenAI on inference, so we win on
**engineering**: tighter loops, deterministic local control, binding safety,
verifiable receipts, and local-runtime tricks cloud harnesses structurally
cannot do (see §12).

The bar is not parity — it is **surprisingly good**: a user coming from
Claude Code or Codex should hit at least one moment per session where Delyx
does something neither CLI can do (a pre-verified diff, a caught injection,
a receipt chain, an instant cached local turn, a self-revoked permission).
§10.1 is the moat we defend; §12 is the surprise backlog we ship from.

The product promise, in user terms — the user can always answer:

- What is Delyx doing right now?
- What files did it inspect?
- What is it asking permission to do, with what scope and rollback plan?
- What did it change? (diff, checkpoint, restore path)
- Did tests actually run? (artifact or it didn't happen)
- What evidence supports the final answer?
- What is blocked, failed, denied, expired, partial, or uncertain?

## 2. Hard rules (non-negotiable)

1. UI-first: every runtime concept has a visible user-facing state. No
   backend-first agent shell with a thin window.
2. **No execution without approval**: file writes, terminal commands,
   connector actions, durable memory saves, scheduled risky actions, and
   external agent runs all require an approval record linked to the active
   run. This is enforced in Rust at the execution bridge, not in the renderer.
3. No tested claims without an execution artifact. No source-backed claims
   without EvidenceRecords. No fake certainty, ever.
4. Failed, blocked, denied, expired, partial, and uncertain states are shown,
   never collapsed into a generic state.
5. Do not weaken validators or tests to make a feature look complete.
6. Local-first by default. Cloud is explicit, opt-in, and visibly bounded.
7. Secrets never in the repo, settings.json, or SQLite — OS keyring only
   (`secret_store` / `secret_bridge`).
8. File-size budget: target ≤300 lines, split at ~400, hard cap 500 unless
   generated/declarative/documented. `npm test` enforces 300 for
   `.rs`/`.ts`/`.tsx`/`.css`.
9. External agents never get broader authority than the current Delyx task.
10. Prefer small PR-sized changes; deterministic fixtures before live-model
    testing; record architectural decisions in this file (§13).

**Definition of done:** code compiles; tests prove the behavior at the right
level; UI states are truthful; this doc matches reality; risky actions stay
approval-gated; no completed/tested/source-backed claims without artifacts;
files stay inside the line budget.

## 3. Verified current state

Gates verified 2026-06-11 on this machine:
`cargo test --no-default-features --lib` = **471 / 0** (clean rebuild),
`npm test` = **147 / 0** Vitest plus all four deterministic verifiers
(workbench/file-size budget, markdown render, ollama agent, release smoke).

Build-environment note: cargo builds run from a sandboxed shell fail with
`os error 4551` ("An Application Control policy has blocked this file") —
freshly compiled, unsigned build scripts are blocked from executing, which
leaves the target cache half-built and produces misleading `E0463 can't find
crate` errors on later runs. Build from a normal (non-sandboxed) shell; if a
cache is poisoned, `cargo clean` and rebuild.

Working end-to-end today:

- **Embedded local runtime** (mistral.rs 0.8.1, GGUF import, Ollama-blob
  reuse, CPU default, CUDA feature build) + Ollama adapter + Claude/Codex CLI
  chat adapters.
- **Agentic tool loop v1** for the local model: bounded Rust-owned loop,
  read-only tools (`read_file` / `list_dir` / `grep`), JSON protocol that
  works with any GGUF, live 🔧 tool narration, streamed final answer, cancel,
  tool receipt in the thread.
- **Codebase awareness**: system prompt carries project identity + branch, a
  capped repo map, and the project's rules files (AGENTS.md/CLAUDE.md), read
  through the scope-enforced bridge and cached per project.
- **Streaming + interrupt** for Delyx Local: token-by-token draft bubble,
  Stop cancels and keeps the partial; Ollama/CLI fall back to non-streaming.
- **CLI-as-executor**: composer "Agent worker" mode → two approval cards →
  launch → read-only or `[files: …]`-scoped write run → diff capture →
  unplanned-edit cross-check (blocks the thread if the CLI touched anything
  outside plan) → write-run edits promoted into an applied, checkpointed
  Delyx patch with the normal diff-review and approval-gated restore path.
- **QA/QC reviewer**: default-on, cheap second model via CLI
  (subscription auth, `--safe-mode`), fix-and-verify loop, **async and
  append-only — it never blocks or gates the answer**.
- **Native projects** (trust/scopes/policy) + **attachments pipeline**
  (propose → classify → approve → parse → budgeted context pack → evidence,
  with chips and context-pack UI).
- **Approval-gated execution islands**: patch propose/apply/restore with
  checkpoints, approved tests, read-only review, repair requests,
  final-answer support synthesis.
- **Bounded Rust driver** (`agent_drive_run`) over scheduler decisions.
- **SQLite persistence throughout**; Focus desktop shell; unsigned NSIS
  Windows installer.
- **Campaign Mode C1–C8**: full war-RPG engine (see §5.11) with seven era
  packs plus drop-in user-folder packs (no rebuild).

Not yet real (the honest list): the full autonomous
Explore→Plan→Build→Test→Review loop driven end-to-end by the Rust driver
(PatchDraft/model generation, approval-proposal creation, repair queueing,
and hooks are still renderer-triggered boundaries); write-capable tools
inside the model tool loop (`propose_edit` / `run_command`); context
compaction for code threads; diff-based (non-whole-file) patches; Ollama
streaming; signing/updater.

## 4. Stack & repo layout

| Layer | Choice | Notes |
|---|---|---|
| Shell | Tauri v2 | single-instance plugin; renderer command UI, no native menus |
| Frontend | React + TypeScript + Vite | no Radix/TanStack/Zustand until a real problem demands them (ADR-0007) |
| Backend | Rust, single crate | `apps/desktop/src-tauri` (ADR-0008); ~283 files, ~44K lines |
| Persistence | SQLite (rusqlite) | every store persists + reloads with ID continuity |
| Local model | `mistralrs` 0.8.1 | optional `embedded_mistral` feature (on by default in npm scripts); CUDA via `embedded_mistral_cuda` |
| Secrets | `keyring` | OS credential store only |
| Styling | CSS variables + small design system | Lucide icons; no Tailwind (ADR-0003) |
| Node/npm | bundled under `.tools\` | there is no global npm; use `.\.tools\npm.cmd` |

```text
Delyx Next/
  AGENTS.md                      runtime rules (read by agents)
  README.md                      quickstart
  SOURCE_OF_TRUTH.md             this file
  Cargo.toml / Cargo.lock        Rust workspace root
  package.json                   npm workspace root (scripts proxy to apps/desktop)
  dev-cuda.cmd                   MSVC+CUDA dev wrapper (vswhere → vcvars64 → nvcc check)
  scripts/                       deterministic verifiers (file-size budget, smoke, evals)
  apps/desktop/
    src/                         React frontend (~208 files, ~19K lines)
      app/                       FocusShell, AppShell, composer bindings, worker actions
      features/                  runs, models, patches, approvals, tests, externalAgents,
                                 campaigns, projects, attachments, evidence, memory
      design-system/             Button, Dialog, Drawer, StatusPill, EmptyState, …
    src-tauri/
      src/                       Rust backend (one module ≈ one concern + paired *_tests.rs)
      packs/                     shipped Campaign era packs (ww1, ww2, civil-war, korea,
                                 vietnam, revolutionary-war, star-wars)
  apps/desktop/evals/            deterministic fixtures (eval:response, eval:agentic)
```

## 5. Runtime architecture

Target runtime shape (domains, not crates — extraction happens only under
pressure, ADR-0008):

```text
Workspace Manager → Thread Manager → Agent Runtime → Permission Engine
→ Tool Layer → Model Layer → Evidence Layer → UI
```

### 5.1 Agent runtime

Four cooperating Rust modules own the loop:

**`agent_tool_loop.rs` — the model-facing tool loop.**
- `MAX_TOOL_STEPS = 6`. Protocol: the model emits either a JSON tool call
  (`{"tool":"read_file","path":"…"}`) or prose (= final answer). The loop
  peeks the first non-whitespace char of the stream: `{` or ``` ``` ``` →
  silent tool turn; anything else → stream to the UI as the answer.
- Tools are **read-only, root-scoped, capped, auto-allowed**:
  `read_file` (line-capped), `list_dir`, `grep`
  (`MAX_GREP_FILES = 600`, `MAX_GREP_FILE_BYTES = 256 KB`, tool result
  truncated at **8 KB** — `agent_tools.rs`).
- Tool errors are formatted as readable text and fed back so the model can
  retry. Cancellation is checked after every streamed chunk via
  `state.take_cancel(&request_id)`.
- **Injection firewall** (`injection_screen.rs`, shipped 2026-06-11): every
  tool result is treated as untrusted data — wrapped in
  `<<<DELYX-UNTRUSTED-DATA-…>>>` markers (embedded close-markers neutralized)
  before being fed back, and screened with precision-first patterns
  (instruction_override / role_hijack / protocol_mimicry). Hits emit a
  `tool_warning` event (⚠ line in the live draft + a security receipt
  message in the thread) and append a SECURITY NOTE telling the model the
  flagged lines are data. Detection warns and hardens; it never blocks.
- Works with any GGUF — no grammar/function-calling support is assumed.
- **Constrained repair** (shipped 2026-06-11): a turn that is tool-shaped
  but malformed gets one retry with the sampler grammar-locked to the
  tool-call JSON schema (`tool_call_json_schema()` in `agent_tools.rs`,
  `Constraint::JsonSchema`/llguidance in mistral.rs), so the repaired call
  is valid by construction and narrated as "(repaired)". Repair failures
  fall back to the old treat-as-answer behavior; they never kill the loop.

**`agent_scheduler.rs` — pure decision logic.** Reads the persisted AgentRun,
approval, patch, test, and review stores and returns a typed decision:
`run_patch_draft` / `request_patch_apply_approval` / `run_patch_apply` /
`run_tests` / `run_review` / `wait_for_approval` / `resume_after_approval` /
`ready_for_final_support` / `terminal` / `blocked`. Authority is always
**derived in Rust from persisted records** — the renderer never passes
approval IDs, commands, roots, or file-write authority as hints. Patch-apply
readiness requires the exact node-scoped approval; generic same-run FileWrite
approvals do not qualify. Test readiness requires an exact executable
same-run TerminalCommand approval whose scope includes the persisted command,
with shell-control text rejected by a shared parser.

**`agent_drive.rs` — the bounded driver.** `MAX_DRIVE_STEPS = 24`. Loop:
ask scheduler → execute the decision (approved patch apply, approved tests,
read-only review, final-support synthesis, single-approval resume) → persist
everything → repeat. Stall detection: two consecutive decisions with the same
signature → halt as blocked. It still **yields** at PatchDraft/model
generation, missing test approvals, apply-approval proposal creation, and
repair queueing — those run through narrower renderer-triggered steps
(`agent_run_patch_draft_step`, `agent_run_patch_apply_step`,
`agent_run_test_step`, `agent_run_review_step`), each of which re-asks the
scheduler and re-verifies the decision before acting.

**`agent_executor.rs` — node execution.** Patch proposal/apply nodes check
`ApprovalEngine.assert_can_execute_action_for_run()` at execution time,
append ledger events, record artifact IDs and diff evidence, and mark nodes
completed/failed.

The run ledger is the spine: `AgentRun { nodes, events, artifacts, evidence,
outcome }`, fully persisted, fully rendered in the UI. Node kinds: classify,
explore, plan, model_call, tool_proposal, wait_for_approval, tool_execution,
patch_proposal, diff_review, test_execution, verify, repair, answer,
memory_candidate, external_agent, done, blocked.

### 5.2 Permission engine (approvals)

`approval.rs` + `approval_bridge.rs`. Risky actions produce a typed
`ActionProposal` **before** execution: action type, risk label
(low/medium/high/dangerous), scope, rationale, expected result, rollback
plan, expiration, run/node IDs. States: pending / approved / denied /
expired. Denied and expired cards stay visible (no disappearing trust
failures). Expired approval requests are requeued with **fresh bridge client
IDs** so SQLite approval dedupe can never revive stale execution authority.

Risk taxonomy floors (exposed read-only via `approval_taxonomy`):

- Low: read project metadata, read approved workspace files.
- Medium: broad reads, memory-save proposal, connector read.
- High: file write/edit, dependency install, connector write, external send,
  **external agent execution**.
- Dangerous: destructive terminal commands, broad filesystem access,
  credential operations, networked commands when restricted, headless risky
  scheduled actions.

The tool execution rule, verbatim:

```text
No file write, terminal command, connector send, durable memory save,
scheduled risky action, or external agent run can execute without an
approval record linked to the active AgentRun.
```

### 5.3 Patch system

`patch.rs` / `patch_bridge.rs` / `patch_persistence.rs`. Whole-file
replacement patches (diff-based deltas are Tier-2 roadmap, §11) with:

- intent preflight (Codex `apply-patch`-inspired): files classified
  `create` / `modify`; no-op proposals rejected before approval or disk
  writes (ADR-0010);
- propose ≠ apply: the proposal approval is **not** write authorization; a
  separate exact apply approval is required (ADR-0009);
- stale-file check: apply verifies the file still matches the proposed
  `before` text; restore verifies it still matches the applied `after`;
- checkpoint before every write; restore is itself approval-gated and
  records rollback receipts;
- everything (diff lines, change kind, checkpoint IDs, approval/run IDs)
  persists to SQLite and renders as a first-class diff receipt.

### 5.4 Test runner & command execution

`test_run_approved` runs only commands accepted by the TestRunner against an
exact executable approval; captures command, cwd, exit code, duration,
stdout/stderr (capped, truncation-flagged), parsed failures, timestamps, and
timeline events as a persisted `TestArtifact`. `command_exec` is the shared
execution-artifact shape (Codex app-server-protocol-inspired) used by tests
and approved external terminal workers. `terminal_command_prep` applies
PowerShell UTF-8 setup without changing the visible approved command.

Final answers must use exactly one of: **tested** (artifact linked) /
**not tested** (reason) / **partially tested** (scope + artifact) /
**failed tests** (artifact). Default not-tested reason: "No approved test
command was executed."

### 5.5 Review & repair

`review_create` runs the read-only ReviewAgent over persisted patch + test
artifacts; findings are prioritized, file/diff-linked, persisted. Reviews
with unresolved findings **block final-support readiness** until accepted or
repaired. `agent_request_review_revision` records a bounded repair-request
marker (repair node + event + artifact, no writes), then the repair path
queues a scoped repair PatchDraft FileWrite approval for the exact finding
path and rejoins the normal draft → apply → test → review cycle.

### 5.6 Evidence layer

`EvidenceStore` + AgentRun EvidenceRecords persist source kind (approval,
review, model_call, local_file, repo_symbol, terminal, test, diff, web,
memory, external_agent), locator, quote, hash, retrieval timestamp, and
relevance (relationship + score + reason). Final-answer support synthesizes
missing receipts from persisted file reads, model calls, diffs, reviews,
approvals, and passed tests, then links evidence + test artifact IDs into
`AgentOutcome`. Name-only evidence cannot support final claims. Support
states render visibly: supported / unsupported / insufficient / partial /
untested.

### 5.7 External agent bridge (Claude Code / Codex CLI as workers)

`external_agent_run_bridge.rs` + typed command contracts. Delyx stays the
control plane: scope, approvals, isolation, transcript, diff capture, and
accept/revert decisions are all Delyx-owned.

Launch contracts (visible to the user before approval):

```text
Claude Code:  claude -p --output-format stream-json --verbose
              --permission-mode {plan|acceptEdits} --max-turns 12
              --allowedTools {Read|Read,Edit}  "<task>"
Codex CLI:    codex exec --json --sandbox {read-only|workspace-write} "<task>"
```

Flow: two approval cards (`external_agent` + `terminal_command`, both must be
approved and unexpired) → for write runs, the `[files: …]` plan defines the
checkpoint scope (no declared files = launch blocked, because no useful
rollback receipt is possible) → temp-backed checkpoint created → subprocess
runs via `spawn_blocking` (no webview freeze) with a 10-minute worker
timeout → transcript parsed (stream-json / JSONL) into ordered events →
byte-level diff of planned files → **unplanned-edit cross-check** (anything
touched outside plan blocks the thread) → diffs **promoted into an applied,
checkpointed Delyx patch** so the normal review UI and approval-gated
restore path apply.

CLI policy (project rule): prefer the Claude/Codex **CLIs on subscription
auth** over direct cloud APIs. Headless Claude uses `--safe-mode` (never
`--bare`, which forces API-key billing).

### 5.8 Model layer

Roles: answer, helper, deepResearch, maxReasoning, coding, embedding,
scoring. Role routes persist to SQLite and only save against providers whose
health is ready; missing-key/unreachable providers stay visible but unusable.

Providers:

- **delyx-local (embedded)** — `model_embedded.rs`, mistral.rs in-process
  GGUF runtime. Import a `.gguf` in Settings (Ollama blob reuse supported);
  lazy load; token-by-token streaming (`model-stream` events: token / done /
  cancelled); per-request cancellation. Weights stay on disk; removing a
  profile never deletes the file. CPU by default; CUDA via
  `embedded_mistral_cuda` (`CUDA_COMPUTE_CAP`: 120 = RTX 50, 89 = RTX 40,
  86 = RTX 30; needs CUDA 12.x + MSVC `cl.exe` from a VS dev shell).
- **ollama-local** — loopback HTTP adapter (`/api/tags` health,
  `/api/chat`); non-streaming today.
- **claude-cli / codex-cli** — `cli_chat.rs` chat adapters (also usable as
  QA/QC reviewer); non-streaming, artifact-captured.
- **OpenAI-compatible / Anthropic direct** — health/config states only;
  explicitly rendered as not-wired, no fake availability.

Entry points: `model_chat` (plain), `model_chat_stream` (streaming),
`model_chat_tools` (streaming + §5.1 tool loop), `model_chat_cancel`.

**QA/QC reviewer** (the cross-model moat): after a primary answer posts, a
cheap second model (CLI adapter) reviews it asynchronously with a
fix-and-verify loop. **It never gates or delays the answer** — findings are
append-only follow-ups. Skips cancelled partials. Same rule everywhere
(chat, code, campaign continuity).

### 5.9 Projects, attachments, context packs

Native `ProjectRecord` (trust, approved roots, scopes, policy, rules-file
detection: `DELYX.md`, `AGENTS.md`, `CLAUDE.md`, `.delyx/rules/*.md`,
`.delyx/memory/*.md`). Reads outside approved roots fail; writes outside are
denied; scope changes require user action.

Attachments pipeline: propose → classify → approve → parse → chunk →
**budgeted context pack** (pinned first, then fill by token estimate;
"partial" status when items are excluded) → evidence locators
(`main.rs#L1-L80` chips). Deliberate, visible context selection instead of
opaque stuffing — this is the context-engineering surface the CLIs hide.

### 5.10 Memory, skills, automations, release

- **Memory governance**: candidates → approval-gated promotion → durable
  records with source run/thread receipts, suppression/supersession. Failed
  runs never auto-promote memory.
- **Skill registry**: imported manifests with trust, permissions, source
  hashes; activation changes registry state only — execution paths must
  still check permissions.
- **Automation engine**: mission contracts (what/when/where/tools/stops),
  approval-gated activation, workspace-drift blocking; risky scheduled runs
  create approval cards instead of executing silently.
- **Release**: Windows-first unsigned NSIS dev packaging (ADR-0004);
  redacted support bundles; signing/updater are environment-blocked
  (certificate needed).

### 5.11 Campaign Mode (war-RPG engine)

Local-first roleplay engine that beats Character.AI by separating the
**narrator** (local model) from the **state** (SQLite). The model never owns
the truth — the app does. Single local user (family use); built for a kid,
so the **parent-controlled content dial** is first-class. Status: **C1–C8
shipped**; entry: dice icon in the rail / `Ctrl+G`.

| Character.AI weakness | Campaign Mode answer |
|---|---|
| Forgets after ~40 messages | Canon in SQLite, injected every prompt; event ledger + rolling summary make turn 200 remember turn 3 |
| No game state | Character sheets, squad roster, world clock, inventory are rows, not prose — "who is alive" is a `WHERE status != 'dead'` query |
| Player always wins | The **app** rolls dice (2d6 + stat vs difficulty) and tells the model the outcome; rolls are shown — visible fairness |
| Hallucinated history | Curated era lore packs ride the attachment/context-pack pipeline, relevance-ranked per turn |
| No fact-checking | Async continuity QA/QC via the CLI reviewer — anachronisms ("M1 Garand in 1917") become `historical` correction events the next turn weaves in |
| One global corporate filter | Per-campaign rating: **story** (~PG) / **heroic** (~PG-13) / **historical**; enforced in the prompt overlay AND checked by QA/QC; rating changes are parent-gated via the approval-card pattern |

Turn loop (`campaign_bridge.rs` + ~40 modules):

```text
1 RESOLVE   app-side dice (deterministic Rust; no model authority over success)
2 ASSEMBLE  9-layer GM prompt: [1] GM contract (never speaks for the player,
            narrates given outcomes, ends on a hook + ```delta block)
            [2] era voice  [3] rating overlay  [4] canon-world (clock,
            location, timeline pressure ≤30 days)  [5] canon-characters
            [6] event ledger (full)  [7] rolling memory summary
            [8] top-k lore chunks  [9] pending QA/QC corrections
            + last ~10 turns verbatim + player text + RESOLUTION block
3 NARRATE   model_chat_stream — tokens stream into the play view; cancel works
4 DELTA     trailing fenced ```delta json parsed in Rust, stripped from prose,
            validated (unknown character → entry rejected; dead can't act;
            clock only moves forward), applied to SQLite; malformed delta is
            non-fatal (narration posts; QA/QC backstops)
5 PERSIST   atomic turn row + canon events (model dies mid-turn → no row,
            retry affordance, state untouched)
6 QA/QC     async CLI reviewer: continuity, anachronisms, rating violations,
            missed deltas → ✓/⚠ chip on the turn; never blocks
```

Era packs are content folders — adding a war is authoring, not engine code:
`pack.json` (gmStyle, checks, ratingOverlays), `scenarios.json` (seeds with
squad + timelinePressure), `lore/*.md` (chunked by the attachment parser),
`sheet-schema.json`. Shipped: WW1, WW2, Civil War, Revolutionary War, Korea,
Vietnam, Star Wars (local personal use). **Drop-in user packs** load from a
user folder with no rebuild. Memory: every ~15 turns (or on budget overflow)
a background `model_chat` call compresses the oldest turns into
`campaigns.memory_summary`; the event ledger keeps hard facts. Prompt budget
target ~6–8k tokens on a 30B/32k context. SQLite: `campaigns`,
`campaign_characters` (sheet/inventory/bonds JSON + GM notes),
`campaign_turns` (player text + resolution + narration + delta + qaqc per
row), `campaign_events` (the canon ledger).

## 6. Frontend architecture

The frontend renders **truthful, UI-ready runtime state** returned by typed
Tauri commands; it never infers runtime truth from logs or hidden side
effects. No Redux/Zustand — focused components, local hooks, typed bridge
clients (`modelClient.ts`, `campaignClient.ts`, `externalAgentClient.ts`,
`agentExecutorClient.ts` pattern: `invoke` + event `listen`).

Shell: `FocusShell` (ADR-0007) — views `home | thread | settings | campaign`;
rail navigation, command palette (Ctrl+K, **UI-state actions only — it can
never execute tools**), thread switcher, model picker. Thread view: goal,
mode, status, plan, step timeline, scheduler next-action line, diff/test/
approval/evidence receipts, composer (with "Agent worker" mode), terminal
drawer with search + collapse. Campaign view: narrative timeline, inline
dice blocks, right rail (sheet, roster with status pips, clock/location,
ledger), pack-defined quick-action chips, QA/QC chip.

UI state requirements: every major runtime state has a designed visual state
— empty, loading, error, waiting, blocked, failed, denied, expired, partial,
untested, insufficient-evidence. Local toasts confirm UI-state changes only;
they are never evidence of runtime work. Accessibility: focus states,
keyboard nav for primary actions, accessible dialogs/drawers, non-color
status labels.

Shared enums: `AgentMode` (explore/plan/build/review/test/research/
automation), `TaskStatus` (idle/exploring/planning/waiting_for_approval/
building/testing/reviewing/blocked/failed/done).

## 7. Persistence (SQLite)

Every store follows the same pattern: TEXT ids, JSON columns for nested
structs, ISO timestamps, `persistent()` load + `save_if_persistent()` after
mutation, ID continuity across reload.

| Domain | Tables |
|---|---|
| Threads/runs | task_threads, thread_messages, thread_run_ledgers, _nodes, _events, _artifacts, _evidence, _outcomes |
| Patches | patch proposals/applies (+ checkpoint receipts, change kind) |
| Approvals | action_proposals (+ decisions, expiry, UI scope) |
| Tests | test artifacts, parsed failures, command-exec events |
| Reviews | review reports, ordered findings, suggested fixes |
| External agents | run artifacts, transcript events, diff summaries, linked test IDs |
| Projects | workspace projects, approved roots, rules files, git metadata, indexed names |
| Attachments | records, chunks, context packs, evidence locators |
| Memory | candidates, durable records, suppression |
| Models | embedded profiles, sampling params, role routes |
| Skills/automations | manifests, trust, contracts, scheduled runs |
| Release | profile, smoke receipts, redacted support bundles |
| Campaign | campaigns, campaign_characters, campaign_turns, campaign_events |

Known weakness: writes are not wrapped in transactions across stores; a
crash mid-operation can leave partial state (see §10.2).

## 8. Core data models

The canonical TS shapes (Rust mirrors are the authority at runtime):

```ts
interface Project { id; name; path; approvedRoots: string[]; git?; rulesFiles; modelProfileId?; createdAt; updatedAt }
interface TaskThread { id; projectId; title; goal; status: TaskStatus; mode: AgentMode; activeRunId?; runIds; createdAt; updatedAt }
interface AgentRun { id; projectId?; threadId?; parentRunId?; goal; mode; status: "created"|"running"|"waiting_for_approval"|"blocked"|"repairing"|"succeeded"|"failed"|"cancelled"; nodes; events; artifacts; evidence; metrics; outcome?; createdAt; updatedAt }
interface ActionProposal { id; runId; nodeId; actionType: "read_file"|"write_file"|"edit_file"|"run_terminal"|"install_dependency"|"save_memory"|"use_connector"|"schedule_work"|"external_send"|"external_agent"; riskLabel: "low"|"medium"|"high"|"dangerous"; requiredPermission; rationale; expectedResult; rollbackPlan?; scope; expiresAt; status: "pending"|"approved"|"denied"|"expired" }
interface EvidenceRecord { id; runId; sourceKind; sourceId; title?; uri?; quote?; hash?; retrievedAt; relevance?: { relationship; score; reason } }
interface AgentOutcome { status: "succeeded"|"failed"|"blocked"; summary; evidenceRecordIds; testArtifactIds }
```

## 9. Build, run, validate

```powershell
.\.tools\npm.cmd run dev:desktop          # Tauri Windows shell (embedded runtime, CPU)
.\.tools\npm.cmd run dev                  # browser preview → http://127.0.0.1:1420
$env:CUDA_COMPUTE_CAP = "120"             # then, from a VS x64 Native Tools shell:
.\.tools\npm.cmd run dev:desktop:cuda     # GPU dev (or use dev-cuda.cmd)
.\.tools\npm.cmd run package:windows      # unsigned NSIS → target/release/bundle/nsis/

# Validation gates (run after every milestone):
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test                     # vitest + file-size budget + contract verifiers
cargo test --workspace                    # or --no-default-features --lib for lean
.\.tools\npm.cmd run build
.\.tools\npm.cmd run smoke:ui
.\.tools\npm.cmd run smoke:tauri          # after packaging
cargo fmt --check
git diff --check

# Deterministic evals:
.\.tools\npm.cmd run eval:response
.\.tools\npm.cmd run eval:agentic
```

## 10. Competitive position vs Claude Code / Codex CLI

Honest harness-to-harness comparison (model quality excluded).

### 10.1 Where Delyx is already ahead (the moat — protect it)

1. **Binding approval gates with receipts.** Writes/commands *physically
   cannot* bypass approvals (Rust execution bridges check the persisted
   approval ledger). Both CLIs rely on softer permission prompts that the
   harness itself can mis-handle.
2. **Checkpointed diffs + restore receipts.** First-class rollback with
   stale-file verification both directions. Claude Code added rewind late;
   Codex leans on git.
3. **Evidence records + truthful final-support states.** Neither CLI links
   claims to artifacts; Delyx refuses to say "tested" without one.
4. **Default-on cross-model QA/QC.** Neither CLI reviews its own output with
   a second model by default — and ours never blocks the answer.
5. **Budgeted, visible context packs.** Deliberate context selection with a
   meter and "partial" honesty vs opaque stuffing.
6. **Unplanned-edit cross-check on external workers.** When a CLI worker
   touches a file outside its declared plan, the thread blocks. Neither CLI
   audits *itself* this way.
7. **Local-first.** In-process GGUF runtime, no cloud assumption, OS-keyring
   secrets, nothing leaves the machine.
8. **Campaign Mode.** A state-owning GM engine with app-side dice, canon
   ledger, and a parent dial — no public agent product has anything like it.

### 10.2 The gap list (severity-ordered; cite-checked)

The defining historical gap — *the CLIs run a tool loop, our model got one
prompt* — is **closed in v1** (read-only loop). What remains:

| # | Gap | Severity | Where |
|---|---|---|---|
| 1 | Tool loop is read-only: no `propose_edit` (→ PatchDraft path) or `run_command` (→ test-runner gates), so the local model can look but not act | **High** | `agent_tool_loop.rs` |
| 2 | No context compaction for code threads — history concatenates until the model window silently truncates (campaign mode already compacts; port it) | **High** | thread prompt assembly |
| 3 | Whole-file replacement patches — token-expensive, fragile on big files; need search/replace or apply-patch-style deltas | **High** | `patch.rs` |
| 4 | Sequential tool calls only (6 max, one at a time); no parallel reads | Medium | `agent_tool_loop.rs:13` |
| 5 | External CLI runs are a 10-minute black box — transcript parsed post-facto, no live streaming to the drawer | Medium | `external_agent_run_bridge.rs` |
| 6 | No token counting anywhere — context management is heuristic estimates | Medium | `context_pack.rs`, prompt builders |
| 7 | Expired approvals block forever; no refresh/re-propose ergonomics; `expire_due()` never proactively called | Medium | `approval.rs:83-93` |
| 8 | No `AutoApprovePolicy` (pre-declared, scoped, expiring consent with visible `auto_granted` records) — Claude's acceptEdits/plan modes are the convenience bar; we must match it *without* creating a bypass | Medium | permission engine |
| 9 | No SQLite transactions across multi-store operations — crash mid-apply can leave partial state | Medium | persistence layer |
| 10 | Ollama non-streaming; cold 30B loads with no keep-loaded/KV-cache reuse — both CLIs feel instant | Medium | `model_ollama.rs`, `model_embedded.rs` |
| 11 | Hardcoded loop constants (6 tool steps, 24 drive steps, 8 KB results) — not tunable per project/model | Low | `agent_tool_loop.rs:13`, `agent_drive.rs:20` |
| 12 | Model gets no loop-state signal ("you have N tool calls left") | Low | tool-loop prompt |
| 13 | Drive loop can't branch, pause to ask a clarifying question, or resume mid-step efficiently | Low | `agent_drive.rs` |
| 14 | No E2E test of the full Explore→Plan→Build→Test→Review chain; no concurrency or context-overflow tests | Low | test suite |
| 15 | Renderer fetches full run snapshots per update — no client cache/optimistic updates | Low | frontend clients |

## 11. Roadmap (ordered by impact)

### Tier 1 — what makes it a peer agent
1. ~~Agentic tool loop v1 (read-only)~~ **DONE 2026-06-09** → **v2: add
   `propose_edit` (routes into PatchDraft/approval) and `run_command`
   (through test-runner gates)** so the local model can act, not just look.
2. ~~Codebase awareness slice 1~~ **DONE** → remaining: model-initiated
   search/read depth + symbols in the repo map.
3. ~~Streaming + interrupt (Delyx Local)~~ **DONE** → remaining: Ollama
   chunked streaming if it stays a daily provider.
4. ~~CLI-as-executor v1 + v2 + diff promotion~~ **DONE (Tier 1 #4 complete)**.

### Tier 2 — close the daily-friction gap
5. Diff-based edits (search/replace deltas, multi-file, token-cheap).
6. Autonomous loop spine: driver-owned PatchDraft + approval-proposal
   creation + repair queueing + hooks → the full
   Explore→Plan→Build→Test→Review engine inside `agent_drive`.
7. Context management: token budgeting for thread history, compaction when
   long, a visible context meter.
8. Permission ergonomics: `AutoApprovePolicy` (scoped, expiring, auditable).
9. Real file paths everywhere: `tauri-plugin-dialog` + file-drop events →
   absolute paths → disk-side parsing.

### Tier 3 — medium
10. Git-aware context + commit assistance. 11. Approval-gated web research
feeding the evidence pipeline. 12. Keep-loaded models + prompt/KV-cache
reuse + load-progress UI. 13. Evidence/diagnostics UI for existing backends.
14. PDF/URL ingestion UI wiring. 15. Thread ergonomics (edit/regenerate,
branch, retry; persisted QA/QC choice). 16. Opt-in cloud provider depth with
a visible cloud-boundary approval.

### Tier 4 — when convenient
17. Subagents/parallel runs. 18. Hooks. 19. Slash/custom commands.
20. Cost/token telemetry. 21. Claude-flag verification against installed
binary + parser-diff/checkpoint cross-check. 22. Ship-it: signing, updater,
upgrade smoke (cert-blocked).

Environment-blocked: cloud routes (HTTPS dep + keys), CLI flag verification
(installed binaries), signing (certificate).

## 12. Frontier edge cases — things no public agent ships

These exploit the one structural advantage we have over Claude Code and
Codex CLI: **we own the runtime**. They run a remote model behind an API;
our model executes in-process, our state is local SQLite, and our loop is
deterministic Rust. Each item below states what, why nobody else does it,
and where it lands. Ordered roughly by leverage-per-effort.

1. **Grammar-constrained tool calls (constrained decoding).** ~~v1 SHIPPED
   2026-06-11~~ as *constrained repair*: when a tool-loop turn looks like a
   tool call but fails to parse (previously it leaked to the user as a
   garbage "answer"), the turn is re-asked once with the sampler locked to
   the tool-call JSON schema via `Constraint::JsonSchema` (llguidance in
   mistral.rs) — the regenerated call is valid by construction, narrated as
   "(repaired)" in the loop. Cloud harnesses can't touch the sampler; they
   parse-and-retry blind. Remaining: constrain campaign ```delta blocks the
   same way (two-pass narrate-then-extract with a schema-locked second
   pass), and consider full constrained tool turns once llguidance grammars
   can express "tool JSON or free prose" cleanly.

2. **KV-cache-aware prompt layout.** Order every prompt stable-prefix-first
   (GM contract / system rules / repo map / canon **before** anything
   per-turn) and make context packs append-only within a session, so the
   embedded runtime reuses the KV cache across turns instead of re-prefilling
   6–8k tokens of a 30B every message. Combined with keep-loaded models
   (Tier 3 #12) this is the difference between "cold local model" and
   "instant local model." Nobody designs their prompt *layout* around cache
   reuse publicly. Lands in: `campaign_prompt.rs`, thread prompt assembly,
   `model_embedded.rs`.

3. **Shadow-apply verification (pre-verified diffs).** Before a diff is even
   *shown* for approval: apply it to a temp checkpoint copy, run the
   already-approved test command there, and attach the pass/fail artifact to
   the proposal card. The user approves a patch that is already proven, not
   promised. Neither CLI verifies an edit before presenting it. All the
   parts exist (checkpoint engine, TestRunner, artifacts) — this is
   composition, not new machinery. Lands in: `patch_bridge.rs` +
   `agent_executor.rs`.

4. **Tool-result prompt-injection firewall.** ~~v1 SHIPPED 2026-06-11~~ for
   the model tool loop (`injection_screen.rs`: untrusted-data wrapping,
   marker neutralization, instruction_override/role_hijack/protocol_mimicry
   screening, ⚠ live narration + thread security receipt — see §5.1).
   Remaining: extend screening to attachment parsing, campaign lore chunks,
   web snapshots, and external-agent transcripts, and persist hits as
   first-class security EvidenceRecords on the run ledger.

5. **Hash-chained, tamper-evident run ledger.** Each persisted event/
   artifact/approval row carries a hash of its predecessor (a local
   blockchain-without-the-coin). "Export verifiable receipt chain" turns an
   agent run into an auditable artifact a third party can check — approvals
   can't be backdated, evidence can't be silently rewritten. No public agent
   has tamper-evident audit logs. Cheap: one column + one hash per write.
   Lands in: persistence layer.

6. **Deterministic replay & seeded runs.** Record sampler seeds, sampling
   params, model-weights hash, and resolved tool outputs per turn (campaign
   turns already store resolution + delta). Any run/turn can then be
   replayed bit-for-bit for debugging ("why did it do that?"), and a
   reported bug becomes a reproducible fixture. Cloud APIs cannot promise
   determinism; an in-process runtime can. Lands in: `model_embedded.rs`
   (seed control) + run/turn persistence.

7. **Model fingerprinting on receipts.** Every model-call receipt records
   the GGUF hash + quantization + sampling params, so "tested/reviewed by
   model X" is a reproducible claim and a silent model/quant swap is
   detectable as drift. Pairs with #6; trivially cheap. Lands in: model
   profiles + evidence records.

8. **Mutation-tested agent patches.** When the agent claims its new tests
   cover its new code: mutate the patched lines (flip a comparison, drop a
   branch) in a shadow copy and confirm the tests *fail*. If they don't, the
   "tested" claim is upgraded honestly to "tests ran but don't constrain
   this change." Mutation testing exists as tooling; no agent applies it to
   its own output as a truthfulness check. Lands in: TestRunner + review
   pipeline, behind the existing artifact rules.

9. **Self-revoking auto-approvals (anomaly-narrowing consent).** When
   `AutoApprovePolicy` lands (Tier 2 #8), make it the first permission
   system that *shrinks itself*: any anomaly — unplanned edit detected,
   injection-screen hit, test regression after an auto-approved write —
   automatically revokes the policy and posts a visible
   `auto_revoked(reason)` record. Claude/Codex permissions only ever ratchet
   looser during a session; ours ratchets tighter on evidence. Lands in:
   permission engine + unplanned-edit/injection signals.

10. **Semantic loop detection.** The drive loop's same-signature stall check
    misses A→B→A→B oscillation and paraphrased repetition. Embed each
    action/tool-call summary with a local embedding model (the `embedding`
    role exists, costs nothing locally) and halt-with-receipt when cosine
    similarity over a sliding window shows a cycle. Cloud agents burn tokens
    looping; ours can afford to *notice*. Lands in: `agent_drive.rs` +
    embedding route.

11. **Idle-time self-evaluation (model report cards).** Overnight, via the
    existing automation-contract machinery (approval-gated, workspace-drift
    blocked), re-run `eval:response` / `eval:agentic` against every
    installed model profile and produce a scored report card per model —
    "your 14B Q5 beats your 30B Q4 on tool-call discipline." Local tokens
    are free; nobody uses idle hardware to continuously benchmark the user's
    own model library. Lands in: automation engine + evals.

12. **Context-pack ablation receipts.** Log exactly which chunks were in
    each prompt (cheap now). Offline (or idle-time), re-run saved turns with
    one chunk held out and score answer drift — producing *measured*
    usefulness per lore/context chunk to improve ranking, and a receipt for
    "why was this in my context?" The CLIs can't even show you what was in
    context; we can show what it was *worth*. Lands in: `context_pack.rs` +
    evals.

13. **Truncation honesty guard.** Detect when the local model's context
    window overflowed (prompt tokens + generation vs n_ctx) and visibly mark
    the answer "context overflowed — answer may be missing earlier thread
    state" instead of silently degrading. Every agent silently truncates;
    being the one that *admits it* is pure trust-layer. Lands in: prompt
    assembly + a token counter (unlocks gap #6 too).

14. **Two-channel output everywhere.** Campaign Mode already proved the
    pattern on a 30B: prose for the human + a machine-validated trailing
    delta for the state. Generalize it to coding answers (claims channel:
    "files I touched, commands I ran, what I'm sure/unsure of" as a
    validated block) so final-support synthesis reads structured claims
    instead of inferring from prose. Lands in: answer prompts +
    final-support bridge, reusing `campaign_delta`-style extraction.

Recommended sequencing: **#1–#3 first** (they directly close gaps #1/#3/#10
while being moat-deepening), then #4/#5/#13 (trust layer), then the rest
opportunistically.

## 13. Architecture decision records

- **ADR-0001** Clean rebuild; old Delyx is reference/spec/eval/salvage only.
- **ADR-0002** Separate app identity: `com.geaux.delyxnext`.
- **ADR-0003** CSS variables + small design system; no Tailwind without a
  new ADR.
- **ADR-0004** Windows-first unsigned dev packaging (NSIS); no pretend
  signing.
- **ADR-0005** `npm run dev` = browser preview; `npm run dev:desktop` =
  Tauri shell. Don't confuse the two when judging desktop polish.
- **ADR-0006** Single-instance desktop shell; renderer command UI over
  native menus.
- **ADR-0007** FocusShell is the mounted workbench; legacy cockpit modules
  are reference only. No Radix/TanStack/Zustand until a demonstrated need.
- **ADR-0008** Single Rust crate until extraction pressure is real.
- **ADR-0009** PatchDraftAgent proposes, never applies: generation and disk
  mutation are separate trust boundaries with separate approvals; the
  scheduler derives all authority from persisted records, never renderer
  hints.
- **ADR-0010** Adapt apply-patch *intent* (create/modify preflight, no-op
  rejection), don't import Codex core.
- **ADR-0011** (2026-06-11) Documentation consolidation: all planning and
  architecture prose collapses into this SOURCE_OF_TRUTH.md. Retired:
  `ROADMAP.md`, `DELYX_NEXT_CAMPAIGN_MODE_ARCHITECTURE.md`,
  `docs/ARCHITECTURE.md`, `docs/PRODUCT_DIRECTION.md`,
  `docs/CODE_WORKBENCH.md`, `docs/EXTERNAL_AGENT_BRIDGE.md`,
  `docs/MIGRATION_FROM_DELYX.md`, `docs/CODEX_REFERENCE_AUDIT.md`,
  `docs/SAFETY_PRIVACY_AND_LOCAL_DATA.md`, `docs/UI_ARCHITECTURE.md`,
  `docs/UI_PRINCIPLES.md` (git history preserves them; the Codex reference
  audit commit was `openai/codex@e093d81`, Apache-2.0). New decisions are
  appended here as ADRs. Behavior changes must update this file in the same
  PR.
