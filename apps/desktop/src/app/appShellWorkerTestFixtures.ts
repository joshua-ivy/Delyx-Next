import { vi } from "vitest";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import type { ComposerBindingState } from "./cockpitComposerBindings";

export function project(): WorkspaceProject {
  return {
    approvalPolicy: "approval-gated",
    approvedRoots: ["C:/code/app"],
    git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
    id: "project-1",
    indexedFiles: [],
    isolation: { detail: "none", label: "none", mode: "none" },
    lastOpenedLabel: "now",
    name: "app",
    path: "C:/code/app",
    pinned: true,
    rulesFiles: [],
  };
}

export function thread(): TaskThread {
  return {
    activeRunId: "run-1",
    archived: false,
    createdAt: "2026-06-09T00:00:00.000Z",
    createdLabel: "now",
    goal: "Refactor the parser",
    id: "thread-1",
    messages: [{ role: "user", body: "Refactor the parser" }],
    mode: "explore",
    projectId: "project-1",
    runIds: ["run-1"],
    status: "idle",
    title: "Refactor the parser",
    updatedAt: "2026-06-09T00:00:00.000Z",
  };
}

export function state(): ComposerBindingState {
  return {
    activeProject: project(),
    activeRun: undefined,
    activeThread: thread(),
    modelSettings: { providers: [], routes: [], selectedProviderId: "" },
    setActionProposals: vi.fn(),
    setActiveThreadId: vi.fn(),
    setAgentRuns: vi.fn(),
    setThreads: vi.fn(),
    setThreadState: vi.fn(),
    threads: [thread()],
  };
}

export function card(over: Partial<ActionProposalView>): ActionProposalView {
  return {
    id: "run-1-worker-external",
    runId: "run-1",
    nodeId: "run-1-worker-external",
    actionType: "external_agent",
    riskLabel: "high",
    requiredPermission: "Run Claude Code read-only inside the project root",
    rationale: "Task: Refactor the parser",
    expectedResult: "explores",
    scope: { kind: "external_agent", summary: "s", root: "C:/code/app" },
    expiresAt: "2999-01-01T00:00:00.000Z",
    status: "pending",
    ...over,
  };
}
