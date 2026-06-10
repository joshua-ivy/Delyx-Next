# Delyx Next — Status & Roadmap

Last updated: 2026-06-09.

This is the single planning document (the three retired root plans live in git
history). Product rules live in `AGENTS.md`; architecture reference lives in
`docs/`. Standing rules: no fake runtime data, every state visible, risky
actions approval-gated, no completed claims without artifacts, files ≤ ~300
lines.

**North star: be as good as — or better than — Claude Code and Codex CLI as a
coding agent**, judged purely as a harness (model quality and datacenters
excluded), while keeping the trust layer they don't have.

---

## Verified current state (2026-06-09)

Gates green: `cargo test --no-default-features --lib` = **415 / 0**, Vitest =
**122 / 0**, typecheck clean, `npm run build` succeeds, `cargo fmt --check`
clean.

Working end-to-end: embedded **mistral.rs** runtime (GGUF import, Ollama-blob
reuse, CUDA build) + Ollama + Claude/Codex CLI chat; **QA/QC reviewer**
(default-on, cheap model, fix-and-verify loop, async, never blocks the answer);
**native projects** (trust/scopes/policy) and the **attachments pipeline**
(propose → classify → approve → parse → context pack → evidence, with chips and
context-pack UI); **approval-gated execution islands** (patch propose/apply/
restore with checkpoints, approved tests, read-only review, repair, final
support); a **bounded Rust driver** over scheduler decisions; SQLite
persistence throughout; Focus desktop shell; unsigned NSIS installer.

---

## Frontier-agent gap review

Honest comparison against Claude Code / Codex CLI **as agent harnesses**.

### Where Delyx is already ahead (the moat — protect it)

- **Binding approval gates with receipts.** Writes/commands physically cannot
  bypass approvals; both CLIs rely on softer permission prompts.
- **Checkpointed diffs + restore receipts.** First-class rollback; Claude Code
  added rewind late, Codex leans on git.
- **Evidence records + truthful final-support states.** Neither CLI links
  claims to artifacts.
- **QA/QC cross-model review.** Neither CLI reviews its own output with a
  second model by default.
- **Attachments → budgeted context packs.** Deliberate, visible context
  selection instead of opaque stuffing.
- **Local-first.** Embedded runtime, no cloud assumption, OS-keyring secrets.

### Where Delyx is behind (the gap list)

The defining difference: **Claude Code/Codex run a tool loop — gather context,
act, observe, iterate. Delyx's model gets one prompt and returns text.** The
local model is blind (no repo search), handless (can't run a command to see
errors), and one-shot (can't iterate on failure). Every tier-1 item below is a
facet of closing that loop without giving up the approval moat.

---

## Roadmap — ordered by impact (massive → minimal)

### Tier 1 — Massive gain (this is what makes it an agent)

1. **Agentic tool loop for the model.** ~~v1 DONE & VERIFIED (2026-06-09)~~:
   bounded Rust-owned loop where Delyx Local calls read-only tools
   (`read_file`/`list_dir`/`grep` — root-scoped, capped, auto-allowed) via a
   JSON protocol that works with any GGUF; tool turns narrate live (🔧 lines in
   the draft), the final answer streams, cancel works, and a tool receipt lands
   in the thread (425 Rust / 143 FE green). **v2 remaining:** `propose_edit`
   (routes into the PatchDraft/approval path) and `run_command` (through the
   test-runner gates) so the local model can act, not just look.
2. **Codebase awareness.** ~~First slice DONE & VERIFIED (2026-06-09)~~: the
   local model's system prompt now carries project identity + branch, a capped
   repo map, and the project's rules files (AGENTS.md/CLAUDE.md — read through
   the scope-enforced bridge, cached per project). Remaining: model-initiated
   search/read (lands with the #1 tool loop) and symbols in the repo map.
3. **Streaming + interrupt.** ~~DONE & VERIFIED for Delyx Local (2026-06-09)~~:
   token-by-token streaming into a live draft bubble, Stop button cancels and
   keeps the partial, QA/QC skips cancelled partials. Ollama/CLI transparently
   fall back to non-streaming (419 Rust / 138 FE green). Remaining: Ollama
   streaming (incremental chunked-HTTP reads) if it stays a daily provider.
4. **CLI-as-executor (the shortcut to frontier quality).** ~~v1 DONE &
   VERIFIED (2026-06-09)~~: composer "Agent worker" mode → two approval cards
   (task visible on the card) → Launch → read-only agentic CLI run → result +
   receipts in the thread; run commands made async (no UI freeze). ~~v2
   (write-mode) DONE & VERIFIED (vitest 135/135)~~: `[files: …]`-tagged tasks give
   write access to exactly the planned files (checkpointed first, shown on the
   approval card), `acceptEdits` run, diff capture, and an unplanned-edit
   cross-check that blocks the thread when the CLI touched anything outside
   plan. ~~Diff promotion DONE & VERIFIED~~: write-run edits are promoted into
   an applied, checkpointed Delyx patch — the existing diff review UI renders
   it and the approval-gated restore path rolls it back. **Tier 1 #4 complete
   (419 Rust / 135 FE tests green).**

### Tier 2 — Large gain (close the daily-friction gap)

5. **Diff-based edits.** Replace whole-file PatchDraft replacement with
   search/replace or apply-patch-style deltas (Codex's `apply_patch` model was
   already evaluated) — multi-file, token-cheap, works on large files.
6. **Autonomous loop spine** (old D2/D5). Driver-owned PatchDraft inside the
   loop, thin renderer dispatcher, remaining deterministic driver tests
   (exact-order, non-progress fixture, external-agent bypass), then hooks.
   With Tier-1 #1 this becomes the full Explore→Plan→Build→Test→Review engine.
7. **Context management.** Token budgeting for thread history, compaction/
   summarization when long, and a visible context meter. Currently the thread
   is concatenated until the model's window silently truncates.
8. **Permission ergonomics.** `AutoApprovePolicy` — pre-declared, scoped,
   expiring consent (file globs, max writes, visible `auto_granted` records).
   Claude Code's acceptEdits/plan modes are the bar; Delyx must match the
   convenience *without* creating a bypass.
9. **Real file paths everywhere.** `tauri-plugin-dialog` (native open/save
   pickers) + Tauri file-drop events so attachments and projects carry absolute
   paths — unlocking disk-side parsing instead of inline-content-only.

### Tier 3 — Medium gain

10. **Git-aware context + commit assistance.** Feed status/diff/log into
    prompts; draft commit messages; baseline/diff helpers (the old D11 salvage
    item, now with a consumer).
11. **Model-initiated web research.** Approval-gated fetch tool feeding the
    existing URL-snapshot → evidence pipeline (the CLIs have WebFetch/Search).
12. **Performance: keep-loaded models + prompt/KV cache reuse** across turns,
    plus load-progress UI. Cold 30B loads are brutal; both CLIs feel instant.
13. **Evidence display + diagnostics UI.** Surface `attachment_evidence_*`
    (locator chips like `main.rs#L1-L80`) and the redacted
    `attachment_report_snapshot` panel — backends exist, no UI.
14. **PDF / URL ingestion UI.** Wire `attachment_parse_pdf` (webview pdf.js)
    and `attachment_external_snapshot` (webview fetch) to existing commands.
15. **Thread ergonomics.** Edit/regenerate a message, branch a thread, retry a
    failed step; QA/QC reviewer choice persisted across launches; clearer
    embedded-vs-Ollama labeling in the picker.
16. **Cloud provider depth** (explicitly opt-in): live OpenAI-compatible +
    direct Anthropic routes with a visible cloud-boundary approval. Needs an
    HTTPS client dep + API keys.

### Tier 4 — Minimal impact (do when convenient)

17. **Subagents / parallel runs** — fan-out is power-user territory; the loop
    must exist first.
18. **Hooks** — post-apply/post-test hook points once the loop owns the cycle.
19. **Slash/custom commands** and reusable prompt snippets.
20. **Cost/token telemetry** — per-turn token counts, QA/QC spend.
21. **D8 tail** — verify Claude CLI flags against the installed binary,
    parser-diff vs checkpoint cross-check, optional `--add-dir`/`--model`.
22. **Ship-it track** — Windows signing, updater, install/upgrade smoke
    (cert-blocked; matters for distribution, not capability).

### Environment-blocked (not startable without inputs)

| Item | Needs |
|---|---|
| Cloud provider routes (16) | HTTPS client dep + API keys + network policy |
| Claude/Codex flag verification (21) | Installed binaries to introspect |
| Signing/updater (22) | Code-signing certificate + release pipeline |

---

## Validation gates

```powershell
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test
cargo test --workspace            # or --no-default-features for the lean build
.\.tools\npm.cmd run build
.\.tools\npm.cmd run smoke:ui
.\.tools\npm.cmd run smoke:tauri
cargo fmt --check
git diff --check
```

## Definition of done (unchanged)

Code compiles; tests prove behavior at the right level; UI states are truthful;
docs match reality; risky actions stay approval-gated; no fake data ships; no
source-backed/tested/completed claims without artifacts; files stay inside the
line budget.
