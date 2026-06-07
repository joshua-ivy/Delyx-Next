# Delyx Next Code Workbench

The first MVP is a coding workbench because it gives Delyx Next the fastest path to real usefulness.

Core loop:

```text
Open project
-> start thread
-> explore read-only
-> plan
-> approve
-> build
-> review diff
-> run approved tests
-> inspect evidence
-> accept, revert, or continue
```

## Modes

### Explore Mode

Purpose:

```text
Understand the project without changing anything.
```

Allowed:

- list approved workspace files
- search approved workspace files
- read approved workspace files
- inspect Git status
- read rules files

Not allowed:

- edits
- writes
- terminal commands unless explicitly approved
- dependency installs
- external agents

Output:

- relevant files
- relevant symbols
- architecture summary
- project commands discovered
- risks
- unknowns
- suggested next steps

### Plan Mode

Purpose:

```text
Turn exploration into an implementation plan.
```

Allowed:

- read/search approved workspace
- produce structured plan

Not allowed:

- file edits
- terminal commands
- dependency installs
- external agents

Output:

- goal understanding
- files likely involved
- step-by-step plan
- risks
- tests to run
- permissions needed
- rollback strategy

### Build Mode

Purpose:

```text
Make approved changes safely.
```

Rules:

- requires an approved plan or direct user instruction
- creates a checkpoint before applying edits
- applies patches only after approval unless project policy explicitly allows a low-risk exception
- captures diff artifact
- updates changed files list
- suggests formatter or tests after changes

### Review Mode

Purpose:

```text
Review changes without editing by default.
```

Rules:

- no writes by default
- findings are prioritized
- findings link to file/diff references
- user can ask Delyx to revise, which starts a new plan/build flow

Output:

- prioritized findings
- risk labels
- file/diff references
- suggested fixes
- test gaps

### Test Mode

Purpose:

```text
Run approved verification commands and capture evidence.
```

Rules:

- commands require approval unless policy allows them
- capture stdout, stderr, exit code, duration, and timestamps
- link artifact to active AgentRun
- parse failures when possible
- never claim tested without an artifact

## Workspace Rules

All project work happens inside approved roots.

Reads outside approved roots must fail. Writes outside approved roots must not be proposed unless the user changes project scope.

Rules files to detect:

- `DELYX.md`
- `AGENTS.md`
- `CLAUDE.md`
- `.delyx/rules/*.md`
- `.delyx/memory/project.md`
- `.delyx/memory/agent-notes.md`

## Patch And Diff Flow

MVP flow:

```text
plan approved
-> checkpoint created
-> patch proposed
-> user approves patch application
-> patch applied
-> diff artifact created
-> review panel updated
```

Patch proposal can exist without being applied.

Patch application requires approval.

Revert restores the checkpoint.

## Checkpoint Strategy

Build checkpoints first. Worktrees come later.

Simple mode:

- create checkpoint before edits
- capture changed files
- support revert to checkpoint

Git mode later:

- branch/worktree per task
- parallel task threads
- commit/stage/PR workflow

## Test Evidence Rules

Test artifacts must include:

- command
- working directory
- exit code
- duration
- stdout
- stderr
- parsed failures when available
- started timestamp
- completed timestamp
- run ID
- approval ID when applicable

Final answers must use one of these states:

- tested, with linked artifact
- not tested, with reason
- partially tested, with scope
- failed tests, with artifact

## Review Findings

Review findings should be:

- prioritized
- specific
- linked to file or diff references
- focused on bugs, regressions, missing tests, security, accessibility, and maintainability

Review Mode should not edit unless the user explicitly switches to Build mode or asks for a revision.

## PR Sequence For Code Workbench

The code workbench comes online across PRs 2 through 10:

1. PR 2: realistic shell prototype; shipped fake data removed in later Phase 1 cleanup
2. PR 3: real workspace manager
3. PR 4: real thread manager
4. PR 5: typed AgentRun ledger
5. PR 6: read-only Explore and Plan
6. PR 7: approval engine
7. PR 8: patch proposal and checkpoints
8. PR 9: test runner artifacts
9. PR 10: review mode

## Acceptance Principles

Delyx can only be considered useful as a code workbench when:

- Explore and Plan cannot edit
- risky actions create approval proposals
- edits create diffs
- tests create artifacts
- final answers do not fake tested status
- failures and blockers are visible
- user can accept, revert, or continue from the review surface
