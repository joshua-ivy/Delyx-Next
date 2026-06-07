# Delyx Next — UI-First Local Agent Workbench Build Plan

> **Product formula:**  
> **Delyx Next = Delyx’s local-first safety, evidence, model routing, and cockpit + Codex-style project/thread/diff workflow + Claude-Code-style terminal/codebase agent flow + a first-class desktop UI.**

This document is meant to be pasted into Codex as the master product and engineering plan for rebuilding Delyx as a clean, UI-first, local-first AI workbench.

---

## 1. North Star

Delyx Next is not just a chat app and not just a CLI with a window.

Delyx Next is a **first-class desktop workbench** for agentic work:

```text
Project
→ Task Thread
→ Explore
→ Plan
→ Approve
→ Build
→ Diff
→ Test
→ Review
→ Accept / Revert / Continue
```

The app should make the user feel:

```text
I can see what the agent is doing.
I can control what it is allowed to do.
I can review every file change.
I can see whether tests actually ran.
I can trace every claim, source, approval, tool call, and artifact.
```

Delyx Next should be useful first for coding and local project work, then expand into research, memory, automations, mobile companion review, and external agent orchestration.

---

## 2. One-Sentence Product Pitch

**Delyx Next is a local-first AI workbench that gives you Codex-style project threads and diffs, Claude-Code-style agent workflows, and Delyx’s approval-first safety, evidence receipts, model routing, memory, and control center.**

---

## 3. What Delyx Next Is

Delyx Next is:

- A real desktop app, not a thin web wrapper feeling like a demo.
- A local-first AI agent workbench.
- A project/thread-based coding and research environment.
- A UI-first control surface for agent work.
- A diff-first code review tool.
- A test-evidence-first coding assistant.
- A safety-first local automation layer.
- A model-router for local and OpenAI-compatible backends.
- A place where external agents like Codex CLI or Claude Code can optionally be launched as controlled workers.
- A system that shows receipts for what it knows, touched, changed, tested, remembered, or delegated.

---

## 4. What Delyx Next Is Not

Delyx Next is not:

- A generic ChatGPT clone.
- A backend-first agent with a frontend added later.
- A mystery-box automation engine.
- A UI that hides failed or blocked states to look clean.
- A tool that claims code was tested when no test artifact exists.
- A tool that edits files or runs commands without clear permission.
- A system that treats symbol-name overlap as evidence.
- A system that uses phrase-list routing as the core reasoning architecture.
- A system that validates reasoning by checking whether final text has the right phrases.
- A giant control cockpit before the simple workflow works.

---

## 5. Main Product Strategy

Build Delyx Next in this order:

```text
1. First-class UI shell and mock workflow
2. Project/workspace manager
3. Task thread manager
4. Typed AgentRun ledger
5. Explore and Plan modes
6. Approval engine
7. Patch/diff/checkpoint system
8. Test runner artifacts
9. Review mode
10. Model provider abstraction
11. External Agent Bridge prototype
12. Source-backed research
13. Memory governance
14. Skills
15. Automations
16. Mobile companion
17. Release and packaging
```

The first MVP should be a **coding workbench** because that is the fastest path to usefulness and gives the project a clean workflow:

```text
Open project
→ ask Delyx to inspect or change something
→ Delyx explores read-only
→ Delyx presents a plan
→ user approves
→ Delyx proposes/applies a patch
→ Delyx runs approved tests
→ user reviews diff and evidence
```

Research, memory, automations, and mobile should come after the core workflow is stable.

---

## 6. Product Pillars

### 6.1 First-Class UI

The UI is not decoration. The UI is the trust layer.

Every meaningful runtime concept must have a visible user-facing surface:

- Projects
- Threads
- Plans
- Agent steps
- Diffs
- Tests
- Terminal output
- Approvals
- Evidence receipts
- Model/provider health
- Blocked states
- Failed states
- Expired approvals
- Memory candidates
- External agent transcripts
- Tool calls
- Files touched
- Commands run

### 6.2 Local-First Safety

Delyx should keep work close to the user’s machine and files by default.

Risky actions must require explicit permission:

- File edits
- File writes
- Terminal commands
- Dependency installs
- Durable memory saves
- Connector sends/writes
- Scheduled risky actions
- External agent execution
- Networked actions when disabled or restricted

### 6.3 Project and Thread Workflow

The main unit should not be a generic chat conversation.

The main unit should be:

```text
Project → Task Thread → AgentRun → Artifacts
```

A project can have multiple threads. A thread can have multiple runs. A run can have steps, approvals, diffs, test artifacts, evidence, and outcomes.

### 6.4 Explicit Agent Modes

Do not use one giant vague “agent mode.”

Use explicit modes:

```text
Explore
Plan
Build
Review
Test
Research
Automate later
```

Each mode has clear permissions and UI.

### 6.5 Diff-First Code Changes

Code edits must be visible in a real review panel.

Do not bury file changes in chat.

The app should show:

- Changed files
- Unified diff
- Side-by-side diff later
- Patch summary
- Risk summary
- Test result
- Revert options
- Ask-for-revision action
- Accept/apply action

### 6.6 Test-Evidence-First Claims

Delyx may only say code was tested when a real execution artifact exists.

A test artifact should include:

- Command
- Working directory
- Exit code
- Duration
- Timestamp
- stdout
- stderr
- Parsed failures if available
- Linked AgentRun ID
- Approval used, if approval was required

If tests did not run, the UI must say:

```text
Not tested
Reason: no approved test command was executed
```

### 6.7 Evidence Receipts

Every final answer should be able to show receipts when evidence exists:

- Files read
- Symbols inspected
- Commands run
- Tests executed
- Diffs produced
- Sources cited
- Memory used
- External agent transcript
- Model calls
- Approvals granted
- Artifacts created

### 6.8 External Agent Bridge

Delyx should optionally launch external coding agents such as Codex CLI or Claude Code as controlled workers.

Delyx remains the control layer:

```text
Delyx owns project scope.
Delyx owns approvals.
Delyx owns evidence capture.
Delyx owns diff review.
Delyx owns test artifacts.
Delyx owns final accept/revert decisions.
```

External workers never get broader authority than the current Delyx task allows.

---

## 7. What to Preserve From Current Delyx

Preserve the best parts of the current Delyx concept:

| Current Delyx Strength | Preserve? | Delyx Next Version |
|---|---:|---|
| Local-first desktop AI | Yes | Make local projects/workspaces the default object |
| Tauri + React + TypeScript + Rust | Yes | Keep unless a decision record proves otherwise |
| SQLite local state | Yes | Use for projects, threads, runs, approvals, artifacts |
| Model provider routing | Yes | Hide advanced routing under settings/power mode |
| Approval-first design | Yes | Make approvals a clean first-class drawer |
| Source-backed research | Yes | Render as evidence receipts and claim support |
| Coding lane | Yes | Make coding workbench the first MVP |
| Python sandbox evidence | Yes | Show tested/not-tested truthfully |
| AgentRun ledger | Yes | Rebuild as actual execution/resume graph |
| Memory | Yes | Use approval-gated project/user memory |
| Diagnostics/control center | Yes | Keep advanced mode, not default mode |
| Mobile companion | Later | Use for review/approval/steering |
| Automations | Later | Use mission contracts and scoped permissions |
| Skills | Yes | Add trust levels and permission scopes |

---

## 8. What to Borrow From Codex-Style Workflows

Borrow workflow ideas, not branding or exact UI.

### 8.1 Project Threads

Delyx should organize work as:

```text
Project
  └── Thread
        └── Runs
              └── Plans / Diffs / Tests / Artifacts / Approvals
```

### 8.2 Worktrees or Checkpoints

Support two isolation modes:

```text
Simple mode: checkpoint before edits
Git mode: branch/worktree per task
```

Build checkpoints first. Add worktrees after patch flow is stable.

### 8.3 Diff-First Review

The review panel should be central:

```text
Plan
Diff
Tests
Risks
Approve / Reject / Revise
```

### 8.4 Review-Only Mode

Add a read-only review mode:

```text
Review my changes
Review this file
Review this PR later
Review for bugs
Review for tests
Review for security
Review for accessibility
```

Review mode should not edit unless the user explicitly switches to Build mode.

### 8.5 Skills

Skills should package reusable workflows:

```text
skills/
  code-review/
    SKILL.md
    scripts/
    references/
  tauri-debugging/
    SKILL.md
  data-center-commissioning-docs/
    SKILL.md
```

Delyx-specific additions:

```text
Skill trust level
Skill source
Skill permission scope
Skill can/cannot run scripts
Skill can/cannot edit files
Skill can/cannot use network
```

### 8.6 Automations Later

Add recurring/background work after the approval/run system is reliable.

Example:

```text
Every morning:
- check repo health
- run approved tests
- summarize failures
- do not edit unless approved
```

---

## 9. What to Borrow From Claude-Code-Style Workflows

Borrow the developer-agent feel:

- Terminal-first task flow
- Read/search/edit/run loop
- Plan before editing
- Project memory files
- Hooks around tools and permissions
- Subagents with restricted tools
- Resumeable sessions
- Checkpoints
- Parallel worktrees later

### 9.1 Explicit Modes

```text
Explore Mode
- read/search only
- no edits
- no terminal unless explicitly approved

Plan Mode
- produce implementation plan
- identify files
- identify risks
- identify tests
- ask for approval

Build Mode
- edit only after approval
- use checkpoints
- run approved commands
- capture artifacts

Review Mode
- inspect diff
- report prioritized findings
- no writes by default

Test Mode
- run approved tests
- capture stdout/stderr/exit code/duration
- link artifacts to AgentRun
```

### 9.2 Hooks

Implement lifecycle hooks:

```text
before_task
before_model_call
before_tool_use
permission_request
after_tool_use
after_edit
after_test
on_failure
on_complete
```

Example hook behavior:

```text
Before terminal command:
- block dangerous commands
- require approval for dependency installs
- require approval for network commands

After edit:
- create diff artifact
- update changed files list
- suggest formatter/test

After test:
- parse failures
- link test artifact to run

On failure:
- create repair run
- attach logs
```

### 9.3 Memory Files

Support common project instruction files:

```text
DELYX.md
AGENTS.md
CLAUDE.md
.delyx/rules/*.md
.delyx/memory/project.md
.delyx/memory/agent-notes.md
```

Example `DELYX.md`:

```md
# DELYX.md

## Project Commands
- npm run typecheck
- npm test
- cargo test --workspace

## Rules
- Do not change approval policy without tests.
- Do not edit generated files.
- Keep Tauri commands typed.
- Never claim code was tested unless test output exists.

## Architecture
- Frontend: React + TypeScript
- Backend: Rust/Tauri
- Store: SQLite
```

### 9.4 Subagents

Use specialized subagents with restricted tools:

```text
ExploreAgent
- read/search only
- summarizes relevant files

PlanAgent
- read/search only
- creates plan and risk list

BuildAgent
- edits only after approval
- proposes patches

ReviewAgent
- reads diff
- gives findings
- no writes

TestAgent
- runs approved tests
- captures artifacts

ResearchAgent
- gathers sources
- creates evidence receipts
```

---

## 10. Delyx Weaknesses to Fix

Do not copy these old failure patterns:

### 10.1 Do Not Build a Giant Cockpit First

The advanced Control Center is valuable, but not as the first user experience.

Default UI should be:

```text
Project → Thread → Plan → Diff → Test → Approve
```

Control Center should be advanced mode.

### 10.2 No Phrase-List-Only Routing

Do not use hand-maintained phrase lists as the core reasoning router.

Use:

```text
Deterministic classifier
+ model/router label where useful
+ fallback rules
+ held-out route evals
```

### 10.3 No String-Shape Reasoning Validators

Do not validate reasoning by checking whether the answer contains words like:

```text
evidence for
evidence against
most likely
ranked
```

Use structured intermediate outputs and evidence records.

### 10.4 No Symbol-Name Overlap as Evidence

A code symbol with a similar name is not proof.

Repo evidence should include:

```text
relationship:
- direct_implementation
- caller
- test
- config
- doc
- name_only
- unknown
```

Name-only evidence cannot support final claims by itself.

### 10.5 No Risky Tool Execution Without Approval

Risky actions must produce an approval request first.

### 10.6 No Fake Tested Claims

Delyx must never say:

```text
I tested this.
```

unless there is an execution artifact.

### 10.7 No UI That Hides Reality

Blocked, failed, partial, expired, denied, waiting, uncertain, and untested states must be visible.

---

## 11. First-Class UI Requirements

### 11.1 UI Principle

The UI is the product and the trust layer.

Delyx Next must feel like a designed desktop workbench from PR 2, even with mock data.

### 11.2 Default Layout

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Top Bar: Project • Branch/Worktree • Model • Mode • Run Status       │
├───────────────┬───────────────────────────────┬──────────────────────┤
│ Projects      │ Active Task Thread             │ Review Panel         │
│ Threads       │                               │                      │
│ Skills        │ Chat / Plan / Steps            │ Diff                 │
│ Automations   │                               │ Tests                │
│ Memory        │ Agent progress timeline        │ Approvals           │
│ Settings      │                               │ Evidence             │
├───────────────┴───────────────────────────────┴──────────────────────┤
│ Bottom Drawer: Terminal • Logs • External Agent Stream               │
└──────────────────────────────────────────────────────────────────────┘
```

### 11.3 Main Regions

```text
Left sidebar:
- Projects
- Threads
- Skills
- Automations later
- Memory
- Settings

Center panel:
- Active task thread
- Chat/composer
- Plan
- Step timeline
- Agent status

Right panel:
- Diff
- Tests
- Approvals
- Evidence
- Review findings

Bottom drawer:
- Terminal
- Logs
- Test output
- External agent transcript

Advanced mode:
- Control Center
- Run Inspector
- Model routing
- Tool policy
- Memory manager
- Automation contracts
- Diagnostics
```

### 11.4 UI Must-Haves

- ~~Command palette with safe local shell actions~~
- ~~Keyboard shortcuts for command palette, primary controls, theme toggle, and pane resize handles~~
- ~~Resizable split panes~~
- ~~Persistent layout for pane sizes and theme preference~~
- ~~Light/dark themes with persisted local preference~~
- ~~Great empty states~~
- ~~Great loading states~~
- ~~Great error states~~
- ~~Toasts/notifications for safe local UI actions~~
- ~~Accessible dialogs, menus, buttons, and drawers~~
- ~~Searchable logs in the bottom drawer~~
- ~~Collapsible long output in the bottom drawer~~
- ~~Status pills for every state~~
- ~~Consistent design tokens~~
- ~~No random one-off styling~~

### 11.5 Design Style

Target feel:

```text
VS Code + Linear + Codex-style workbench + serious local control panel
```

The UI should be:

```text
professional
calm
fast
trustworthy
dense but readable
inspectable
not toy-like
not gamer-like
not cluttered
```

---

## 12. Core Screens

### 12.1 Projects Screen

Purpose:

```text
Choose the repo/folder/workspace Delyx is allowed to work in.
```

Show:

- ~~Recent projects~~
- ~~Pinned projects~~
- ~~Add project~~
- ~~Project health~~
- ~~Git status~~
- ~~Allowed workspace scope~~
- ~~Model profile~~
- ~~Rules files found~~
- ~~Last run status~~
- ~~Active threads~~

Project card fields:

```text
Name
Path
Git branch
Uncommitted changes
Active threads
Last run status
Provider/model health
Approval policy
```

Status update: top-bar Git/isolation truth slice implemented on 2026-06-07.
Status update: workspace manager project surface implemented on 2026-06-07.

- ~~Git UI shows the current `main` branch from real local project facts.~~
- ~~Uncommitted count is not faked; the UI says changes are not loaded until a real dirty-count artifact exists.~~
- ~~Checkpoint/worktree isolation has its own visible chip and starts as no active isolation.~~
- ~~Project card shows name, path, Git status, active threads, last run, provider/model health, and approval policy from real local state or honest empty state.~~

### 12.2 Thread View

Thread states:

```text
idle
exploring
planning
waiting_for_approval
building
testing
reviewing
blocked
failed
done
```

Status update: full workflow thread status vocabulary implemented on 2026-06-07.

- ~~Thread UI and domain model cover idle, exploring, planning, waiting_for_approval, building, testing, reviewing, blocked, failed, and done.~~

Show:

- ~~User goal~~
- ~~Current mode~~
- ~~Agent plan~~
- ~~Step timeline~~
- ~~Files touched~~
- ~~Commands run~~
- ~~Approvals needed~~
- ~~Test result~~
- ~~Final answer~~
- ~~Evidence receipts~~

Status update: current mode pill and workflow pipeline derive from thread status as of 2026-06-07.
Status update: thread summary stats cover commands, approvals, final answer, tests, files, and evidence as of 2026-06-07.

### 12.3 Plan Panel

Before edits, show:

```text
Goal
Understanding
Files likely involved
Proposed steps
Risks
Tests to run
Permissions needed
```

Actions:

```text
Approve Plan
Edit Plan
Ask Question
Switch to Read-Only Review
Cancel
```

Status update: plan panel field and action coverage implemented on 2026-06-07.

- ~~Plan panel shows goal, understanding, files likely involved, proposed steps, risks, tests to run, and permissions needed.~~
- ~~Plan actions include Approve Plan, Edit Plan, Ask Question, Switch to Read-Only Review, and Cancel with safe local UI behavior.~~

### 12.4 Diff Review Panel

MVP:

```text
Changed files
Unified diff
Approve apply
Reject
Revert checkpoint
Ask for revision
```

Status update: diff review MVP controls implemented on 2026-06-07.

- ~~Diff panel shows changed files and unified diff artifacts.~~
- ~~Diff panel exposes Approve apply, Reject, Revert checkpoint, and Ask revision as safe local UI controls.~~

Later:

```text
Side-by-side diff
File tree
Per-hunk approve/reject
Inline comments
Stage/unstage
Commit
Push
PR creation
Open in editor
Copy patch
```

### 12.5 Test Panel

The test panel answers:

```text
Did this actually run, and what happened?
```

Show:

```text
Command
Working directory
Exit code
Duration
stdout
stderr
Parsed failures
Artifacts
Timestamp
Run ID
Approval used
```

Status update: test panel artifact receipts implemented on 2026-06-07.

- ~~Test panel shows command, working directory, exit code, duration, stdout, stderr, parsed failures, artifact ID, timestamp, run ID, and approval used when TestArtifact data exists.~~

### 12.6 Approval Drawer

Each approval card shows:

```text
Action requested
Risk level
Why Delyx wants it
Files/commands involved
Expected result
Rollback plan
Expiration
Approve once
Deny
Always allow for this project later
Edit scope
```

Status update: approval drawer card controls implemented on 2026-06-07.

- ~~Approval cards show action, risk, reason, files/commands scope, expected result, rollback plan, expiration, and safe local controls for approve once, deny, always allow later, and edit scope.~~

Approval types:

```text
Read workspace
Edit file
Run command
Install dependency
Use external agent
Save memory
Use connector
Schedule automation
Send external message
```

### 12.7 Terminal Panel

Support:

```text
Command history
Multiple output blocks
Copy output
Collapse long logs
Search logs
Jump to error
Open referenced file
Rerun command
Approve rerun
```

Status update: terminal drawer support controls implemented on 2026-06-07.

- ~~Terminal drawer shows command history, multiple output blocks, copy output, collapse long logs, search logs, jump to error, open referenced file, rerun command, and approve rerun controls with honest empty states.~~

### 12.8 Evidence / Receipts Panel

Show:

```text
Why Delyx believes this
What Delyx changed
What Delyx tested
What Delyx did not test
What still needs review
Files read
Symbols inspected
Commands run
Diffs produced
Sources cited
Memory used
External agent transcript
Model calls
Approvals granted
```

Status update: evidence coverage matrix implemented on 2026-06-07.

- ~~Evidence panel shows every requested receipt category with real counts when receipts exist or an honest not-recorded/not-tested state when they do not.~~

---

## 13. Design System

Recommended frontend stack:

```text
React
TypeScript
Vite
Tauri v2
CSS variables or Tailwind
Radix UI primitives
Lucide icons
TanStack Query for server state
Zustand only if needed for local UI state
Monaco or CodeMirror for code/diff later
xterm.js or similar for terminal later
```

### 13.1 Component Structure

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
    Card.tsx
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
      ProjectSidebar.tsx
      ProjectCard.tsx
      AddProjectDialog.tsx
      ProjectHealthBadge.tsx

    threads/
      ThreadList.tsx
      ThreadView.tsx
      ThreadComposer.tsx
      ThreadStatusPill.tsx

    task/
      ActiveTaskLayout.tsx
      PlanPanel.tsx
      StepTimeline.tsx
      ModeSwitcher.tsx

    review/
      ReviewPanel.tsx
      ChangedFilesTree.tsx
      DiffViewer.tsx
      InlineComment.tsx
      ReviewActions.tsx

    approvals/
      ApprovalDrawer.tsx
      ApprovalCard.tsx
      RiskBadge.tsx
      PermissionScopeEditor.tsx

    tests/
      TestPanel.tsx
      TestRunCard.tsx
      LogViewer.tsx
      FailureSummary.tsx

    terminal/
      TerminalPanel.tsx
      CommandBlock.tsx
      CommandApprovalBanner.tsx

    evidence/
      EvidencePanel.tsx
      ReceiptCard.tsx
      ArtifactViewer.tsx
      SourceList.tsx

    models/
      ModelStatus.tsx
      ProviderSettings.tsx
      RoleRoutingTable.tsx

    memory/
      MemoryCandidates.tsx
      MemoryReviewCard.tsx

    control-center/
      ControlCenter.tsx
      RuntimeHealth.tsx
      ToolPolicyView.tsx
      RunInspector.tsx
```

---

## 14. UI State Contracts

The frontend should not infer runtime truth from messy logs.

Backend should return UI-ready view models.

### 14.1 Task Status

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

### 14.2 Thread Summary

```ts
interface TaskThreadSummary {
  id: string;
  projectId: string;
  title: string;
  status: TaskStatus;
  mode: "explore" | "plan" | "build" | "review" | "test" | "research";
  branch?: string;
  worktreePath?: string;
  changedFilesCount: number;
  pendingApprovalsCount: number;
  lastRunStatus?: string;
  updatedAt: string;
}
```

### 14.3 Active Task View Model

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

---

## 15. Target Architecture

```text
Delyx Next
├── Workspace Manager
│   ├── projects
│   ├── approved roots
│   ├── Git state
│   ├── checkpoints
│   └── worktrees later
│
├── Thread Manager
│   ├── task threads
│   ├── conversation state
│   ├── active mode
│   └── linked AgentRuns
│
├── Agent Runtime
│   ├── ExploreAgent
│   ├── PlanAgent
│   ├── BuildAgent
│   ├── ReviewAgent
│   ├── TestAgent
│   └── ResearchAgent
│
├── Permission Engine
│   ├── read policy
│   ├── write policy
│   ├── terminal policy
│   ├── network policy
│   ├── memory policy
│   └── automation policy
│
├── Tool Layer
│   ├── file read/search
│   ├── patch proposal
│   ├── patch apply
│   ├── terminal
│   ├── test runner
│   ├── Git
│   ├── Python sandbox
│   └── external agent bridge
│
├── Model Layer
│   ├── mock provider
│   ├── local provider
│   ├── Ollama provider
│   ├── OpenAI-compatible provider
│   ├── role routing
│   └── health checks
│
├── Evidence Layer
│   ├── diffs
│   ├── test output
│   ├── source receipts
│   ├── terminal logs
│   ├── file hashes
│   └── final answer support
│
└── UI
    ├── Project sidebar
    ├── Thread view
    ├── Plan panel
    ├── Diff panel
    ├── Test panel
    ├── Approval drawer
    ├── Evidence panel
    ├── Terminal panel
    ├── Run inspector
    └── Control Center
```

---

## 16. Core Data Models

### 16.1 Project

```ts
interface Project {
  id: string;
  name: string;
  path: string;
  approvedRoots: string[];
  git?: {
    isRepo: boolean;
    branch?: string;
    hasUncommittedChanges: boolean;
    remoteUrl?: string;
  };
  rulesFiles: ProjectRulesFile[];
  modelProfileId?: string;
  createdAt: string;
  updatedAt: string;
}
```

### 16.2 Task Thread

```ts
interface TaskThread {
  id: string;
  projectId: string;
  title: string;
  goal: string;
  status: TaskStatus;
  mode: "explore" | "plan" | "build" | "review" | "test" | "research";
  activeRunId?: string;
  runIds: string[];
  createdAt: string;
  updatedAt: string;
}
```

### 16.3 AgentRun

```ts
interface AgentRun {
  id: string;
  projectId?: string;
  threadId?: string;
  parentRunId?: string;
  goal: string;
  mode: "explore" | "plan" | "build" | "review" | "test" | "research" | "automation";
  status:
    | "created"
    | "running"
    | "waiting_for_approval"
    | "blocked"
    | "repairing"
    | "succeeded"
    | "failed"
    | "cancelled";
  nodes: AgentNode[];
  events: AgentEvent[];
  artifacts: Artifact[];
  evidence: EvidenceRecord[];
  metrics: RunMetrics;
  outcome?: AgentOutcome;
  createdAt: string;
  updatedAt: string;
}
```

### 16.4 AgentNode

```ts
type AgentNodeKind =
  | "classify"
  | "explore"
  | "plan"
  | "model_call"
  | "tool_proposal"
  | "wait_for_approval"
  | "tool_execution"
  | "patch_proposal"
  | "diff_review"
  | "test_execution"
  | "verify"
  | "repair"
  | "answer"
  | "memory_candidate"
  | "external_agent"
  | "done"
  | "blocked";

interface AgentNode {
  id: string;
  runId: string;
  kind: AgentNodeKind;
  status: "pending" | "running" | "waiting" | "succeeded" | "failed" | "skipped";
  dependsOn: string[];
  input: unknown;
  output?: unknown;
  error?: string;
  startedAt?: string;
  completedAt?: string;
}
```

### 16.5 Approval Proposal

```ts
interface ActionProposal {
  id: string;
  runId: string;
  nodeId: string;
  actionType:
    | "read_file"
    | "write_file"
    | "edit_file"
    | "run_terminal"
    | "install_dependency"
    | "save_memory"
    | "use_connector"
    | "schedule_work"
    | "external_send"
    | "external_agent";
  riskLabel: "low" | "medium" | "high" | "dangerous";
  requiredPermission: string;
  rationale: string;
  expectedResult: string;
  rollbackPlan?: string;
  scope: PermissionScope;
  expiresAt: string;
  status: "pending" | "approved" | "denied" | "expired";
}
```

### 16.6 Evidence Record

```ts
interface EvidenceRecord {
  id: string;
  runId: string;
  sourceKind:
    | "local_file"
    | "repo_symbol"
    | "terminal"
    | "test"
    | "diff"
    | "web"
    | "memory"
    | "external_agent"
    | "model_call";
  sourceId: string;
  title?: string;
  uri?: string;
  quote?: string;
  hash?: string;
  retrievedAt: string;
  relevance?: {
    relationship:
      | "direct_implementation"
      | "caller"
      | "test"
      | "config"
      | "doc"
      | "name_only"
      | "unknown";
    score: number;
    reason: string;
  };
}
```

### 16.7 Test Run Artifact

```ts
interface TestRunArtifact {
  id: string;
  runId: string;
  command: string;
  cwd: string;
  exitCode: number | null;
  durationMs: number;
  stdout: string;
  stderr: string;
  parsedFailures?: ParsedFailure[];
  startedAt: string;
  completedAt: string;
  approvalId?: string;
}
```

---

## 17. Repository Structure

```text
delyx-next/
  AGENTS.md
  README.md
  package.json
  tsconfig.json
  vite.config.ts

  apps/
    desktop/
      src/
        app/
        design-system/
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
        lib/
        styles/
      src-tauri/
        Cargo.toml
        tauri.conf.json
        src/
          main.rs
          lib.rs
          commands/
          services/
          models/

    mobile-companion/
      README.md
      package.json
      src/
      android/

  crates/
    delyx-core/
      src/
        project.rs
        thread.rs
        agent_run.rs
        events.rs
        approvals.rs
        artifacts.rs
        evidence.rs
        tool_policy.rs
        errors.rs

    delyx-runtime/
      src/
        executor.rs
        node.rs
        scheduler.rs
        resume.rs
        repair.rs
        modes.rs
        hooks.rs

    delyx-store/
      src/
        sqlite.rs
        migrations/
        repositories/

    delyx-models/
      src/
        provider.rs
        mock.rs
        ollama.rs
        openai_compatible.rs
        roles.rs

    delyx-tools/
      src/
        registry.rs
        file_tools.rs
        terminal_tools.rs
        git_tools.rs
        patch_tools.rs
        test_runner.rs
        python_sandbox.rs
        external_agent_bridge.rs

    delyx-research/
      src/
        retrieval.rs
        evidence_bank.rs
        claim_audit.rs
        contradiction.rs
        citations.rs

    delyx-code-workbench/
      src/
        explorer.rs
        planner.rs
        patch.rs
        verifier.rs
        review.rs
        lsp.rs
        repair.rs

    delyx-eval/
      src/
        cases.rs
        runner.rs
        graders.rs
        analyzer.rs

  docs/
    PRODUCT_DIRECTION.md
    UI_PRINCIPLES.md
    UI_ARCHITECTURE.md
    CODE_WORKBENCH.md
    EXTERNAL_AGENT_BRIDGE.md
    MIGRATION_FROM_DELYX.md
    ARCHITECTURE.md
    SAFETY_PRIVACY_AND_LOCAL_DATA.md
    REASONING_AND_EVALS.md
    MODEL_PROVIDERS.md
    RELEASE_AND_UPDATE.md

  tests/
    fixtures/
    frontend/
    runtime/
    agent_eval/

  scripts/
    setup-windows.ps1
    run-tests.ps1
    smoke-windows.ps1
    eval-response.mjs
    eval-agentic.mjs
```

Important:

```text
Do not create architecture theater.
Only create modules needed by the current milestone.
Keep the target structure in docs, but do not fill the repo with empty abstractions.
```

---

## 18. Root AGENTS.md

Create this at the repo root.

```md
# AGENTS.md

## Project Mission

Delyx Next is a local-first, UI-first AI workbench.

It combines:
- Delyx local-first safety, approvals, model routing, source receipts, memory, and diagnostics
- Codex-style project/thread/diff workflow
- Claude-Code-style Explore/Plan/Build/Review agent workflow
- first-class desktop UI as the trust layer

The product promise:
- useful local agent behavior
- visible trust boundaries
- explicit approvals for risky actions
- source-backed answers
- test-backed coding claims
- inspectable run timelines
- diff-first code review
- local-first data by default
- no fake certainty

## Hard Rules

1. Do not blindly copy old Delyx architecture.
2. Use the old Delyx repo as reference, spec, eval source, safety-policy source, and salvage pool.
3. Delyx Next must be UI-first from day one.
4. Do not build a backend-first agent shell with a thin UI.
5. Every runtime concept must have a visible user-facing state.
6. Do not execute file writes, terminal commands, connector actions, durable memory saves, scheduled risky actions, or external agents without approval.
7. Do not claim code was tested unless an execution artifact exists.
8. Do not claim source-backed facts unless EvidenceRecords support them.
9. Do not hide failed, blocked, denied, expired, partial, or uncertain states in the UI.
10. Do not weaken validators or tests to make a feature look complete.
11. Do not add broad dependencies without explaining why.
12. Do not add cloud-first assumptions. Local-first is the default.
13. Do not mark a milestone done without tests, trace markers, or analyzer artifacts.
14. Keep source files focused: aim for 300 lines or fewer, split/review around 400 lines, and treat 500 lines as a hard cap unless the file is generated, declarative config, or has a documented exception.

## UI Rules

- The UI is the product and trust layer.
- Default workflow is Project -> Thread -> Explore -> Plan -> Approve -> Build -> Diff -> Test -> Review.
- Control Center is advanced mode, not the default first experience.
- Diffs are first-class.
- Tests are first-class.
- Approvals are first-class.
- Evidence receipts are first-class.
- Empty, loading, error, blocked, failed, and waiting states must be designed.
- Keyboard navigation must work for primary actions.
- Use shared design-system components and tokens.
- Avoid one-off styling.
- New runtime states require matching UI states.

## Safety Rules

- Risky tools must produce an ActionProposal before execution.
- Approvals must include scope, reason, risk, expiration, and rollback plan where applicable.
- External agents never get broader authority than the current Delyx task.
- File edits must be checkpointed or isolated.
- Terminal commands must be captured as artifacts.
- Secrets must never be stored in the repo.

## Implementation Style

- Prefer small PR-sized changes.
- Prefer small, focused files. Extract components, services, or helpers before a source file grows past the file-size budget.
- Keep Rust domain models typed.
- Keep frontend state truthful and boring.
- Add tests before polish.
- Use deterministic fixtures before live-model testing.
- Record architectural decisions in docs/ARCHITECTURE.md.
- When touching grounded answers, report whether directness, grounding, source quality, or usefulness changed.

## Default Validation Commands

Run relevant checks after every milestone:

```bash
npm run typecheck
npm test
npm run build
cargo test --workspace
```

For Tauri-specific changes:

```bash
npm run tauri dev
```

For eval changes:

```bash
npm run eval:response
npm run eval:agentic
```

## Definition of Done

A task is not done until:
- code compiles
- tests pass
- feature behavior has a deterministic test or fixture
- user-visible states are truthful
- docs are updated when behavior changes
- risky actions remain approval-gated
- test claims link to execution artifacts
- UI states exist for success, failure, blocked, waiting, empty, and loading states
```

---

## 19. Build Plan / PR Sequence

### ~~PR 1 — Product Direction and Planning Docs~~

Status: Complete on 2026-06-06.

No app code yet.

Create:

- ~~docs/PRODUCT_DIRECTION.md~~
- ~~docs/UI_PRINCIPLES.md~~
- ~~docs/UI_ARCHITECTURE.md~~
- ~~docs/CODE_WORKBENCH.md~~
- ~~docs/EXTERNAL_AGENT_BRIDGE.md~~
- ~~docs/MIGRATION_FROM_DELYX.md~~
- ~~docs/ARCHITECTURE.md~~
- ~~docs/SAFETY_PRIVACY_AND_LOCAL_DATA.md~~
- ~~AGENTS.md~~

Acceptance:

- ~~Product direction is clear.~~
- ~~UI-first requirement is explicit.~~
- ~~Delyx strengths to preserve are listed.~~
- ~~Codex-style workflow ideas are listed.~~
- ~~Claude-Code-style workflow ideas are listed.~~
- ~~Old Delyx patterns to avoid are listed.~~
- ~~First 12 PRs are defined.~~
- ~~No implementation code yet.~~

---

### ~~PR 2 — App Shell + Design System + Mock UI Prototype~~

Status: Complete on 2026-06-06.

Update: Replaced the initial PR 2 mock shell with the provided `delyx-next.zip`
Cockpit UI handoff as the exact visual reference. The app now uses the Cockpit
tokens, fixed 1440x900 workbench canvas, mode-colored accent system, mode
pipeline hero, review/approval panel, and terminal drawer from the handoff.

Scope:

- ~~Tauri + React + TypeScript + Rust skeleton~~
- ~~App shell with split panes~~
- ~~Left project/thread sidebar~~
- ~~Center active task thread~~
- ~~Right review panel~~
- ~~Bottom terminal/log drawer~~
- ~~Command palette placeholder~~
- ~~Light/dark theme tokens~~
- ~~Reusable design-system components~~
- ~~Mock project data~~
- ~~Mock thread data~~
- ~~Mock plan~~
- ~~Mock approval~~
- ~~Mock diff~~
- ~~Mock test output~~
- ~~Mock evidence receipt~~
- ~~Mock failed state~~
- ~~Mock blocked state~~
- ~~Mock external agent transcript~~

Acceptance:

- ~~npm run typecheck passes~~
- ~~npm test passes~~
- ~~npm run build passes~~
- ~~cargo test --workspace passes~~
- ~~app opens to realistic Delyx workbench UI~~
- ~~user can click Project -> Thread -> Plan -> Approval -> Diff -> Test -> Evidence using mock data~~
- ~~blocked, failed, waiting_for_approval, testing, and done states are visible~~

Important:

```text
Do not build blank placeholder pages.
The app should look like the real product with mock data.
```

---

### ~~PR 3 — Workspace Manager Wired to UI~~

Status: Complete on 2026-06-06.

Update: Added the typed Rust workspace manager, approved-root enforcement,
rules-file detection, Git metadata, read-only file indexing/search, and a
workspace manager overlay wired into the Cockpit UI. The UI includes ready,
empty, loading, error, and denied states while keeping file edits and terminal
execution out of scope for this PR.

Scope:

- ~~Add project model~~
- ~~Add approved workspace roots~~
- ~~Add add/remove project flow~~
- ~~Add project health card~~
- ~~Add Git detection~~
- ~~Add read-only file indexing/search~~
- ~~Wire real projects into sidebar~~

Acceptance:

- ~~User can add a project folder.~~
- ~~Project appears in sidebar.~~
- ~~Git status is shown if applicable.~~
- ~~Workspace scope is visible.~~
- ~~Read outside approved root is denied.~~
- ~~UI has empty/error/loading states.~~

---

### ~~PR 4 — Thread Manager Wired to UI~~

Status: Complete on 2026-06-06.

Update: Added a typed thread manager with project linkage, create/list/update,
archive behavior, conversation messages, and guarded status transitions. The
Cockpit sidebar and active thread view now read from thread state, and the
thread manager overlay exposes create, archive, empty, idle, active, blocked,
failed, and done states.

Scope:

- ~~Add TaskThread model~~
- ~~Create/list/update threads~~
- ~~Thread status transitions~~
- ~~Conversation state~~
- ~~Link threads to projects~~
- ~~Wire thread list and thread view~~

Acceptance:

- ~~User can create a thread inside a project.~~
- ~~Thread appears in thread list.~~
- ~~Thread can show mock or real status.~~
- ~~UI clearly shows idle, active, blocked, failed, and done states.~~

---

### ~~PR 4.5 — Remove Demo Data From UI~~

Status: Complete on 2026-06-06.

Update: Removed rendered demo threads, fake runtime content, fake approvals,
fake diffs, fake test output, fake evidence, and fake terminal output. The
workbench now opens on real local project/workspace facts plus honest empty
states for runtime surfaces that are not wired yet.

Scope:

- ~~Remove seeded demo threads from rendered UI~~
- ~~Replace fake plan, approval, diff, test, evidence, and terminal content~~
- ~~Keep real local workspace/project facts visible~~
- ~~Keep honest empty/not-yet-wired states for runtime surfaces~~
- ~~Allow user-created in-session threads only~~

Acceptance:

- ~~No fake thread cards render on first load.~~
- ~~No fake approvals, diffs, test results, evidence receipts, or terminal output render.~~
- ~~Empty states explain what is not wired yet.~~
- ~~Workspace scope, Git status, and rules files still render from the current local project.~~

---

### ~~PR 5 — Typed AgentRun Ledger~~

Status: Complete on 2026-06-06.

Note: Per the real-data rule added after PR 4, deterministic test fixtures may
create AgentRuns, but the rendered UI must not invent a run. Empty timeline,
approval, diff, test, and evidence states remain visible until real ledger
records exist.

Update: Added typed AgentRun models, nodes, events, artifacts, evidence records,
metrics, outcomes, guarded terminal states, std-only persistence/reload tests,
a SQLite migration schema artifact, command-shaped Rust facades ready for Tauri
macro wiring, and a frontend run view that renders real events when present but
opens empty today.

Scope:

- ~~AgentRun~~
- ~~AgentNode~~
- ~~AgentEvent~~
- ~~Artifact~~
- ~~EvidenceRecord~~
- ~~RunMetrics~~
- ~~AgentOutcome~~
- ~~SQLite migration schema artifact~~
- ~~Tauri command facades:~~
  - ~~create_agent_run~~
  - ~~list_agent_runs~~
  - ~~get_agent_run~~
  - ~~append_agent_event~~
- ~~Run timeline UI wired to real data~~

Acceptance:

- ~~Create run.~~
- ~~Append nodes.~~
- ~~Persist/reload run.~~
- ~~Complete run.~~
- ~~Fail run.~~
- ~~Link run to thread.~~
- ~~Timeline shows real run events when ledger data exists.~~

Architectural rule:

```text
The AgentRun graph is the future execution engine, not just an inspection artifact.
```

---

### ~~PR 6 — Explore and Plan Modes~~

Status: Complete on 2026-06-07.

Update: Added read-only ExploreAgent and PlanAgent domain models, approved
workspace search/read tools, structured ExploreOutput and PlanOutput, plan
decision states, and deterministic tests proving Explore/Plan cannot edit or
run terminal commands. The UI starts empty, then derives a real in-session plan
from the active user-created thread and current workspace index.

Scope:

- ~~ExploreAgent with read/search only~~
- ~~PlanAgent with read/search only~~
- ~~File search tool~~
- ~~File read tool inside approved workspace~~
- ~~Plan output model~~
- ~~Plan panel wired to real output~~

Explore output:

```text
Relevant files
Relevant symbols
Project commands discovered
Risks
Unknowns
```

Plan output:

```text
Goal understanding
Files likely involved
Steps
Risks
Tests to run
Permissions needed
```

Acceptance:

- ~~Explore cannot edit.~~
- ~~Plan cannot edit.~~
- ~~Reads outside workspace fail.~~
- ~~Plan is visible in UI.~~
- ~~User can approve, revise, or cancel plan.~~

---

### ~~PR 7 — Approval Engine~~

Status: Complete on 2026-06-07.

Update: Added ActionProposal, ApprovalDecision, RiskyAction/RiskLevel taxonomy,
approve/deny/expire flows, waiting/ready/blocked gate states, and execution
guards proving risky actions cannot execute unless approved and unexpired. The
approval drawer now renders from the real proposal store, which opens empty.

Scope:

- ~~ActionProposal model~~
- ~~ApprovalDecision model~~
- ~~Risk taxonomy~~
- ~~Approve/deny/expire flow~~
- ~~Approval drawer wired to real proposals~~
- ~~Run gate transitions:~~
  - ~~waiting_for_approval~~
  - ~~blocked~~
- ~~Tests proving risky actions cannot execute without approval~~

Risky actions:

```text
file write/edit
terminal command
dependency install
connector send/write
durable memory save
scheduled risky action
external agent execution
external send
```

Acceptance:

- ~~Risky action creates approval proposal.~~
- ~~Approval changes proposal status.~~
- ~~Denial blocks node.~~
- ~~Expired proposal blocks node.~~
- ~~UI shows risk, scope, reason, expected result, and expiration when real proposals exist.~~

---

### ~~PR 8 — Patch Proposal and Checkpoints~~

Status: Complete on 2026-06-07.

Update: Added a typed patch proposal/checkpoint engine, approval-gated patch
apply and checkpoint restore, deterministic diff artifacts, approved-root path
enforcement, and a real empty patch proposal store for the review diff panel.
The UI remains honest on first load and only renders diff files when real
PatchProposalView records exist.

Scope:

- ~~Patch proposal tool~~
- ~~Diff artifact~~
- ~~Checkpoint creation before edits~~
- ~~Apply approved patch~~
- ~~Restore checkpoint~~
- ~~Diff panel wired to real patch data~~

Acceptance:

- ~~Patch can be proposed without applying.~~
- ~~Patch cannot be applied without approval.~~
- ~~Approved patch creates checkpoint.~~
- ~~Revert restores checkpoint.~~
- ~~Diff panel shows changed files and patch.~~

---

### ~~PR 9 — Test Runner Artifacts~~

Status: Complete on 2026-06-07.

Update: Added an approval-gated TestRunner, deterministic test command
detection, TestArtifact records with stdout/stderr/exit code/duration/failure
summary, approved-root working-directory enforcement, and a real empty test
artifact store wired into the review panel. The runtime exposes
has_execution_artifact so tested claims can be gated on captured artifacts.

Scope:

- ~~Approved terminal command runner~~
- ~~Test command detection~~
- ~~Test artifact model~~
- ~~stdout/stderr capture~~
- ~~exit code capture~~
- ~~duration capture~~
- ~~Test panel wired to real artifacts~~

Acceptance:

- ~~Test command requires approval unless policy allows it.~~
- ~~Test output is captured.~~
- ~~Exit code is shown.~~
- ~~Failed tests show parsed failure summary when possible.~~
- ~~Final answer cannot claim tested unless artifact exists.~~

---

### ~~PR 10 — Review Mode~~

Status: Complete on 2026-06-07.

Update: Added a read-only ReviewAgent with explicit read-only capabilities,
deterministic diff/test artifact review, prioritized findings linked to
patch/file/hunk references, failed-test findings, revision requests that mark
the next plan/build flow, and a real empty ReviewReport store wired into the
review panel.

Scope:

- ~~ReviewAgent~~
- ~~Read diff~~
- ~~Produce prioritized findings~~
- ~~No writes by default~~
- ~~Review panel findings UI~~
- ~~Ask-to-revise selected finding~~

Acceptance:

- ~~Review mode does not edit.~~
- ~~Review findings link to files/diff hunks.~~
- ~~Findings are prioritized.~~
- ~~User can ask Delyx to revise, which creates a new plan/build flow.~~

---

### ~~PR 11 — Model Provider Abstraction~~

Status: Complete on 2026-06-07.

Update: Added a local-first model registry with deterministic mock provider
responses, Ollama/OpenAI-compatible health states, role routing persistence,
missing API key status, external-only secret policy, top-bar provider health,
and provider routing/settings UI. No real provider network calls or repo-stored
secrets were added.

Scope:

- ~~Mock provider~~
- ~~Ollama provider health/list models~~
- ~~OpenAI-compatible provider health/list models~~
- ~~Role routing settings~~
- ~~Model status in top bar~~
- ~~Provider settings UI~~

Model roles:

```text
answer
helper
deepResearch
maxReasoning
coding
embedding
scoring
```

Acceptance:

- ~~Mock provider works deterministically.~~
- ~~Provider health appears in UI.~~
- ~~Role routing can be saved.~~
- ~~Missing provider/API key produces clear UI state.~~
- ~~Secrets are not stored in repo.~~

---

### ~~PR 12 — External Agent Bridge Prototype~~

Status: Complete on 2026-06-07.

Update: Added an approval-gated ExternalAgentBridge prototype with Codex CLI
and Claude Code adapter placeholders, a generic terminal-agent prototype
adapter, approved-root/checkpoint scope enforcement, transcript/terminal output
capture, diff-review flags, linked test artifact trust checks, and a truthful
empty external-agent stream in the bottom drawer. No external worker is spawned
yet.

Scope:

- ~~ExternalAgentBridge abstraction~~
- ~~Codex CLI adapter placeholder/detection~~
- ~~Claude Code adapter placeholder/detection~~
- ~~Generic terminal-agent adapter~~
- ~~Run external worker only in approved project/worktree/checkpoint scope~~
- ~~Capture transcript~~
- ~~Capture diff~~
- ~~Capture terminal output~~
- ~~Capture test output if available~~
- ~~Show transcript/diff/output in UI~~

Acceptance:

- ~~External agent cannot run without approval.~~
- ~~External agent scope is visible.~~
- ~~Transcript is captured.~~
- ~~Diffs are reviewed by Delyx UI.~~
- ~~Tests are not trusted unless captured as artifacts.~~
- ~~Delyx remains approval/evidence/control layer.~~

---

### ~~PR 13 — Source-Backed Research MVP~~

Status: Complete on 2026-06-07.

Update: Added a deterministic local ResearchAgent, EvidenceRecord store,
local/repo-first receipt ordering, claim audits, numeric/date support checks,
insufficient-evidence summaries, contradiction records, and receipt rendering
in the Evidence panel. No web retrieval or unsupported source-backed claims
were added.

Scope:

- ~~ResearchAgent~~
- ~~EvidenceRecord store~~
- ~~Local/repo evidence first~~
- ~~Claim audit~~
- ~~Citation/receipt rendering~~
- ~~Contradiction visibility~~

Acceptance:

- ~~Research answer can show evidence receipts.~~
- ~~Missing evidence produces “insufficient evidence.”~~
- ~~Numeric/date claims require support.~~
- ~~Conflicting evidence is shown clearly.~~

---

### ~~PR 14 — Memory Governance~~

Status: Complete on 2026-06-07.

Update: Added project/user memory candidates, approval-gated durable memory
promotion, failed-run promotion blocking, source run/thread tracking,
candidate and memory suppression, supersession behavior, and a real empty
memory review panel.

Scope:

- ~~Project memory~~
- ~~User memory~~
- ~~Memory candidates~~
- ~~Approval before durable save~~
- ~~Source run ID~~
- ~~Supersession/suppression~~
- ~~Memory review UI~~

Acceptance:

- ~~Memory candidate requires approval.~~
- ~~Failed run cannot auto-promote memory.~~
- ~~User can suppress memory.~~
- ~~Memory shows source run/thread.~~

---

### ~~PR 15 — Skills~~

Status: Complete on 2026-06-07.

Update: Added a skill registry that imports/scans skill files, computes source
hashes, marks all imported skills inactive by default, supports
activate/disable/suppress review actions, exposes skill permissions, and blocks
script execution unless the reviewed permission explicitly allows it. The UI now
has a truthful empty skills review panel.

Scope:

- ~~Import skill files~~
- ~~Scan skill~~
- ~~Mark inactive by default~~
- ~~Review/activate/suppress~~
- ~~Skill permissions~~
- ~~Skill source/hash~~

Acceptance:

- ~~Third-party skill never auto-activates.~~
- ~~Skill can be disabled.~~
- ~~Skill permissions are visible.~~
- ~~Skill cannot run scripts unless allowed.~~

---

### ~~PR 16 — Automations / Mission Contracts~~

Status: Complete on 2026-06-07.

Update: Added MissionContract and ScheduledRun models, paused-by-default
contracts, approved activation, scoped allowed tools, active hours/timezone,
delivery targets, stop conditions, workspace drift blocking, approval proposals
for risky scheduled actions, and an empty automation review UI.

Scope:

- ~~MissionContract model~~
- ~~Scheduled run model~~
- ~~Scoped allowed tools~~
- ~~Active hours/timezone~~
- ~~Approval rules~~
- ~~Delivery targets~~
- ~~Automation UI~~

Acceptance:

- ~~Recurring work starts paused until approved.~~
- ~~Contract shows what it can do, when, where, and when it stops.~~
- ~~Workspace drift blocks scheduled work.~~
- ~~Risky scheduled action creates approval instead of executing.~~

---

### ~~PR 17 — Mobile Companion~~

Status: Complete on 2026-06-07.

Update: Added a mobile companion policy/view layer for thread status,
pending approval scope, run status, and low-risk approval decisions bounded by
desktop policy. The UI defaults to an unpaired real state with no file or
terminal access.

Scope:

- ~~Mobile review/steering design~~
- ~~View threads~~
- ~~View pending approvals~~
- ~~Approve/deny limited low-risk approvals if configured~~
- ~~View run status~~
- ~~No broad file/terminal access by default~~

Acceptance:

- ~~Mobile cannot grant broader permissions than desktop policy.~~
- ~~Mobile approval scope is visible.~~
- ~~Mobile can review status without running full agent runtime.~~

---

### ~~PR 18 — Packaging and Release~~

Status: Complete on 2026-06-07.

Update: Added real Tauri runtime packaging, Windows NSIS unsigned dev target,
release smoke checks, explicit unsigned signing validation, support bundle
redaction, a disabled update metadata placeholder, a Windows icon resource, and
release readiness UI. Produced the unsigned installer at
`target/release/bundle/nsis/Delyx Next_0.0.0_x64-setup.exe`.

Scope:

- ~~Windows-first build~~
- ~~Unsigned dev installer~~
- ~~Release smoke~~
- ~~Signing checks~~
- ~~Support bundle export~~
- ~~Update metadata placeholder~~

Acceptance:

- ~~Windows dev build works~~
- ~~Release smoke passes~~
- ~~Signing checks are clear~~
- ~~Support bundle exports logs/config summary without secrets~~

---

## 20. Eval and Testing Strategy

### 20.1 Required Commands

Status: Verified on 2026-06-07.

- ~~npm run typecheck~~
- ~~npm test~~
- ~~npm run build~~
- ~~cargo test --workspace~~

Later commands implemented and verified on 2026-06-07:

- ~~npm run smoke:ui~~
- ~~npm run smoke:tauri~~
- ~~npm run eval:response~~
- ~~npm run eval:agentic~~

### 20.2 Required Seed Test Suites

Status: Seed coverage implemented with deterministic smoke scripts, eval
fixtures, and Rust unit tests.

- ~~UI mock workflow~~
- ~~Workspace policy~~
- ~~Thread state transitions~~
- ~~AgentRun persistence~~
- ~~Approval policy~~
- ~~Patch proposal/checkpoint~~
- ~~Test artifact capture~~
- ~~Review mode read-only behavior~~
- ~~Provider mock behavior~~
- ~~External agent bridge permission behavior~~
- ~~Research evidence behavior~~
- ~~Memory governance behavior~~

### 20.3 Eval Case Shape

Status: Implemented as deterministic `response-cases.json` and
`agentic-cases.json` fixtures.

```json
{
  "id": "code_workbench_plan_001",
  "mode": "plan",
  "prompt": "Add a setting to disable network access for tests.",
  "fixtures": ["sample-tauri-project"],
  "expect": {
    "mustInclude": ["approval", "test", "settings"],
    "mustNotInclude": ["applied patch without approval"],
    "requiredTraceMarkers": ["explore", "plan", "no_write"],
    "approvalExpectations": ["no risky action executed"]
  }
}
```

### 20.4 UI Acceptance Tests

Status: Implemented as `npm run smoke:ui` against built desktop assets.

- ~~Project -> Thread -> Plan -> Approval -> Diff -> Test -> Evidence markers are present in the built UI.~~
- ~~Blocked, failed, done, waiting, empty, loading, and error states have deterministic UI markers.~~
- ~~The first-run workbench uses real local project facts plus honest empty states, not seeded demo runtime data.~~
- ~~No major runtime surface renders blank when no ledger data exists.~~

---

## 21. External Agent Bridge Design

Status: Bridge contract strengthened on 2026-06-07. The generic terminal
adapter now supports one approval-gated `terminal_command` launch inside
checkpoint/worktree isolation with captured stdout, stderr, exit status, and
failed-artifact visibility. Codex CLI and Claude Code remain detection-only
until their command contracts are added.

### 21.1 Purpose

Delyx should be able to launch external agents as controlled workers.

Examples:

```text
Codex CLI
Claude Code
generic terminal agent
```

Delyx does not surrender control.

Delyx provides:

- ~~Scope~~
- ~~Permissions~~
- ~~Worktree/checkpoint~~
- ~~Prompt/task~~
- ~~Timeout~~
- ~~Allowed tools~~
- ~~Transcript capture~~
- ~~Diff capture~~
- ~~Test artifact capture~~
- ~~Review UI~~
- ~~Accept/revert control~~
- ~~Generic terminal worker command execution when `terminal_command` is explicitly allowed by task policy~~
- ~~Failed worker commands are captured as failed artifacts instead of hidden errors~~

### 21.2 Rule

```text
~~External agents never get more permission than Delyx gives the task.~~
```

### 21.3 External Worker Flow

```text
User starts task
→ Delyx creates thread/run
→ Delyx creates checkpoint/worktree
→ Delyx proposes external worker action
→ user approves
→ Delyx launches worker in approved scope
→ transcript is streamed/captured
→ changed files are captured as diff
→ tests are captured if run
→ Delyx shows review panel
→ user accepts/reverts/continues
```

### 21.4 Adapter Interface

```ts
interface ExternalAgentAdapter {
  id: string;
  displayName: string;
  detect(): Promise<ExternalAgentAvailability>;
  run(request: ExternalAgentRunRequest): AsyncIterable<ExternalAgentEvent>;
  stop(runId: string): Promise<void>;
}
```

### 21.5 Captured Events

Status: Implemented as timestamped Rust/TypeScript event variants for the
prototype bridge.

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

---

## 22. Code Workbench Modes

Status: Implemented on 2026-06-07 with typed Explore/Plan outputs,
Build/Test mode gates, patch checkpoints, test artifacts, and read-only Review
mode tests.

### 22.1 Explore Mode

Purpose:

```text
Understand the project without changing anything.
```

Allowed:

- ~~List files~~
- ~~Search files~~
- ~~Read files~~
- ~~Inspect Git status~~
- ~~Read rules files~~

Not allowed:

- ~~Edits~~
- ~~Writes~~
- ~~Terminal commands unless explicitly approved~~
- ~~External agents~~

Output:

- ~~Relevant files~~
- ~~Relevant symbols~~
- ~~Architecture summary~~
- ~~Unknowns~~
- ~~Suggested next steps~~

### 22.2 Plan Mode

Purpose:

```text
Turn exploration into an implementation plan.
```

Output:

- ~~Goal understanding~~
- ~~Files likely to change~~
- ~~Step-by-step plan~~
- ~~Risks~~
- ~~Tests to run~~
- ~~Permissions needed~~
- ~~Rollback strategy~~

### 22.3 Build Mode

Purpose:

```text
Make approved changes safely.
```

Rules:

- ~~Requires approved plan or direct user instruction.~~
- ~~Creates checkpoint before edit.~~
- ~~Applies patch only after approval, unless project policy allows lower-risk auto-apply.~~
- ~~Captures diff artifact.~~

### 22.4 Test Mode

Purpose:

```text
Run approved verification commands and capture evidence.
```

Rules:

- ~~Commands require approval unless already allowed by project policy.~~
- ~~Capture stdout/stderr/exit code/duration.~~
- ~~Link artifact to run.~~
- ~~Do not claim tested without artifact.~~

### 22.5 Review Mode

Purpose:

```text
Review changes without editing by default.
```

Output:

- ~~Prioritized findings~~
- ~~File/diff references~~
- ~~Risk labels~~
- ~~Suggested fixes~~

---

## 23. Safety and Permission Model

Status: Approval records, execution gates, run/node linkage, and per-action
risk floors implemented on 2026-06-07. Future tools must extend the same typed
taxonomy instead of inventing one-off risk labels.

### 23.1 Risk Taxonomy

- ~~Risk levels have typed ordering: low < medium < high < dangerous.~~
- ~~Each risky action has a typed taxonomy entry with minimum risk, summary, and rollback requirement.~~
- ~~Approval proposals clamp requested risk to the action minimum so callers cannot downgrade risky work.~~
- ~~The approval UI shows the active risk taxonomy policy when proposals are empty or present.~~

```text
low:
- read project metadata
- read already-approved workspace files

medium:
- broad file reads
- memory save proposal
- connector read

high:
- file write/edit
- dependency install
- connector write
- external send
- external agent execution

dangerous:
- shell commands with destructive potential
- broad filesystem access
- credential-related operations
- networked commands when restricted
- scheduled/headless risky action
```

### 23.2 Approval Requirements

Approval must include:

- ~~action type~~
- ~~risk label~~
- ~~scope~~
- ~~reason~~
- ~~expected result~~
- ~~rollback plan if applicable~~
- ~~expiration~~
- ~~run ID~~
- ~~node ID~~

### 23.3 Tool Execution Rule

- ~~No file write, terminal command, connector send, durable memory save, scheduled risky action, or external agent run can execute without an approval record linked to the active AgentRun.~~

---

## 24. Codex Master Prompt

Paste this into Codex first.

```md
I want to build Delyx Next as a clean new app.

Reference repo:
https://github.com/joshua-ivy/Delyx

Product target:
Delyx Next should combine:
- Delyx local-first safety, approvals, model routing, source receipts, memory, and diagnostics
- Codex-style project/thread workflow, worktree/checkpoint isolation, diff review, Git-aware flow
- Claude-Code-style terminal/codebase task workflow, Explore/Plan/Build/Review modes, hooks, memory files, and subagents
- first-class desktop UI from day one

Critical requirement:
Delyx Next is not a backend-first agent shell with a thin frontend.
The UI is the product and the trust layer.

Default workflow:
Project
→ Thread
→ Explore
→ Plan
→ Approve
→ Build
→ Diff
→ Test
→ Review
→ Accept / Revert / Continue

Main layout:
- left sidebar: Projects, Threads, Skills, Automations, Memory, Settings
- center panel: Active task thread, chat, plan, step timeline
- right panel: Review panel with diff, tests, approvals, evidence
- bottom drawer: Terminal, logs, external agent transcript
- advanced mode: Control Center and Run Inspector

Keep these Delyx strengths:
- local-first desktop app
- Tauri + React + TypeScript + Rust + SQLite unless a decision record proves otherwise
- visible approval boundaries
- AgentRun ledger
- source-backed research
- coding lane
- Python sandbox evidence
- model/runtime routing
- local memory with approval
- Control Center for advanced inspection
- mobile companion later
- automations later

Borrow these Codex-style ideas:
- project/thread sidebar
- parallel task threads
- worktree/checkpoint isolation
- diff-first review
- Git-aware flow later
- review-only mode
- skills as reusable workflows
- automations later
- mobile steering/approval later

Borrow these Claude-Code-style ideas:
- terminal/codebase task feel
- explicit Explore, Plan, Build, Review modes
- read-only exploration agents
- hooks around tool use and permissions
- project memory files like DELYX.md / AGENTS.md / CLAUDE.md
- subagents with restricted tools
- resumable sessions
- checkpointing

Fix these Delyx weaknesses:
- too much cockpit complexity too early
- no phrase-list-only routing as the core reasoning system
- no string-shape validators pretending to prove reasoning
- no symbol-name overlap treated as real evidence
- no risky tool execution without approval
- no tested-code claims without execution artifacts
- no UI hiding failed, blocked, expired, denied, partial, or uncertain states

First task:
Create docs only. Do not implement app code yet.

Produce:

1. docs/PRODUCT_DIRECTION.md
2. docs/UI_PRINCIPLES.md
3. docs/UI_ARCHITECTURE.md
4. docs/CODE_WORKBENCH.md
5. docs/EXTERNAL_AGENT_BRIDGE.md
6. docs/MIGRATION_FROM_DELYX.md
7. docs/ARCHITECTURE.md
8. docs/SAFETY_PRIVACY_AND_LOCAL_DATA.md
9. root AGENTS.md

The docs must define:
- Delyx Next as Delyx + Codex simplicity + Claude Code workflow
- first-class UI as a non-negotiable pillar
- Simple Mode vs Advanced/Cockpit Mode
- default user flow
- UI layout
- design system
- mock-data prototype strategy
- Workspace Manager
- Thread Manager
- Agent Runtime
- Permission Engine
- Tool Layer
- Model Layer
- Evidence Layer
- External Agent Bridge
- Code Workbench modes
- approval rules
- tested-code evidence rules
- first 12 PRs
- non-goals

Important:
Plan PR 2 as App Shell + Design System + Mock UI Prototype.
The app should look like the real product before the backend is fully implemented.

After docs, stop and summarize:
- the UI product direction
- the first 12 PRs
- which screens are built first
- what mock states are required
- what acceptance tests prove the UI is first-class
```

---

## 25. PR 2 Implementation Prompt

Use this after PR 1 docs are done.

```md
Implement PR 2: App Shell + Design System + Mock UI Prototype.

Follow AGENTS.md and docs/UI_PRINCIPLES.md.

Scope:
- Tauri + React + TypeScript + Rust skeleton
- app shell with split panes
- top bar with project, branch/worktree, model, mode, run status
- left project/thread sidebar
- center active task thread
- right review panel
- bottom terminal/log drawer
- command palette placeholder
- light/dark theme tokens
- reusable design-system components
- mock project data
- mock thread data
- mock plan
- mock approval
- mock diff
- mock test output
- mock evidence receipt
- mock failed state
- mock blocked state
- mock external agent transcript

Do not implement real agent runtime yet.
Do not implement real model providers yet.
Do not build a blank demo shell.

Acceptance:
- npm run typecheck passes
- npm test passes
- npm run build passes
- cargo test --workspace passes
- the app opens to a realistic Delyx workbench UI
- user can click through Project → Thread → Plan → Approval → Diff → Test → Evidence using mock data
- blocked, failed, waiting_for_approval, testing, reviewing, and done states are visible
- all major screens have empty/loading/error states
- keyboard navigation works for primary shell actions
```

---

## 26. PR 3 Implementation Prompt

```md
Implement PR 3: Workspace Manager wired to UI.

Scope:
- Project model
- approved workspace roots
- add project flow
- remove project flow
- project health summary
- Git detection
- read-only file index/search
- rules file detection for DELYX.md, AGENTS.md, CLAUDE.md
- wire real project data into sidebar and project screen

Rules:
- Do not allow reads outside approved roots.
- Do not add file edits yet.
- Do not add terminal execution yet.

Acceptance:
- user can add a project
- project appears in sidebar
- approved scope is visible
- Git status is visible when available
- rules files are detected
- read outside workspace fails
- UI has empty/loading/error/denied states
```

---

## 27. PR 4 Implementation Prompt

```md
Implement PR 4: Thread Manager wired to UI.

Scope:
- TaskThread model
- create/list/update/archive thread
- thread status transitions
- conversation state
- link threads to projects
- wire real data into thread list and thread view

Acceptance:
- user can create a thread inside a project
- thread appears in thread list
- thread can store goal and messages
- thread states render correctly
- empty, failed, blocked, and done states are visible
```

---

## 28. PR 5 Implementation Prompt

```md
Implement PR 5: Typed AgentRun ledger.

Scope:
- AgentRun
- AgentNode
- AgentEvent
- Artifact
- EvidenceRecord
- RunMetrics
- AgentOutcome
- SQLite migrations
- Tauri commands:
  - create_agent_run
  - list_agent_runs
  - get_agent_run
  - append_agent_event
- wire real AgentRun timeline into UI

Do not implement real model calls yet.
Use deterministic mock run creation only.

Architectural rule:
The AgentRun graph is the future execution/resume engine, not just an inspection artifact.

Acceptance:
- create run
- append nodes
- persist/reload run
- complete run
- fail run
- link run to thread
- timeline displays real events
- tests cover persistence and status transitions
```

---

## 29. PR 6 Implementation Prompt

```md
Implement PR 6: Explore and Plan modes.

Scope:
- ExploreAgent with read/search only
- PlanAgent with read/search only
- read approved files
- search approved workspace
- detect relevant files
- generate structured PlanViewModel
- wire plan output into PlanPanel

Do not edit files.
Do not run terminal commands.

Acceptance:
- Explore mode cannot edit
- Plan mode cannot edit
- reads outside approved workspace fail
- plan shows goal, files, steps, risks, tests, permissions
- user can approve, revise, or cancel plan
```

---

## 30. Non-Negotiable Guardrails for Every Codex Task

Paste this into every implementation prompt.

```md
Non-negotiable guardrails:

- Keep diffs narrow.
- Do not weaken tests.
- Do not change safety defaults to make demos easier.
- Do not execute risky local actions without approval.
- Do not store secrets in repo.
- Do not hide failed states in UI.
- Do not hide blocked states in UI.
- Do not hide expired approvals in UI.
- Do not claim tested behavior without test artifacts.
- Do not treat old Delyx code as automatically correct.
- Do not mark complete without tests.
- Update docs when behavior changes.
- Every new runtime state needs a matching UI state.
- Every risky action must be visible in the approval drawer.
- Every code edit must be visible in the review/diff panel.
- Every test claim must link to a test artifact.
- Every final answer should have receipts when evidence exists.
- Keyboard navigation must work for primary actions.
- Components must use consistent tokens, spacing, typography, and status colors.
- Keep source files focused: target 300 lines or fewer, split/review around 400 lines, and use 500 lines as a hard cap unless generated, declarative config, or explicitly documented.
- Avoid one-off styling unless the design system cannot handle it.
```

---

## 31. Definition of Great

Delyx Next is not great when it has a pretty chat UI.

Delyx Next is great when this is true:

- ~~A user can open a project, start a thread, ask Delyx to explore, plan, build, test, or review.~~
- ~~Delyx can do safe read-only work directly.~~
- ~~Delyx pauses before risky actions.~~
- ~~Delyx shows the plan before changing files.~~
- ~~Delyx shows every diff.~~
- ~~Delyx shows whether tests actually ran.~~
- ~~Delyx shows evidence receipts for claims.~~
- ~~Delyx can use external agents without surrendering control.~~
- ~~Delyx can resume after approval.~~
- ~~Delyx can say "not enough evidence" or "not tested" without pretending.~~
- ~~The whole thing feels like a serious first-class desktop workbench.~~

The final identity:

```text
Delyx Next is a local-first AI workbench for coding, research, files, memory, and automations.

It has the simple task-thread workflow of Codex,
the terminal/codebase feel of Claude Code,
and Delyx’s local approvals, source receipts, model routing, memory, and safety cockpit.
```
