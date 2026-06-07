# External Agent Bridge

Delyx Next can launch approved generic terminal workers as controlled workers.
Codex CLI and Claude Code now have typed command contracts, but launch still
requires explicit external-agent and terminal-command approvals.

Examples:

- Codex CLI
- Claude Code
- generic terminal agent

Delyx remains the control layer.

## Core Rule

```text
External agents never get more permission than Delyx gives the current task.
```

## Responsibilities

Delyx owns:

- project scope
- workspace root
- approval records
- checkpoint or worktree
- task prompt
- timeout
- allowed tools
- transcript capture
- terminal output capture
- diff capture
- test artifact capture
- review UI
- accept/revert/continue decisions

The external worker only performs the approved task inside the approved scope.

Current implementation:

- generic terminal adapter can run one approved `terminal_command`
- command cwd must be inside the approved project scope and allowed paths
- checkpoint or worktree isolation is required before launch
- stdout, stderr, command label, exit status, and duration are captured
- nonzero command exits create visible failed artifacts
- Codex CLI and Claude Code adapters are detection-only and report whether their executables are on PATH
- Codex CLI contracts use `codex exec` with explicit sandbox mode and JSONL output
- Claude Code contracts use `claude -p` with stream JSON output, permission mode, and restricted tools
- Codex CLI and Claude Code commands still run only through the approval-gated terminal worker path

## Worker Flow

```text
User starts task
-> Delyx creates thread/run
-> Delyx creates checkpoint or worktree
-> Delyx proposes external worker action
-> user approves
-> Delyx launches worker in approved scope
-> transcript streams into bottom drawer
-> changed files are captured as diff
-> tests are captured if run
-> Delyx shows review panel
-> user accepts, reverts, or continues
```

## Adapter Interface

```ts
interface ExternalAgentAdapter {
  id: string;
  displayName: string;
  detect(): Promise<ExternalAgentAvailability>;
  run(request: ExternalAgentRunRequest): AsyncIterable<ExternalAgentEvent>;
  stop(runId: string): Promise<void>;
}
```

Availability:

```ts
interface ExternalAgentAvailability {
  available: boolean;
  version?: string;
  path?: string;
  reason?: string;
}
```

Run request:

```ts
interface ExternalAgentRunRequest {
  runId: string;
  threadId: string;
  projectId: string;
  cwd: string;
  prompt: string;
  timeoutMs: number;
  allowedTools: string[];
  env: Record<string, string>;
  approvalId: string;
  checkpointId?: string;
  worktreePath?: string;
}
```

Captured events:

```ts
type ExternalAgentEvent =
  | { type: "started"; timestamp: string }
  | { type: "stdout"; text: string; timestamp: string }
  | { type: "stderr"; text: string; timestamp: string }
  | { type: "file_changed"; path: string; timestamp: string }
  | { type: "command"; command: string; timestamp: string }
  | { type: "test_result"; artifactId: string; timestamp: string }
  | { type: "completed"; exitCode: number; timestamp: string }
  | { type: "failed"; error: string; timestamp: string };
```

## Approval Requirements

External agent execution is high risk by default.

Approval must show:

- adapter name
- exact project scope
- working directory
- task prompt summary
- allowed tools
- timeout
- expected result
- rollback plan
- checkpoint or worktree ID
- expiration

## Scope Restrictions

An external agent must not:

- run outside approved workspace scope
- inherit broad shell permissions by default
- access secrets unless explicitly allowed
- use network when network is disabled or restricted
- apply changes without Delyx capturing the diff
- make test claims unless Delyx captured test artifacts

## PR 12 Prototype Scope

PR 12 should implement:

- bridge abstraction
- adapter detection for Codex CLI
- adapter detection for Claude Code
- generic terminal-agent adapter shape
- approval-gated run request
- transcript capture
- diff capture
- terminal output capture
- test output capture if available
- UI panels for transcript, diff, and output

It should not attempt broad autonomous delegation before approvals, checkpoints, diffs, and test artifacts are reliable.
