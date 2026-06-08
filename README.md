# Delyx Next

Delyx Next is a local-first, UI-first AI workbench for project threads, plans, approvals, diffs, tests, evidence, and later external agent orchestration.

The desktop UI now follows the Command Deck direction: a mode-tinted spine,
command bar, real work pane, contextual inspector, pinned composer, and hint
bar. Prototype scenario data is intentionally not shipped; the app renders
real local state or truthful empty states only.

Start with the source of truth:

- `DELYX_NEXT_UI_FIRST_CODEX_BUILD_PLAN.md`
- `AGENTS.md`
- `docs/`

The PR 1-18 implementation sequence is skeleton-complete in the local scaffold:
typed models, Tauri bridge slices, honest empty-state UI, and deterministic
slice tests exist. It is not yet functionally complete as an autonomous
workbench. The largest remaining depth gaps are real SQLite persistence, an
AgentRun execution/resume engine, behavioral frontend tests, and the final UI
architecture decision. Deterministic fixtures live under `apps/desktop/evals`.

## Run Locally

From PowerShell:

```powershell
.\.tools\npm.cmd run dev
```

Then open the web preview:

```text
http://127.0.0.1:1420
```

To run the actual Windows desktop shell:

```powershell
.\.tools\npm.cmd run dev:desktop
```

## Validate

```powershell
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test
cargo test --workspace
.\.tools\npm.cmd run build
```

Note: `npm test` currently runs deterministic source-contract and smoke
verifiers, plus non-frontend checks. It is not yet a behavioral React/component
test suite; that is Phase 2 depth work.

Extra deterministic checks:

```powershell
.\.tools\npm.cmd run smoke:ui
.\.tools\npm.cmd run eval:response
.\.tools\npm.cmd run eval:agentic
```

After packaging, verify Tauri artifacts:

```powershell
.\.tools\npm.cmd run smoke:tauri
```

## Windows Dev Installer

```powershell
.\.tools\npm.cmd run package:windows
```

The unsigned NSIS installer is produced under:

```text
target/release/bundle/nsis/
```
