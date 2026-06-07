# Safety, Privacy, And Local Data

Delyx Next is local-first by default.

The product should make local authority visible: what Delyx can read, what it can change, what can leave the machine, what was remembered, and what still needs approval.

## Safety Principles

- Risky actions require approval.
- Approval scope must be visible.
- Failed, blocked, denied, expired, partial, and uncertain states must be visible.
- File edits must be checkpointed or isolated.
- Terminal commands must be captured as artifacts.
- Test claims require test artifacts.
- Source-backed claims require evidence records.
- Secrets must not be stored in the repo.
- External agents never get broader authority than the current Delyx task.

## Risk Taxonomy

Low risk:

- read project metadata
- read already-approved workspace files

Medium risk:

- broad file reads
- memory save proposal
- connector read

High risk:

- file write/edit
- dependency install
- connector write
- external send
- external agent execution

Dangerous:

- destructive terminal commands
- broad filesystem access
- credential-related operations
- networked commands when restricted
- scheduled/headless risky action

## Approval Requirements

Approval records must include:

- action type
- risk label
- scope
- reason
- expected result
- rollback plan where applicable
- expiration
- run ID
- node ID

Approval states:

- pending
- approved
- denied
- expired

Denied and expired approvals should block the node and show clearly in the UI.

## Tool Execution Rule

```text
No file write, terminal command, connector send, durable memory save,
scheduled risky action, or external agent run can execute without an
approval record linked to the active AgentRun.
```

## Workspace Scope

Projects define approved roots.

Rules:

- Reads outside approved roots fail.
- Writes outside approved roots are denied.
- Approved scope is shown in project UI.
- Scope changes require user action.
- Workspace drift should block scheduled work later.

## File Edits

File edits require:

- approved plan or direct user instruction
- approval record for patch application
- checkpoint before applying edits
- diff artifact after applying edits
- visible review state
- revert path

## Terminal Commands

Terminal commands require approval unless a project policy allows that command class.

Captured artifact:

- command
- working directory
- exit code
- duration
- started timestamp
- completed timestamp
- stdout
- stderr
- approval ID
- run ID

## Test Evidence

Delyx may only claim tested behavior when an execution artifact exists.

Allowed final states:

- tested, with linked artifact
- not tested, with reason
- partially tested, with scope and artifact
- failed tests, with artifact

Default not-tested reason:

```text
No approved test command was executed.
```

## Memory Governance

Durable memory saves require approval.
Promotion approvals are bound to the reviewed memory candidate, not only the source run.

Memory records should include:

- source run ID
- source thread ID
- created timestamp
- scope
- reason
- supersession/suppression state

Failed runs must not auto-promote memory.

## Model And Network Privacy

The UI must show:

- active provider
- model role routing
- provider health
- whether remote providers are configured
- whether networked tools are allowed
- what can leave the machine

Secrets must be stored outside the repo. Missing secrets must produce a clear UI state.

## External Agents

External agent execution is high risk by default.

External agents require:

- explicit approval
- approved workspace scope
- checkpoint or worktree
- timeout
- transcript capture
- diff capture
- test artifact capture when tests run

Tests run by an external agent are not trusted unless Delyx captures them as artifacts.

## Automations Later

Recurring work should remain paused until approved.

Automation contracts must define:

- what can run
- when it can run
- where it can run
- what tools it can use
- what stops it
- what requires fresh approval
- delivery targets

Risky scheduled actions should create approvals instead of executing silently.
