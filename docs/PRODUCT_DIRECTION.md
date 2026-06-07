# Delyx Next Product Direction

Delyx Next is a clean rebuild of Delyx as a local-first, UI-first AI workbench.

Product formula:

```text
Delyx Next =
  Delyx's local-first safety, evidence, model routing, memory, and diagnostics
  + Codex-style project/thread/diff workflow
  + Claude-Code-style terminal/codebase agent flow
  + a first-class desktop UI
```

The old Delyx repo is the baseline reference and salvage pool, not a structure to copy blindly.

Reference:

- https://github.com/joshua-ivy/Delyx

## North Star

Delyx Next should make agent work visible, controllable, reviewable, and resumable.

Default workflow:

```text
Project
-> Thread
-> Explore
-> Plan
-> Approve
-> Build
-> Diff
-> Test
-> Review
-> Accept / Revert / Continue
```

The user should always be able to answer:

- What is Delyx doing?
- What files did it inspect?
- What is it asking permission to do?
- What did it change?
- Did tests actually run?
- What evidence supports the final answer?
- What is blocked, failed, expired, or uncertain?

## Product Promise

Delyx Next is not a generic chat app. It is a desktop workbench for local agentic work across code, research, files, memory, and later automations.

It must feel useful first for coding work:

```text
Open project
-> start task thread
-> inspect project read-only
-> review plan
-> approve risky action
-> review patch/diff
-> inspect test artifacts
-> accept, revert, or continue
```

## Pillars

### UI As Trust Layer

The UI is a core product surface, not decoration.

Every meaningful runtime concept needs a visible state:

- projects
- threads
- modes
- agent runs
- plans
- approvals
- diffs
- tests
- evidence receipts
- terminal output
- external agent transcripts
- memory candidates
- model/provider health
- blocked and failed states

### Local-First Safety

Local project work is the default. Risky actions require explicit approval and visible scope.

Risky actions include:

- file writes or edits
- terminal commands
- dependency installs
- durable memory saves
- connector sends or writes
- scheduled risky work
- external agent execution
- networked actions when network access is restricted

### Project And Thread Workflow

The primary object is not a chat conversation. The primary object is:

```text
Project -> TaskThread -> AgentRun -> Artifacts
```

Each project can contain many task threads. Each thread can contain multiple agent runs. Each run can contain steps, approvals, diffs, tests, receipts, failures, and outcomes.

### Explicit Modes

Delyx Next should use explicit agent modes instead of a vague all-purpose agent toggle.

Core modes:

- Explore
- Plan
- Build
- Review
- Test
- Research

Automate mode comes later, after approvals and run evidence are reliable.

### Diff-First Code Changes

Code changes are reviewed in the UI, not buried in chat.

The review surface must show:

- changed files
- unified diff in the MVP
- patch summary
- risks
- test results
- accept/apply
- reject
- revert checkpoint
- ask for revision

### Test-Evidence-First Claims

Delyx may only say code was tested when a real test artifact exists.

A test artifact records:

- command
- working directory
- exit code
- duration
- timestamp
- stdout
- stderr
- parsed failures when available
- linked run ID
- approval ID when required

If no approved test command ran, the UI and final answer must say:

```text
Not tested
Reason: no approved test command was executed
```

### Evidence Receipts

Final answers should be traceable when evidence exists.

Receipts can include:

- files read
- symbols inspected
- commands run
- tests executed
- diffs produced
- sources cited
- memory used
- external agent transcript
- model calls
- approvals granted
- artifacts created

### External Agent Bridge

Delyx can later launch external agents such as Codex CLI, Claude Code, or a generic terminal agent as controlled workers.

Delyx keeps ownership of:

- scope
- permissions
- approval records
- transcript capture
- diff review
- test artifacts
- final accept/revert decisions

External agents never get broader authority than the current Delyx task.

## Simple Mode And Advanced Mode

### Simple Mode

Simple Mode is the default. It focuses on the core flow:

```text
Project -> Thread -> Plan -> Approve -> Diff -> Test -> Review
```

The UI should be calm, direct, and complete enough for real work.

### Advanced Mode

Advanced Mode contains the deeper cockpit:

- Control Center
- Run Inspector
- model routing
- tool policy
- memory manager
- automation contracts
- diagnostics

Advanced Mode must not be the first experience.

## What To Preserve From Old Delyx

Preserve the concepts that make old Delyx valuable:

- local-first desktop app
- Tauri, React, TypeScript, Rust, and SQLite unless a decision record proves otherwise
- approval-first local actions
- model/runtime routing
- AgentRun-style ledger
- source-backed research
- coding lane
- sandbox/test evidence
- local memory with approval
- diagnostics/control center
- skills as visible capabilities
- mobile companion later
- automations later

## What To Avoid From Old Delyx

Do not copy the failure patterns:

- giant cockpit before the core workflow works
- phrase-list-only routing as core reasoning architecture
- string-shape validators pretending to prove reasoning quality
- symbol-name overlap treated as real evidence
- risky local actions without approval
- tested-code claims without execution artifacts
- UI hiding blocked, failed, denied, expired, partial, or uncertain states

## First Twelve PRs

1. Product Direction and Planning Docs
2. App Shell + Design System + Mock UI Prototype
3. Workspace Manager Wired to UI
4. Thread Manager Wired to UI
5. Typed AgentRun Ledger
6. Explore and Plan Modes
7. Approval Engine
8. Patch Proposal and Checkpoints
9. Test Runner Artifacts
10. Review Mode
11. Model Provider Abstraction
12. External Agent Bridge Prototype

## Non-Goals For The First MVP

These are valuable later, but not before the workbench loop is real:

- broad automation runtime
- full mobile companion
- connector marketplace
- side-by-side diff and per-hunk staging
- PR creation and push flow
- live external agent execution
- full research engine
- durable user memory beyond approved candidates
- elaborate cockpit-first dashboards
- cloud-first sync or hosted account assumptions

## Engineering Guardrail

Keep source files intentionally small. The default budget is 300 lines or fewer, with a split/review threshold at 400 lines and a hard cap at 500 lines unless the file is generated, declarative config, or has a documented exception.
