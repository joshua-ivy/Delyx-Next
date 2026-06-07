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
Status update: read-only Git index dirty count implemented on 2026-06-07.

- ~~Git UI shows the current `main` branch from real local project facts.~~
- ~~Uncommitted count is not faked; the UI says changes are not loaded until a real dirty-count artifact exists.~~
- ~~Desktop workspace snapshots can replace "changes not loaded" with a read-only Git index dirty count when metadata exists.~~
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
- ~~Edit Plan uses the real revision-request path instead of a not-wired toast.~~
- ~~Ask Question records a local run event without pretending a model call ran.~~
- ~~Read-only review action moves the active thread and run into reviewing mode without edits.~~

### 12.4 Diff Review Panel

MVP:

```text
Changed files
Unified diff
Patch action status
```

Status update: diff review artifact surface implemented on 2026-06-07.

- ~~Diff panel shows changed files and unified diff artifacts.~~
- ~~Diff panel hides patch action controls until executable PatchProposal bindings exist.~~

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
```

Status update: approval drawer card controls implemented on 2026-06-07.

- ~~Approval cards show action, risk, reason, files/commands scope, expected result, rollback plan, expiration, and safe local controls for approve once and deny.~~
- ~~Project-wide allow and scope editing controls stay hidden until they have real policy bindings.~~

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

Status update: UI state contract types implemented on 2026-06-07.

- ~~Task status, thread summary, blocker, review, and active task view-model contracts are represented in shared TypeScript types.~~

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

Status update: frontend TaskThread core model fields implemented on 2026-06-07.

- ~~TaskThread carries mode, activeRunId, runIds, createdAt, and updatedAt in the local UI model.~~

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

Status update: frontend AgentRun core model fields implemented on 2026-06-07.

- ~~AgentRun UI model carries project/thread links, parent run, goal, mode, full lifecycle status, nodes, events, artifacts, evidence, metrics, outcome, createdAt, and updatedAt.~~

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

Status update: frontend AgentNode core model fields implemented on 2026-06-07.

- ~~AgentNode UI model uses typed kind/status, runId, dependsOn, input/output/error, startedAt, and completedAt.~~

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

Status update: frontend ActionProposal core model fields implemented on 2026-06-07.

- ~~ActionProposal UI model uses actionType, riskLabel, requiredPermission, rationale, PermissionScope, expiration, status, and optional rollback plan.~~

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

Status update: frontend EvidenceRecord is shared by AgentRun and research receipts as of 2026-06-07.

- ~~Evidence receipts use the shared EvidenceRecord fields: sourceKind, sourceId, title, uri, quote, hash, retrievedAt, and relevance.~~

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

Status update: frontend TestRunArtifact core model fields implemented on 2026-06-07.

- ~~TestRunArtifact UI model uses command, cwd, exitCode, durationMs, stdout, stderr, parsedFailures, startedAt, completedAt, and optional approvalId.~~

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

Status update: AppShell cockpit DOM bindings were extracted into a focused hook on 2026-06-07 to preserve the source file budget.

- ~~AppShell remains under the 300-line target while cockpit DOM bindings live in a focused hook.~~
- ~~Plan-action DOM bindings are split into a focused module before the cockpit hook approaches the 300-line target.~~
- ~~Unused generic safe-action no-op binder is removed after controls become stateful or unavailable.~~
- ~~External-agent path-scope checks are split into a focused helper module before the bridge file approaches the 300-line target.~~
- ~~External-agent terminal command tests are split into a focused module to keep test files under the verifier line budget.~~
- ~~External-agent adapter defaults are split into a focused helper before the bridge file hits the verifier line cap.~~

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

Status update: root AGENTS.md exists and matches the active project rules as of 2026-06-07.

- ~~Root AGENTS.md is present with project mission, hard rules, UI rules, safety rules, implementation style, validation commands, and Definition of Done.~~

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

Update: Replaced the initial PR 2 mock shell with the provided Command Deck
handoff as the exact visual reference. The app now uses the shared mode-tinted
tokens, spine, command bar, work pane, contextual inspector, pinned composer,
and hint bar from the handoff without copying the prototype scenario data.

Scope:

- ~~Tauri + React + TypeScript + Rust skeleton~~
- ~~App shell with split panes~~
- ~~Mode spine with project/thread entry points~~
- ~~Work pane with active task thread~~
- ~~Contextual inspector for approvals, diffs, tests, review, and receipts~~
- ~~Terminal/log controls inside the work pane without fake command history~~
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
- ~~user can click Project -> Thread -> Plan -> Approval -> Diff -> Test -> Evidence using real in-session state or honest empty states~~
- ~~blocked, failed, waiting_for_approval, testing, and done states are visible~~
- ~~Command Deck shell uses real local state or truthful empty states only; the prototype scenario data is not rendered.~~

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
- ~~Workspace manager last-run status reads the current in-session AgentRun state instead of a static empty ledger.~~
- ~~Workspace file indexing skips symlink entries so read-only discovery cannot walk outside approved roots.~~
- ~~Rules file detection skips symlinked rule files so external rules cannot masquerade as approved workspace rules.~~

---

### ~~PR 3.1 - Tauri Workspace Snapshot MVP~~

Status: Complete on 2026-06-07.

Update: Added a read-only Tauri `workspace_snapshot` command and renderer
bridge. The desktop app now loads real approved-root metadata, rules files, Git
branch, and indexed file names from the selected local path. Browser preview no
longer carries a stale hardcoded file index and instead shows honest unloaded
workspace search state until the Rust bridge is available.

Scope:

- ~~Expose workspace metadata through a typed Tauri command.~~
- ~~Index file names under the approved root without reading arbitrary file contents.~~
- ~~Load rules files and Git branch from the Rust workspace manager.~~
- ~~Remove static indexed-file seed data from the default frontend project.~~
- ~~Keep Git dirty counts unknown until a real dirty-count artifact exists.~~

Acceptance:

- ~~Workspace search does not render stale demo file lists.~~
- ~~The desktop bridge loads real indexed file names from the approved project path.~~
- ~~Browser preview shows an honest unloaded workspace state.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 3.2 - Remove Simulated Workspace State Controls~~

Status: Complete on 2026-06-07.

Update: Removed command-palette entries and workspace-panel buttons that
manually simulated loading, error, or denied workspace states for acceptance
checks. The UI still defines those states, but they are now driven by real
workspace bridge load, add-project, remove-project, or policy outcomes.

Scope:

- ~~Remove fake workspace loading/error command-palette actions.~~
- ~~Remove fake denied/loading/error workspace buttons.~~
- ~~Keep truthful loading, error, denied, empty, and ready state copy.~~
- ~~Show read-policy facts instead of simulation controls.~~

Acceptance:

- ~~Users do not see fake workspace-state actions.~~
- ~~Workspace states remain present for real runtime outcomes.~~
- ~~Verifier and smoke checks require truthful state copy.~~

---

### ~~PR 3.3 - Read-Only Git Dirty Count~~

Status: Complete on 2026-06-07.

Update: Added a std-only Git index reader for the workspace manager. Desktop
workspace snapshots now report a conservative dirty-file count when `.git/index`
metadata is present, without launching `git` or any hidden terminal command.
If the index is missing or unsupported, the UI keeps the honest "changes not
loaded" state.

Scope:

- ~~Read Git branch and dirty count from local Git metadata only.~~
- ~~Count modified, deleted, and untracked workspace files from approved-root metadata.~~
- ~~Keep dirty count unknown when Git index metadata is missing or unsupported.~~
- ~~Show whether the workspace dirty count came from read-only Git index metadata.~~
- ~~Cover clean, dirty, and missing-index cases with deterministic Rust tests.~~

Acceptance:

- ~~Workspace snapshot can expose a real dirty count without running a terminal command.~~
- ~~Browser fallback still stays honest when no Rust workspace snapshot is available.~~
- ~~Verifier covers the Git index reader and UI policy copy.~~
- ~~Source files stay within the line-budget rule.~~

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
- ~~Thread creation starts with an empty user-goal input instead of a seeded fixture goal.~~
- ~~No fake approvals, diffs, test results, evidence receipts, or terminal output render.~~
- ~~Empty states explain what is not wired yet.~~
- ~~Workspace scope, Git status, and rules files still render from the current local project.~~
- ~~Thread empty state is reached through real no-thread or archive flow, not command-palette simulation.~~

---

### ~~PR 4.6 - Remove Simulated Thread Empty-State Controls~~

Status: Complete on 2026-06-07.

Update: Removed the command-palette `Show empty threads` action and the thread
manager `Show empty state` button. Empty thread UI still renders when no active
threads exist, and archiving the last active thread naturally returns the
thread surface to empty.

Scope:

- ~~Remove fake thread empty-state command-palette action.~~
- ~~Remove fake thread empty-state button from the thread manager.~~
- ~~Keep create, select, archive, and status controls tied to real in-session thread records.~~
- ~~Keep empty-state copy visible when no active thread exists.~~

Acceptance:

- ~~Users do not see fake thread-state actions.~~
- ~~No command clears thread/run/plan data purely to simulate acceptance states.~~
- ~~Verifier checks require real empty-thread copy instead of simulator controls.~~

---

### ~~PR 4.7 - Remove Legacy Decorative Thread Controls~~

Status: Complete on 2026-06-07.

Update: Removed unbound decorative buttons from the legacy ThreadView,
PlanPanel, and ReviewPanel source paths. The live command palette now describes
plan approval as queueing a scoped build approval proposal instead of "UI state
only."

Scope:

- ~~Replace legacy PlanPanel approve button with approval-required status.~~
- ~~Replace legacy ThreadView follow-up button with composer guidance status.~~
- ~~Replace legacy waiting-for-approval button with approval-pending status.~~
- ~~Replace legacy ReviewPanel accept/reject/revision buttons with approval-gated patch status.~~
- ~~Remove "UI state only" plan approval copy from command palette messaging.~~

Acceptance:

- ~~No source path renders decorative plan/thread/review buttons without handlers.~~
- ~~Plan approval copy describes the real approval proposal flow.~~
- ~~Verifier forbids the old fake/decorative action labels.~~

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
- ~~User-created in-session threads attach a real local AgentRun entry and set activeRunId/runIds without seeding fake first-run data.~~
- ~~Thread status changes update the attached in-session AgentRun status, mode, updatedAt, metrics, and timeline event.~~
- ~~No-op thread status selections do not create duplicate AgentRun timeline events.~~

Architectural rule:

```text
The AgentRun graph is the future execution engine, not just an inspection artifact.
```

---

### ~~PR 5.1 - Tauri Thread/Run Session Bridge~~

Status: Complete on 2026-06-07.

Update: Added a Tauri-backed thread/run session bridge that creates UI-ready
threads through the Rust ThreadManager, allocates an AgentRun ledger record,
captures a real `thread.created` event, and lets the renderer restore bridge
session threads instead of relying only on React state. Web preview keeps the
existing local fallback.

Scope:

- ~~Tauri command for creating a thread/run session record.~~
- ~~Tauri command for listing current bridge session thread/run records.~~
- ~~Renderer thread creation prefers the bridge and falls back only when the bridge is unavailable.~~
- ~~Composer-created first threads use the same bridge path.~~
- ~~Deterministic Rust tests cover create, list, and empty-goal rejection.~~

Acceptance:

- ~~Created bridge threads are idle/explore and do not claim model or tool execution.~~
- ~~Initial bridge runs are UI-ready `created` records with a real `thread.created` event.~~
- ~~Snapshots only return records for the requested project.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 5.2 - Thread Session Status and Archive Bridge~~

Status: Complete on 2026-06-07.

Update: Extended the Tauri thread/run session bridge so thread status changes
and archive actions update the Rust ThreadManager-backed session state. Bridge
snapshots now map the visible run status from the current thread state, while
web preview keeps the existing local state fallback.

Scope:

- ~~Tauri command for thread status updates.~~
- ~~Tauri command for archiving active threads.~~
- ~~Bridge snapshots hide archived threads from the active list.~~
- ~~Renderer status/archive actions notify the bridge while preserving immediate UI updates.~~
- ~~Split bridge view mapping into a focused file to stay inside the source-size budget.~~

Acceptance:

- ~~Planning/building/testing/reviewing thread states restore with matching run visibility.~~
- ~~Archived bridge threads are removed from active snapshots.~~
- ~~Invalid status keys are rejected by the bridge.~~
- ~~Source files stay within the line-budget rule.~~

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
- ~~Creating a plan moves the active thread and attached in-session AgentRun into planning mode with a timeline event.~~
- ~~Plan controls report truthful warnings instead of silently no-oping when no thread or plan exists.~~

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
- ~~Approving a real in-session plan queues a scoped edit_file ActionProposal and moves the thread/run to waiting_for_approval instead of executing build work.~~
- ~~Approval changes proposal status.~~
- ~~Approval drawer Approve once and Deny controls update in-session proposal status and move the thread/run to building or blocked without executing tools.~~
- ~~Approval drawer hides pending decision controls once a proposal is approved or denied.~~
- ~~Approval drawer decision controls keep keyboard role, tabindex, and labels after becoming stateful controls.~~
- ~~AgentRun timeline records approval proposal and decision receipts without implying a patch or command executed.~~
- ~~Revising or cancelling an approved plan expires stale pending build approvals.~~
- ~~Denial blocks node.~~
- ~~Expired proposal blocks node.~~
- ~~Expired pending proposals render as expired and remove decision controls.~~
- ~~Approval expiration is closed at the exact deadline across UI and Rust gates.~~
- ~~Approved approvals visibly expire after their execution window closes.~~
- ~~UI shows risk, scope, reason, expected result, and expiration when real proposals exist.~~
- ~~Approval policy controls without implementations stay hidden instead of rendering unavailable no-ops.~~
- ~~Execution gates reject approvals for the wrong risky action type.~~
- ~~Execution gates reject approvals from another AgentRun before performing run-scoped work.~~
- ~~Approval decision handlers reject stale, non-pending, or expired proposals before recording decision events.~~

---

### ~~PR 7.1 - Tauri Approval Session Bridge~~

Status: Complete on 2026-06-07.

Update: Desktop approval proposal creation and approve/deny decisions now flow
through a Tauri `ApprovalBridgeState` backed by the Rust `ApprovalEngine`.
The bridge returns the same UI-ready proposal shape as the renderer, dedupes
repeated plan-approval requests by client ID, rejects unsupported non-risky
approval actions, keeps expired decisions visible, and preserves the web
preview fallback. This does not execute file writes, terminal commands,
connectors, durable memory saves, scheduled work, or external agents.

Scope:

- ~~Expose `approval_propose`, `approval_decide`, and `approval_snapshot` Tauri commands.~~
- ~~Map frontend risky-action and risk labels into the Rust ApprovalEngine.~~
- ~~Wire plan approval requests through the bridge before showing the proposal.~~
- ~~Wire approval drawer approve/deny decisions through the bridge before moving thread/run state.~~
- ~~Keep web preview using the renderer fallback when no Tauri runtime exists.~~
- ~~Add deterministic Rust tests and verifier markers.~~

Acceptance:

- ~~Desktop approval proposals are recorded in the Rust approval gate.~~
- ~~Duplicate plan approval clicks do not create duplicate bridge proposals.~~
- ~~Expired approval decisions stay visible as expired instead of moving the run forward.~~
- ~~Unsupported non-risky read approvals are rejected by the bridge.~~
- ~~Source files stay within the line-budget rule.~~

Validation:

- ~~`npm run typecheck` passed.~~
- ~~`cargo test --workspace` passed with 166 Rust tests.~~
- ~~`npm test` passed.~~
- ~~`npm run build` passed.~~
- ~~`npm run smoke:ui` passed.~~
- ~~`npm run smoke:tauri` passed.~~
- ~~`git diff --check` passed with only existing CRLF normalization warnings.~~

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
- ~~Empty diff state hides patch action controls until a real PatchProposal exists.~~
- ~~Patch action controls without execution/state bindings stay hidden instead of rendering unavailable no-ops.~~
- ~~Restored checkpoints cannot be reused to overwrite later local changes.~~
- ~~Patch proposals reject duplicate normalized file paths so diff/checkpoint artifacts stay unambiguous.~~

---

### ~~PR 8.1 - Remove Unimplemented Approval and Diff Controls~~

Status: Complete on 2026-06-07.

Update: Removed disabled future controls from the approval and diff panels.
Approval proposals now show only the bound `Approve once` and `Deny` actions.
Patch review panels show real diff artifacts plus a non-action status note
until executable patch controls exist.

Scope:

- ~~Remove `Always allow later` and `Edit scope` controls until policy bindings exist.~~
- ~~Remove disabled patch action controls until executable PatchProposal bindings exist.~~
- ~~Keep pending approval decisions usable through approve-once and deny controls.~~
- ~~Keep diff panels truthful without advertising unavailable commands.~~

Acceptance:

- ~~No disabled future approval controls render.~~
- ~~No disabled future patch controls render.~~
- ~~Verifier checks require truthful non-action status text.~~

---

### ~~PR 8.2 - Tauri Patch Proposal Bridge~~

Status: Complete on 2026-06-07.

Update: Added a proposal-only Tauri patch bridge that accepts explicit
approved-root file content requests, uses the Rust PatchEngine to build
UI-ready diff records, stores in-session patch proposals by run, and exposes a
frontend client for later Build flow wiring. The bridge never applies writes,
creates checkpoints, or invents patch content.

Scope:

- ~~Expose `patch_propose` and `patch_snapshot` Tauri commands.~~
- ~~Use the Rust PatchEngine to compute diff artifacts from explicit file-after contents.~~
- ~~Reject paths outside approved roots without storing a patch proposal.~~
- ~~Deduplicate repeated bridge requests by client patch ID.~~
- ~~Add a frontend patch client without seeding fake patch data.~~
- ~~Add deterministic Rust tests and verifier markers.~~

Acceptance:

- ~~Patch proposals can cross the Tauri bridge without applying file writes.~~
- ~~Diff records are UI-ready PatchProposalView shapes.~~
- ~~Outside-root patch requests are rejected and remain invisible in snapshots.~~
- ~~First-run UI still shows honest empty diff state until a real patch exists.~~
- ~~Source files stay within the line-budget rule.~~

Validation:

- ~~`npm run typecheck` passed.~~
- ~~`cargo test --workspace` passed with 170 Rust tests.~~
- ~~`npm test` passed.~~
- ~~`npm run build` passed.~~
- ~~`npm run smoke:ui` passed.~~
- ~~`npm run smoke:tauri` passed.~~

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
- ~~Terminal rerun/open/error controls stay disabled until a command artifact exists.~~
- ~~Test command detection rejects shell wrappers that can hide non-test work.~~
- ~~Approved test commands have explicit timeouts and timeout failures do not create test artifacts.~~

---

### ~~PR 9.1 - Tauri Test Runner Bridge~~

Status: Complete on 2026-06-07.

Update: Added an approval-gated Tauri test-runner bridge and frontend client.
The bridge reads the Rust ApprovalEngine owned by the approval bridge before
running any command, uses the Rust TestRunner to reject pending approvals,
wrong actions, non-test commands, timeouts, and outside-root working
directories, then stores UI-ready TestArtifactView records by run. No test UI
execution control was added, so first-run state remains empty until a future
approved Test mode flow calls the bridge.

Scope:

- ~~Expose `test_run_approved` and `test_snapshot` Tauri commands.~~
- ~~Reuse the Rust approval bridge gate before executing a test command.~~
- ~~Return UI-ready test artifacts with stdout, stderr, exit code, duration, failure summary, approval ID, and timestamps.~~
- ~~Reject pending approvals and non-test commands without storing artifacts.~~
- ~~Add a frontend test client without seeding fake test artifacts.~~
- ~~Add deterministic Rust tests and verifier markers.~~

Acceptance:

- ~~Approved test commands can produce TestArtifactView records through the bridge.~~
- ~~Pending approval blocks execution and leaves no artifact.~~
- ~~Non-test commands are rejected before execution.~~
- ~~First-run UI still shows honest empty test state until a real artifact exists.~~
- ~~Source files stay within the line-budget rule.~~

Validation:

- ~~`npm run typecheck` passed.~~
- ~~`cargo test --workspace` passed with 174 Rust tests.~~
- ~~`npm test` passed.~~
- ~~`npm run build` passed.~~
- ~~`npm run smoke:ui` passed.~~
- ~~`npm run smoke:tauri` passed.~~

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

### ~~PR 10.1 - Tauri Review Report Bridge~~

Status: Complete on 2026-06-07.

Update: Added a read-only Tauri review bridge and frontend client. The bridge
accepts real PatchProposalView and TestArtifactView-shaped inputs, converts
them into the Rust ReviewAgent domain model, returns UI-ready ReviewReportView
records, stores snapshots by run, and rejects not-run test artifacts so review
cannot convert missing execution into tested claims. It also rejects patch or
test artifacts from another run before storing a review report. No write, patch
apply, terminal, connector, memory, scheduled-work, or external-agent authority
was added.

Scope:

- ~~Expose `review_create` and `review_snapshot` Tauri commands.~~
- ~~Map patch diff records into the Rust ReviewAgent without adding write capability.~~
- ~~Map test artifacts into failed/passed review inputs and reject not-run artifacts.~~
- ~~Reject patch or test artifacts whose run does not match the review request.~~
- ~~Return UI-ready prioritized findings linked to file paths and hunk labels.~~
- ~~Add a frontend review client without seeding fake ReviewReports.~~
- ~~Add deterministic Rust tests and verifier markers.~~

Acceptance:

- ~~Real patch/test artifacts can produce ReviewReportView records through the bridge.~~
- ~~Review remains read-only and cannot edit.~~
- ~~Not-run test artifacts are rejected before review.~~
- ~~Cross-run patch/test artifacts are rejected before review.~~
- ~~First-run UI still shows honest empty review state until a real report exists.~~
- ~~Source files stay within the line-budget rule.~~

Validation:

- ~~`npm run typecheck` passed.~~
- ~~`cargo test review_bridge_tests --workspace` passed.~~
- ~~`cargo test --workspace` passed with 179 Rust tests.~~
- ~~`npm test` passed, including workbench, Ollama agent, and release smoke verifiers.~~
- ~~`npm run build` passed.~~
- ~~`npm run smoke:ui` passed.~~
- ~~`npm run smoke:tauri` passed with installer artifact `target/release/bundle/nsis/Delyx Next_0.0.0_x64-setup.exe`.~~
- ~~`git diff --check` passed with only Windows line-ending warnings.~~
- ~~Desktop source scan found no `.rs`, `.ts`, `.tsx`, or `.mjs` files over 300 lines.~~

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
- ~~Role routing rejects non-ready providers instead of treating a discovered model list as usable.~~

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
- ~~Missing-key, unconfigured, or unreachable providers cannot be saved as active routes.~~
- ~~First-run model routing UI does not mark missing or mock routes as saved.~~

---

### ~~PR 11.1 - Ollama Composer MVP~~

Status: Complete on 2026-06-07.

Update: Added renderer-side local Ollama discovery and chat for the Command
Deck composer. The app checks `http://127.0.0.1:11434/api/tags`, selects a
ready local Ollama model when one exists, sends real composer messages to
`/api/chat` with `stream: false`, appends the returned assistant message, and
records a `model_call` node/event/artifact/evidence marker in the active
AgentRun. If Ollama is not reachable or no model is installed, the UI records a
truthful system message and a failed model-call event instead of pretending.

Scope:

- ~~Discover local Ollama models from `/api/tags`.~~
- ~~Show selected Ollama provider/model in Command Deck context.~~
- ~~Refresh Ollama models from the command palette.~~
- ~~Send composer messages to `/api/chat`.~~
- ~~Append only real assistant responses from Ollama.~~
- ~~Record model-call execution artifacts in the AgentRun ledger.~~
- ~~Show blocked/unavailable state when Ollama is not ready.~~

Acceptance:

- ~~No prototype scenario or canned assistant text is rendered.~~
- ~~Composer does not claim a model reply unless Ollama returns one.~~
- ~~Ollama failures are visible in the thread and run timeline.~~
- ~~Provider/model state remains local-first and stores no secrets.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.2 - Tauri Runtime Bridge MVP~~

Status: Complete on 2026-06-07.

Update: Added the first read-only Tauri command bridge for the UI runtime.
The desktop backend now exposes app identity, packaged milestone, configured
model providers, and coding-route status through `runtime_status`. The renderer
invokes that command when running inside Tauri and falls back to an honest
web-preview state when the Rust bridge is unavailable.

Scope:

- ~~Expose read-only runtime status through a typed Tauri command.~~
- ~~Serialize Rust bridge views in the camelCase shape expected by TypeScript.~~
- ~~Show Rust/web bridge availability in the Command Deck context chips.~~
- ~~Keep the bridge read-only; no tools, files, terminal commands, memory saves, or external agents execute from this command.~~
- ~~Add deterministic verifier and Rust unit-test coverage for the bridge.~~

Acceptance:

- ~~The UI has a visible runtime bridge state instead of hiding desktop/web differences.~~
- ~~The bridge reports provider and coding-route status without storing secrets.~~
- ~~The command is registered through Tauri and covered by Rust tests.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.3 - Ollama PlanAgent MVP~~

Status: Complete on 2026-06-07.

Update: The active Create Plan flow now asks a ready local Ollama coding route
to draft a read-only PlanView. Delyx accepts only structured JSON, filters
model file references to the approved workspace index, records successful and
failed model calls in the AgentRun ledger, and shows blocked state when Ollama
is unavailable or returns unusable output. No file writes, terminal commands,
connector actions, memory saves, scheduled work, or external agents execute
from this plan draft.

Scope:

- ~~Add a PlanAgent prompt for local Ollama with explicit no-fake/test-claim rules.~~
- ~~Parse Ollama JSON into the typed PlanView and ExploreView contracts.~~
- ~~Reject model file references outside the approved indexed workspace files.~~
- ~~Route cockpit and command-palette Create Plan controls through the same Ollama PlanAgent path.~~
- ~~Record model-call success/failure artifacts, evidence, nodes, and events in AgentRun.~~
- ~~Add deterministic fixture coverage for PlanAgent parsing and file filtering.~~

Acceptance:

- ~~Create Plan does not claim a model-authored plan unless Ollama returns parseable JSON.~~
- ~~Unavailable Ollama or invalid model output is visible as a blocked thread state.~~
- ~~A successful plan draft keeps risky build actions approval-gated.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.4 - Remove Frontend Mock Model Route~~

Status: Complete on 2026-06-07.

Update: The frontend no longer selects the deterministic mock provider or
`delyx-mock-coder` as the active user-facing model route. The default model
state now shows Ollama as not configured with no saved route until real local
discovery succeeds. Mock provider behavior remains available in backend tests
and deterministic fixtures only.

Scope:

- ~~Default frontend model settings to Ollama instead of the deterministic mock provider.~~
- ~~Remove the saved mock coding route from user-facing frontend state.~~
- ~~Keep the model context chip honest when no ready Ollama route exists.~~
- ~~Ensure workspace model profile labels only show routes for the selected provider.~~
- ~~Make the verifier forbid frontend mock-coder route strings.~~

Acceptance:

- ~~First-run UI does not present `delyx-mock-coder` as a usable model.~~
- ~~Ollama readiness still promotes a real discovered model into the coding route.~~
- ~~Backend deterministic mock coverage remains available for fixtures.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.5 - Remove Runtime Mock Route~~

Status: Complete on 2026-06-07.

Update: The live Tauri `runtime_status` command now uses runtime defaults that
exclude the deterministic mock provider and do not select a mock coding route.
The mock provider remains available only through explicit fixture registries for
backend tests.

Scope:

- ~~Add runtime model-provider defaults separate from fixture defaults.~~
- ~~Return no coding route from live runtime status until a real route is ready.~~
- ~~Keep mock completion tests on the explicit fixture registry.~~
- ~~Add bridge verifier coverage so runtime status cannot silently reselect the mock route.~~

Acceptance:

- ~~Tauri runtime status does not expose `mock-local` as a live provider.~~
- ~~Tauri runtime status does not expose `delyx-mock-coder` as the live coding route.~~
- ~~Backend mock provider tests continue to pass as deterministic fixtures.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.6 - Tauri Ollama Runtime Detection~~

Status: Complete on 2026-06-07.

Update: The live Tauri `runtime_status` command now performs a read-only
loopback probe against local Ollama `/api/tags`, reports ready, unreachable,
or empty-model states without storing secrets, and promotes the first discovered
local Ollama model into the coding route. This only exposes truthful provider
state; it does not execute tools, file writes, terminal commands, memory saves,
connector actions, scheduled work, or external agents.

Scope:

- ~~Probe local Ollama through `127.0.0.1:11434/api/tags` from the Rust runtime bridge.~~
- ~~Parse Ollama tags with `serde_json` instead of ad hoc response scanning.~~
- ~~Map discovered local models into a ready Ollama provider state.~~
- ~~Keep no-model, HTTP-error, and connection-error states visible and unusable.~~
- ~~Promote only a ready discovered Ollama model into the coding route.~~
- ~~Add deterministic parser/provider/bridge tests and verifier markers.~~

Acceptance:

- ~~Tauri runtime status can surface a real local Ollama coding route.~~
- ~~Unavailable Ollama remains a truthful unreachable or not-configured state.~~
- ~~The bridge remains read-only and local-first with no repo-stored secrets.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.7 - Renderer Runtime Model Sync~~

Status: Complete on 2026-06-07.

Update: The desktop renderer now applies the Tauri `runtime_status` provider
and coding-route response to the visible model settings on first load. Web
preview still performs direct read-only Ollama discovery, and failed browser
refreshes clear stale Ollama routes instead of leaving a model selected after
the provider becomes unavailable.

Scope:

- ~~Map Tauri runtime providers into the frontend model settings shape.~~
- ~~Promote only the bridge-reported coding route into the visible route list.~~
- ~~Use browser `/api/tags` refresh only when the Tauri bridge is unavailable or the user explicitly refreshes Ollama.~~
- ~~Clear stale Ollama routes when refresh reports unreachable or not configured.~~
- ~~Add verifier markers for bridge-to-model sync and route cleanup.~~

Acceptance:

- ~~Desktop first load can show the real Ollama model discovered by Rust runtime status.~~
- ~~Web preview remains honest when the Rust bridge is unavailable.~~
- ~~Unavailable Ollama does not leave stale saved Ollama routes in UI state.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.8 - Tauri Ollama Chat Bridge~~

Status: Complete on 2026-06-07.

Update: Desktop Ollama chat calls now flow through a Tauri `ollama_chat`
runtime command instead of renderer-only networking. The bridge validates the
selected model and message roles before requesting local `/api/chat`, returns
provider/model/text output for AgentRun evidence, keeps the direct browser
fetch only as a web-preview fallback, and has deterministic parser/error tests
without requiring a live model.

Scope:

- ~~Add a Tauri runtime command for local Ollama `/api/chat`.~~
- ~~Validate model selection and supported message roles before network I/O.~~
- ~~Return provider/model/text output for composer and PlanAgent model-call artifacts.~~
- ~~Use the runtime bridge in desktop while preserving web-preview fallback.~~
- ~~Add deterministic chat parser, HTTP-error, and preflight validation tests.~~

Acceptance:

- ~~Desktop agent replies and PlanAgent drafts use the Tauri Ollama bridge.~~
- ~~Web preview remains usable when no Tauri bridge exists.~~
- ~~Malformed, empty, unavailable, and HTTP-error chat states remain visible as failures.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.9 - Remove Frontend Mock Provider Kind~~

Status: Complete on 2026-06-07.

Update: Removed the user-facing `"mock"` provider kind from the frontend model
view types and empty provider fallback. Unknown or unsupported runtime provider
kinds now map to an unavailable UI state, and the verifier rejects frontend
source files that reintroduce `kind: "mock"`.

Scope:

- ~~Remove `"mock"` from frontend `ProviderKind`.~~
- ~~Use an unavailable provider kind for empty model settings.~~
- ~~Map unexpected runtime provider kinds to unavailable instead of mock.~~
- ~~Add verifier coverage against frontend `kind: "mock"` regression.~~

Acceptance:

- ~~The UI model layer cannot present a mock provider kind as live state.~~
- ~~Unexpected runtime provider kinds remain visible as unavailable.~~
- ~~Backend deterministic mock fixtures remain isolated to Rust tests.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 11.10 - Ollama Agent Session Bridge~~

Status: Complete on 2026-06-07.

Update: The real Ollama composer and PlanAgent paths now append user,
assistant, and system messages through the Tauri thread/run bridge when running
in desktop mode. Model-call status changes use the same bridge-backed thread
state, active Ollama work can settle back to idle after a real response, and
web preview keeps its local fallback. This records conversation state without
granting tool, file, terminal, connector, durable memory, scheduled-work, or
external-agent authority.

Scope:

- ~~Add a Tauri command for bridge-backed thread message appends.~~
- ~~Validate message role, non-empty body, status, and linked thread before recording.~~
- ~~Wire composer user, assistant, and failure messages through the bridge.~~
- ~~Wire Ollama PlanAgent system, success, and failure messages through the bridge.~~
- ~~Allow active explore/plan model work to return to idle as a typed transition.~~
- ~~Add verifier markers and deterministic Rust tests for the session bridge.~~

Acceptance:

- ~~Desktop Ollama conversations are represented in the bridge snapshot instead of only renderer state.~~
- ~~Invalid message roles are rejected without changing thread history.~~
- ~~Ollama success/failure remains visible and local-first with no stored secrets.~~
- ~~Source files stay within the line-budget rule.~~

Validation:

- ~~`npm run typecheck` passed.~~
- ~~`cargo test --workspace` passed with 161 Rust tests.~~
- ~~`npm test` passed.~~
- ~~`npm run build` passed.~~
- ~~`npm run smoke:ui` passed.~~
- ~~`npm run smoke:tauri` passed.~~
- ~~`git diff --check` passed with only existing CRLF normalization warnings.~~

---

### ~~PR 12 — External Agent Bridge Prototype~~

Status: Complete on 2026-06-07.

Update: Added an approval-gated ExternalAgentBridge prototype with Codex CLI
and Claude Code detection-only adapters, a generic terminal-agent prototype
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

### ~~PR 12.1 - Truthful External Agent Detection~~

Status: Complete on 2026-06-07.

Update: Replaced the Codex CLI and Claude Code placeholder availability states
with read-only PATH detection. Delyx now reports whether `codex` or `claude`
executables are available while keeping their execution disabled until explicit
command contracts and approvals are added.

Scope:

- ~~Detect Codex CLI from PATH without launching it.~~
- ~~Detect Claude Code from PATH without launching it.~~
- ~~Report missing and not-checked states truthfully.~~
- ~~Keep generic terminal execution as the only approved external worker path.~~
- ~~Cover detection with deterministic fake-executable tests.~~

Acceptance:

- ~~Detection does not execute external agents.~~
- ~~Installed and missing adapter states are both test-covered.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 12.2 - External Agent Command Contracts~~

Status: Complete on 2026-06-07.

Update: Added typed command contracts for Codex CLI and Claude Code without
auto-launching either adapter. Contracts produce visible command arrays,
working directory, permission mode, transcript format, and required Delyx tools.
Execution still requires the existing external-agent approval, terminal-command
approval, checkpoint/worktree isolation, captured terminal output, diff review,
and rollback path.

Scope:

- ~~Build Codex CLI `codex exec` contracts with JSONL output and explicit sandbox mode.~~
- ~~Build Claude Code `claude -p` contracts with stream JSON output, permission mode, and restricted tools.~~
- ~~Expose read-only and workspace-write contract modes without bypassing approvals.~~
- ~~Reject empty tasks and generic-terminal contract generation.~~
- ~~Add deterministic contract tests and verifier markers.~~

Acceptance:

- ~~Codex CLI and Claude Code are no longer only placeholder/detection concepts.~~
- ~~Command contracts are inspectable before execution.~~
- ~~No external agent command runs without approval-gated bridge execution.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 12.3 - External Agent Contract UI State~~

Status: Complete on 2026-06-07.

Update: Added a frontend external-agent command contract ledger and renderer.
The ledger starts empty, so no fake contract appears, but real contracts now
have a visible pre-execution UI state showing permission mode, command, cwd,
transcript format, required Delyx tools, and safety summary.

Scope:

- ~~Add an external-agent command contract view model.~~
- ~~Keep contract previews empty until a real contract is proposed.~~
- ~~Render contract permissions, command, cwd, transcript, tools, and safety summary.~~
- ~~Keep captured external-agent artifacts separate from pre-execution contracts.~~
- ~~Add verifier coverage for the empty and rendered contract states.~~

Acceptance:

- ~~External-agent command contracts have a first-class UI state before execution.~~
- ~~No fake external-agent contract is seeded into the first-run UI.~~
- ~~External-agent run artifacts remain visible only after captured execution.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 12.4 - External Agent Contract Preview Command~~

Status: Complete on 2026-06-07.

Update: Added a Tauri-backed command contract preview flow for Codex CLI and
Claude Code. The command palette can request a read-only contract preview for
the active thread/run, the Rust bridge builds the inspectable command contract,
and the UI upserts the returned contract into the external-agent ledger without
launching an external agent or requesting broader permissions.

Scope:

- ~~Expose a Tauri `external_agent_contract_preview` command.~~
- ~~Build preview contracts from the Rust command-contract source of truth.~~
- ~~Store returned contracts in the frontend external-agent ledger.~~
- ~~Add command palette entries for Codex and Claude read-only previews.~~
- ~~Keep web preview truthful when the desktop bridge is unavailable.~~
- ~~Add deterministic Rust tests and verifier markers.~~

Acceptance:

- ~~A user can create an inspectable external-agent contract before execution.~~
- ~~Contract preview does not launch Codex, Claude, terminal commands, or file writes.~~
- ~~No fake contract is seeded; previews require an active real thread/run.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 12.5 - External Agent Command Array Rendering~~

Status: Complete on 2026-06-07.

Update: Contract previews now render the command program and `args[]` entries
separately instead of flattening the command array into a shell-like string.
This keeps task prompts, flags, and sandbox arguments inspectable before any
approval or execution path.

Scope:

- ~~Render external-agent command `program` separately from `args[]`.~~
- ~~Preserve each argument as an indexed UI item.~~
- ~~Add verifier coverage for array rendering.~~

Acceptance:

- ~~Command contracts remain inspectable as arrays before execution.~~
- ~~Flattened command strings are not the only visible contract form.~~
- ~~Source files stay within the line-budget rule.~~

---

### ~~PR 12.6 - External Agent Adapter Status Bridge~~

Status: Complete on 2026-06-07.

Update: Added a read-only Tauri status bridge for external-agent adapters.
The UI now loads Codex CLI, Claude Code, and generic terminal adapter
availability into the runtime drawer. Web preview maps adapters to a truthful
not-checked state when the desktop bridge is unavailable.

Scope:

- ~~Expose `external_agent_status` from the Tauri runtime.~~
- ~~Map Rust adapter availability into frontend adapter status views.~~
- ~~Load adapter status into AppShell external-agent state on startup.~~
- ~~Render adapter availability in the terminal/runtime drawer.~~
- ~~Use not-checked adapter states when the desktop bridge is unavailable.~~
- ~~Add deterministic Rust tests and verifier markers.~~

Acceptance:

- ~~External-agent detection is visible in the UI trust layer.~~
- ~~Web preview does not pretend adapters were checked.~~
- ~~No external agent command runs during adapter detection.~~
- ~~Source files stay within the line-budget rule.~~

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

### ~~PR 13.1 - Active Run Evidence Inspector Wiring~~

Status: Complete on 2026-06-07.

Update: The main inspector now renders real active-run EvidenceRecords through
the Evidence panel. Run receipts are marked as recorded, not claim-supporting,
until a real ResearchAnswer audit exists for that run.

Scope:

- ~~Derive a ResearchAnswer view from active AgentRun evidence receipts.~~
- ~~Mark unaudited run receipts as recorded instead of supports/contradicts.~~
- ~~Prefer real ResearchAnswer audits when they exist for the active run.~~
- ~~Render active-run evidence in the inspector before generic run status.~~
- ~~Add verifier coverage for active-run evidence wiring.~~

Acceptance:

- ~~Model-call and other run evidence receipts can appear in the Evidence panel.~~
- ~~Recorded receipts do not claim source-backed support.~~
- ~~No fake evidence receipt is seeded into the first-run UI.~~
- ~~Source files stay within the line-budget rule.~~

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
- ~~Promoted memory candidates cannot be re-suppressed as pending candidates.~~
- ~~Memory shows source run/thread.~~
- ~~Durable memory approvals are bound to the specific candidate node so one save approval cannot promote another candidate from the same run.~~

---

### ~~PR 14.1 - Active Run Memory Inspector Wiring~~

Status: Complete on 2026-06-07.

Update: Wired the existing memory governance state into the cockpit inspector
only when real candidates or records exist for the active run. Empty memory
state remains invisible in the inspector, so no fake memory panel replaces the
normal run status.

Scope:

- ~~Detect active-run memory candidates and records.~~
- ~~Render real memory state beside evidence receipts when both exist.~~
- ~~Keep the default empty UI free of seeded memory data.~~
- ~~Add verifier coverage for memory inspector wiring.~~

Acceptance:

- ~~Memory inspector output is based on MemoryStateView only.~~
- ~~Empty memory state does not claim saved memory.~~
- ~~Evidence receipts and memory receipts can both remain visible.~~
- ~~Source files stay within the line-budget rule.~~

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

### ~~PR 15.1 - Imported Skills Inspector Wiring~~

Status: Complete on 2026-06-07.

Update: Wired the existing skill state into the cockpit inspector only when
real skills have been imported. The default empty UI remains free of seeded
skill manifests.

Scope:

- ~~Detect imported skills from SkillStateView.~~
- ~~Render real skill status, trust, source hash, and permissions.~~
- ~~Keep third-party skills inactive unless explicitly reviewed.~~
- ~~Add verifier coverage for skill inspector wiring.~~

Acceptance:

- ~~Skills inspector output is based on SkillStateView only.~~
- ~~Empty skill state does not claim imported or active skills.~~
- ~~Skill permissions remain visible when real skills exist.~~
- ~~Source files stay within the line-budget rule.~~

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
- ~~Mission contract activation approvals are bound to the specific contract so one scheduled approval cannot activate another mission.~~

---

### ~~PR 16.1 - Automation Inspector Wiring~~

Status: Complete on 2026-06-07.

Update: Wired the existing automation state into the cockpit inspector only
when real mission contracts or scheduled runs exist. The default empty UI does
not claim paused contracts or recurring work.

Scope:

- ~~Detect mission contracts and scheduled runs from AutomationStateView.~~
- ~~Render real automation status, scope, allowed tools, active hours, and stop conditions.~~
- ~~Keep recurring work paused until approved.~~
- ~~Add verifier coverage for automation inspector wiring.~~

Acceptance:

- ~~Automation inspector output is based on AutomationStateView only.~~
- ~~Empty automation state does not claim configured mission contracts.~~
- ~~Risky scheduled work remains approval-gated by the runtime model.~~
- ~~Source files stay within the line-budget rule.~~

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
- ~~Mobile approval view filters out non-pending approval records.~~
- ~~Mobile can review status without running full agent runtime.~~
- ~~Mobile denial remains available for broad or high-risk requests without granting file or terminal access.~~

---

### ~~PR 17.1 - Mobile Companion Inspector Wiring~~

Status: Complete on 2026-06-07.

Update: Wired the existing mobile companion state into the cockpit inspector
only when the companion is paired or real mobile-visible threads, approvals,
or runs exist. The default unpaired state stays hidden from the inspector and
does not imply mobile access.

Scope:

- ~~Detect paired/mobile-visible activity from MobileStateView.~~
- ~~Render real mobile policy, approval scope, threads, and run status.~~
- ~~Keep broad file and terminal access denied by default.~~
- ~~Add verifier coverage for mobile inspector wiring.~~

Acceptance:

- ~~Mobile inspector output is based on MobileStateView only.~~
- ~~Unpaired empty state does not claim mobile access.~~
- ~~Mobile policy remains visible when real mobile activity exists.~~
- ~~Source files stay within the line-budget rule.~~

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
- ~~Support bundle redaction covers common token prefixes, AWS access keys, and private-key blocks.~~
- ~~First-run release readiness does not claim release smoke or support bundle artifacts are loaded until real artifacts are attached.~~

---

### ~~PR 18.1 - Release Readiness Inspector Wiring~~

Status: Complete on 2026-06-07.

Update: Wired the existing release readiness state into the cockpit inspector
only when real release smoke, support bundle, update metadata, or non-default
signing status exists. The default unsigned-dev placeholder does not create an
inspector panel by itself.

Scope:

- ~~Detect real release readiness signals from ReleaseStateView.~~
- ~~Render release smoke, signing, support bundle, and update metadata status only when present.~~
- ~~Keep first-run release UI from claiming unattached artifacts.~~
- ~~Add verifier coverage for release inspector wiring.~~

Acceptance:

- ~~Release inspector output is based on ReleaseStateView only.~~
- ~~Default placeholder state does not claim release artifacts.~~
- ~~Support bundle and update metadata states remain explicit.~~
- ~~Source files stay within the line-budget rule.~~

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
- ~~Changed-file capture metadata is rejected when it points outside the approved external-agent scope.~~
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

~~Do not implement real agent runtime yet.~~ Superseded narrowly by PR 11.3:
Ollama may draft read-only PlanView output through the visible AgentRun ledger,
while risky tool execution remains approval-gated and unimplemented here.
~~Do not implement real model providers yet.~~ Superseded by PR 11.1: local
Ollama composer usage is now allowed and implemented behind truthful provider
health states.
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

~~Do not implement real model calls yet.~~ Superseded by PR 11.1 and PR 11.3:
local Ollama model calls are allowed for composer replies and read-only plan
drafts when recorded as AgentRun artifacts. Deterministic fixtures still cover
parser and verifier behavior.

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
