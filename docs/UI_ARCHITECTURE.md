# Delyx Next UI Architecture

The frontend should render truthful, UI-ready runtime state. It should not infer runtime truth from messy logs or hidden backend side effects.

Backend commands should return typed view models that map directly to the workbench panels.

## Frontend Structure

Target structure:

```text
apps/desktop/src/
  app/
    AppShell.tsx
    routes.tsx
    providers.tsx

  design-system/
    tokens.css
    Button.tsx
    IconButton.tsx
    Badge.tsx
    Dialog.tsx
    Drawer.tsx
    Tabs.tsx
    Tooltip.tsx
    SplitPane.tsx
    CommandPalette.tsx
    EmptyState.tsx
    ErrorState.tsx
    LoadingState.tsx
    StatusPill.tsx

  features/
    projects/
    threads/
    task/
    review/
    approvals/
    tests/
    terminal/
    evidence/
    models/
    memory/
    control-center/
```

Only create folders and modules needed by the current milestone. Keep the target structure in docs, but do not fill the repo with empty abstractions.

## App Shell

The app shell owns persistent layout:

- top bar
- left sidebar
- center task thread area
- right review panel
- bottom drawer
- command palette
- theme provider

The shell should preserve user layout preferences later, but PR 2 can use deterministic default pane sizes.

## Core Panels

### Top Bar

Shows:

- active project
- approved workspace root
- branch or checkpoint/worktree
- model/provider health
- active mode
- run status

### Left Sidebar

Shows:

- project list
- thread list
- active project health
- pinned or recent projects
- navigation to skills, automations later, memory, and settings

### Center Panel

Shows:

- active thread
- user goal
- conversation
- composer
- plan
- step timeline
- current blockers

### Right Review Panel

Uses tabs or a segmented control for:

- Diff
- Tests
- Approvals
- Evidence
- Findings

### Bottom Drawer

Shows:

- terminal output
- logs
- test output
- external agent transcript

## State Contracts

Use a shared mode enum anywhere a task/thread/run mode is displayed.

```ts
type AgentMode =
  | "explore"
  | "plan"
  | "build"
  | "review"
  | "test"
  | "research"
  | "automation";
```

Task status:

```ts
type TaskStatus =
  | "idle"
  | "exploring"
  | "planning"
  | "waiting_for_approval"
  | "building"
  | "testing"
  | "reviewing"
  | "blocked"
  | "failed"
  | "done";
```

Thread summary:

```ts
interface TaskThreadSummary {
  id: string;
  projectId: string;
  title: string;
  status: TaskStatus;
  mode: AgentMode;
  branch?: string;
  worktreePath?: string;
  changedFilesCount: number;
  pendingApprovalsCount: number;
  lastRunStatus?: string;
  updatedAt: string;
}
```

Active task view model:

```ts
interface ActiveTaskViewModel {
  thread: TaskThreadSummary;
  plan?: PlanViewModel;
  timeline: TimelineItem[];
  review: ReviewViewModel;
  tests: TestRunViewModel[];
  approvals: ApprovalViewModel[];
  evidence: EvidenceViewModel[];
  terminal: TerminalBlockViewModel[];
  blockers: BlockerViewModel[];
}
```

## Project Screen Contract

Project cards should show:

- name
- path
- approved roots
- Git repo state
- branch
- uncommitted changes
- active thread count
- last run state
- model profile
- rules files found
- approval policy summary

## Thread View Contract

Thread view should show:

- goal
- current mode
- current status
- active plan
- step timeline
- files touched
- commands run
- pending approvals
- test result summary
- evidence receipts
- final answer or blocker

## Plan Panel Contract

Before edits, plan view should show:

- goal understanding
- relevant files
- proposed steps
- risks
- tests to run
- permissions needed
- rollback strategy

Actions:

- approve plan
- edit plan
- ask question
- switch to read-only review
- cancel

## Review Panel Contract

Diff review should show:

- changed files
- unified diff in MVP
- patch summary
- risk summary
- test result
- accept/apply
- reject
- revert checkpoint
- ask for revision

Later:

- side-by-side diff
- per-hunk approve/reject
- inline comments
- stage/unstage
- commit
- push
- PR creation
- open in editor
- copy patch

## Test Panel Contract

The test panel answers:

```text
Did this actually run, and what happened?
```

It must show:

- command
- working directory
- exit code
- duration
- stdout
- stderr
- parsed failures
- timestamp
- run ID
- approval used

If no approved command ran, render the untested state.

## Approval Drawer Contract

Approval cards must show:

- requested action
- risk level
- reason
- scope
- expected result
- rollback plan when applicable
- expiration
- run ID
- node ID

Actions:

- approve once
- deny
- edit scope
- always allow for this project later

Project-wide always-allow policies should be delayed until the basic approval engine is stable.

## Evidence Panel Contract

Evidence receipts should show:

- source kind
- title
- URI or local path
- retrieved timestamp
- hash when available
- relationship to claim
- quote or summary when safe
- linked run/artifact

Name-only evidence cannot support final claims by itself.

## Command Palette

Current implementation: the command palette is a safe local shell surface. It
opens with the top-bar K control or Ctrl+K and only dispatches UI-state actions.
It does not execute file writes, terminal commands, connectors, memory saves, or
external agents.

Early commands:

- open workspace
- open thread manager
- create plan
- approve, revise, or cancel plan
- show deterministic thread/workspace states
- toggle bottom drawer
- toggle theme

## UI Data Boundary

Frontend state should be boring and truthful:

- no hidden success assumptions
- no test claim without artifact
- no final answer support without evidence records
- no write state without approval record
- no failed or blocked state collapsed into a generic idle state
