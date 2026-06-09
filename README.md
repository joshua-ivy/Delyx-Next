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

## Local models

Delyx Next can run a **Delyx Local** model in-process — import a local `.gguf`
file in Settings → Providers & Keys, select it as your chat model, and it answers
through the embedded runtime. Ollama remains an optional adapter, and the
Claude/Codex CLIs can be selected for chat or as a QA/QC reviewer. The embedded
runtime is **on by default** (`embedded_mistral` feature), so `npm run dev:desktop`
and `npm run package:windows` build it automatically — Delyx Local is the default
provider and you switch to Ollama in Settings if you prefer. A lean build without
the embedded runtime is available via `cargo build --no-default-features`.
Imported model weights stay on disk; removing a profile never deletes the file.

### GPU (CUDA) build — NVIDIA

The default build runs the embedded runtime on **CPU**. To use an NVIDIA GPU
(much faster), build with the CUDA feature:

```
npm run dev:desktop:cuda        # dev
npm run package:windows:cuda    # installer
```

Prerequisites (on the build machine):

- A recent NVIDIA driver and the **CUDA Toolkit 12.x** (`nvcc` on `PATH`). For
  RTX 50-series (Blackwell, compute capability 12.0) use **CUDA Toolkit 12.8+**.
- If the build can't auto-detect your GPU's compute capability, set it first:
  PowerShell `$env:CUDA_COMPUTE_CAP = "120"` (120 = RTX 50-series, 89 = RTX 40,
  86 = RTX 30).

The first CUDA build is heavy (it compiles candle's CUDA kernels). Pick a model
that fits VRAM — a 14B `Q4_K_M`/`Q5_K_M` (~9–11 GB) fits a 16 GB card with room
for context; a 32B does not fit 16 GB without slow CPU offload.

The PR 1-18 implementation sequence is skeleton-complete in the local scaffold:
typed models, Tauri bridge slices, honest empty-state UI, and deterministic
slice tests exist. It is not yet functionally complete as an autonomous
workbench. The largest remaining depth gaps are real SQLite persistence, an
AgentRun execution/resume engine, behavioral frontend tests, and the final UI
architecture decision. Deterministic fixtures live under `apps/desktop/evals`.

## Run The Windows Desktop App

From PowerShell:

```powershell
.\.tools\npm.cmd run dev:desktop
```

This launches the native Tauri Windows shell for local desktop QA.

## Run The Browser Preview

Use this for fast UI iteration when you do not need native shell behavior:

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

Current dev installer name:

```text
Delyx Next_0.1.0_x64-setup.exe
```

This is a Windows-first dev package. Signing, updater publishing, and installer
upgrade smoke are still explicit Phase 2 depth work.
