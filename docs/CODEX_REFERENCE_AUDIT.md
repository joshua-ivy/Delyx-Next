# Codex Reference Audit

Audited reference: `openai/codex` at commit `e093d81`.

License: Apache-2.0.

Purpose: cherry-pick proven local-agent ideas without turning Delyx Next into a
Codex clone. Delyx remains UI-first, approval-first, local-first, and
artifact-first.

## Pulled Now

- PowerShell UTF-8 output preparation from the Codex shell-command pattern.
  - Implemented locally as `terminal_command_prep`.
  - Wired only into approved external terminal worker execution.
  - No new dependency.
  - Original approved command remains visible in artifacts.
- Codex CLI read-only launch contract.
  - Implemented locally through `external_agent_run_bridge`.
  - Requires both `external_agent` and `run_terminal` approvals.
  - Captures stdout, stderr, exit status, duration, transcript events, and UI artifacts.
  - Write-capable launch remains blocked until real checkpoint/worktree isolation exists.
- Command execution artifact shape from app-server protocol ideas.
  - Implemented locally as `command_exec`.
  - Used by approved test commands and approved external terminal workers.
  - Captures command label, cwd, run/approval IDs, timeout, exit status, duration, capped stdout/stderr, truncation flags, and timeline events.
  - No new dependency; approval checks stay in the caller before execution.

## Best Phase 2 Picks

- Exec policy ideas from `codex-rs/execpolicy`
  - Prefix rules with allow, prompt, forbidden decisions.
  - Justifications attached to policy matches.
  - Network rules split from command rules.
  - Delyx adaptation: convert matches into `ActionProposal` records and visible UI policy reasons.

- Deeper command execution protocol ideas from `codex-rs/app-server-protocol`
  - Separate command params, output deltas, timeout, output caps, terminate, resize, cwd, env, sandbox policy.
  - Delyx adaptation: typed `CommandExecArtifact` with timeline events and approval IDs.

- Thread and turn protocol ideas from `codex-rs/app-server-protocol`
  - Separate thread start from turn start.
  - Turn-scoped cwd, workspace roots, approval policy, sandbox policy, model, and additional context.
  - Delyx adaptation: make AgentRun nodes own this data visibly in the UI.

- Storage interfaces from `codex-rs/thread-store` and `codex-rs/agent-graph-store`
  - Storage-neutral thread API.
  - Parent/child topology for spawned agents.
  - Delyx adaptation: use as shape reference for the SQLite repositories, not as direct imports.

- Apply-patch ideas from `codex-rs/apply-patch`
  - Parse patch intent before applying.
  - Report proposed file changes separately from committed deltas.
  - Preserve partial-failure deltas.
  - Delyx adaptation: replace narrow patch proposal logic after SQLite/checkpoint depth exists.

- Git utility ideas from `codex-rs/git-utils`
  - Baseline diffs, branch metadata, diff-to-remote, apply/stage helpers.
  - Delyx adaptation: add only the pieces needed for UI diff review and rollback.

- Keyring abstraction from `codex-rs/keyring-store`
  - Small trait around load/save/delete secrets.
  - Mock store for tests.
  - Delyx adaptation: use for OpenAI-compatible provider secrets if D7 stays in scope.

- Ollama polish from `codex-rs/ollama`
  - Default local OSS model concept.
  - Server version checks.
  - Pull progress reporting.
  - Delyx adaptation: local model readiness UI with no fake availability.

- MCP/tool/plugin patterns from `codex-rs/mcp-server`, `codex-rs/tools`, and protocol schemas
  - Typed tool requests.
  - Approval-shaped elicitation.
  - Delyx adaptation: keep every connector action behind Delyx approval and evidence receipts.

- Sandbox/process hardening ideas from `codex-rs/sandboxing`, `windows-sandbox-rs`, `linux-sandbox`, and `process-hardening`
  - Platform-specific sandbox capability detection.
  - Command isolation and process cleanup.
  - Delyx adaptation: use as design reference before importing any OS-specific backend.

## Avoid For Now

- Whole `codex-rs/core` import.
- Whole app-server daemon/client stack.
- Generated protocol macro stack.
- Cloud auth and ChatGPT OAuth paths.
- Broad Starlark exec-policy dependency before Delyx has a SQLite-backed policy store.
- Tree-sitter command parsing before command summaries need that depth.

## Import Rule

Every Codex-derived or Codex-inspired import must:

- name the reference crate/module in docs or comments when code is directly derived
- stay behind Delyx approval gates
- produce UI-visible state
- have deterministic tests
- avoid broad dependencies unless the PR explains why
- keep files inside the Delyx line budget
