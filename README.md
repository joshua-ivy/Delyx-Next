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

The PR 1-18 implementation sequence is complete in the local scaffold. The UI
defaults to real empty state; deterministic fixtures live under
`apps/desktop/evals`.

## Run Locally

From PowerShell:

```powershell
.\.tools\npm.cmd run dev
```

Then open:

```text
http://127.0.0.1:1420
```

## Validate

```powershell
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test
cargo test --workspace
.\.tools\npm.cmd run build
```

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
