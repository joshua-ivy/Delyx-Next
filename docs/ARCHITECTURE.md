# Delyx Next Architecture

Delyx Next should be local-first, UI-first, typed, and evidence-oriented.

Keep architecture proportional to the current milestone. Avoid empty module structures that exist only because the target architecture is large.

## Stack

Default stack:

- Tauri v2
- React
- TypeScript
- Rust
- SQLite
- Vite
- CSS variables for design tokens
- Radix UI primitives where useful
- Lucide icons

Use old Delyx's Tauri/React/TypeScript/Rust/SQLite direction unless a decision record proves otherwise.

## Source File Size Budget

Keep source files small enough to review comfortably:

- target: 300 lines or fewer
- review/split threshold: 400 lines
- hard cap: 500 lines

Exceptions are allowed for generated code, declarative config, snapshots/fixtures, or a documented architecture decision.

When a source file approaches the threshold, split by responsibility:

- React components
- hooks
- view-model builders
- Rust services
- repository methods
- policy evaluators
- type definitions

## Target Runtime Shape

```text
Delyx Next
  Workspace Manager
  Thread Manager
  Agent Runtime
  Permission Engine
  Tool Layer
  Model Layer
  Evidence Layer
  UI
```

## Workspace Manager

Owns:

- projects
- approved workspace roots
- Git state
- read-only file index/search
- rules file detection
- checkpoints
- worktrees later

Early rules:

- reads outside approved roots fail
- writes are not implemented before approval engine and patch system
- workspace scope is visible in UI

## Thread Manager

Owns:

- task threads
- conversation state
- active mode
- thread status
- linked AgentRuns
- archive/update flow

Threads are scoped to projects.

## Agent Runtime

Owns:

- AgentRun graph
- AgentNode execution
- AgentEvent log
- mode-specific agents
- lifecycle hooks
- resume/repair later

Core agents:

- ExploreAgent
- PlanAgent
- BuildAgent
- ReviewAgent
- TestAgent
- ResearchAgent

The AgentRun graph is the future execution/resume engine, not just an inspector artifact.

## Permission Engine

Owns:

- read policy
- write policy
- terminal policy
- network policy
- memory policy
- automation policy
- approval proposal creation
- approval decision enforcement

Risky actions must produce an ActionProposal before execution.

## Tool Layer

Owns:

- file read/search
- patch proposal
- patch apply
- terminal command runner
- test runner
- Git tools
- Python sandbox
- external agent bridge

Tool output should become artifacts or evidence records when relevant.

## Model Layer

Owns:

- mock provider
- Ollama provider
- OpenAI-compatible provider
- role routing
- model health checks
- missing provider/API-key states

Initial implementation should include a deterministic mock provider before real model calls.
Role routing may only save routes to providers whose health is ready; missing-key,
unconfigured, or unreachable providers remain visible but unusable.

Model roles:

- answer
- helper
- deepResearch
- maxReasoning
- coding
- embedding
- scoring

Secrets must not be stored in the repo.

## Evidence Layer

Owns:

- diffs
- test output
- terminal logs
- source receipts
- file hashes
- model-call records
- external agent transcripts
- final answer support

Evidence records should make claim support inspectable.

## Data Models

### Project

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

### TaskThread

```ts
interface TaskThread {
  id: string;
  projectId: string;
  title: string;
  goal: string;
  status: TaskStatus;
  mode: AgentMode;
  activeRunId?: string;
  runIds: string[];
  createdAt: string;
  updatedAt: string;
}
```

### AgentRun

```ts
interface AgentRun {
  id: string;
  projectId?: string;
  threadId?: string;
  parentRunId?: string;
  goal: string;
  mode: AgentMode;
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

### ActionProposal

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

### EvidenceRecord

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

## Decision Records

Record architecture decisions here until a dedicated ADR folder exists.

### ADR-0001: Clean Rebuild With Old Delyx As Baseline

Decision: Delyx Next is a clean rebuild. Old Delyx is used as reference, spec, eval source, safety-policy source, and salvage pool.

Reason: The new product direction requires a UI-first project/thread/diff workflow and should avoid copying old cockpit-first complexity.

### ADR-0002: Separate App Identity

Decision: Use `com.geaux.delyxnext` unless later packaging work changes it.

Reason: Old Delyx and Delyx Next should coexist during development and migration.

### ADR-0003: CSS Variables First

Decision: Use CSS variables and shared React components for PR 2 design tokens. Do not add Tailwind unless a later decision record justifies it.

Reason: The app needs stable product-specific tokens and a small, controlled design system before broad styling utilities.

### ADR-0004: Windows-First Unsigned Dev Packaging

Decision: PR 18 targets Windows NSIS packaging first. Dev builds are explicitly unsigned until certificate, digest, timestamp, and sign command inputs are configured together.

Reason: Packaging should be testable without pretending production signing exists. Support bundles redact logs and config summaries before export, and update metadata remains a disabled local placeholder.
