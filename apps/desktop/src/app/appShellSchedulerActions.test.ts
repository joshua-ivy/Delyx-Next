import { beforeEach, describe, expect, it, vi } from "vitest";

import type { PlanView } from "../features/plans/planTypes";
import { resumeWaitingRunOverBridge } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { resumeSchedulerRun } from "./appShellSchedulerActions";

vi.mock("../features/runs/agentExecutorClient", () => ({
  resumeWaitingRunOverBridge: vi.fn(),
}));

vi.mock("../features/threads/threadClient", () => ({
  loadThreadRunSnapshot: vi.fn(),
}));

vi.mock("./ShellPreferenceController", () => ({
  notifyLocalAction: vi.fn(),
}));

const resumeBridge = vi.mocked(resumeWaitingRunOverBridge);
const loadSnapshot = vi.mocked(loadThreadRunSnapshot);

beforeEach(() => {
  vi.clearAllMocks();
  resumeBridge.mockResolvedValue({
    approvalIds: [],
    kind: "run_tests",
    message: "Tests are next.",
    patchCount: 0,
    runId: "run-1",
    testCount: 0,
  });
  loadSnapshot.mockResolvedValue(undefined);
});

describe("resumeSchedulerRun", () => {
  it("passes the active plan supported-test signal to the scheduler bridge", async () => {
    await resumeSchedulerRun(stateWithPlan([".\\.tools\\npm.cmd test"]));

    expect(resumeBridge).toHaveBeenCalledWith(expect.objectContaining({
      hasSupportedTestCommand: true,
      runId: "run-1",
    }));
  });

  it("does not treat unsafe shell-control test text as supported", async () => {
    await resumeSchedulerRun(stateWithPlan(["npm test && whoami"]));

    expect(resumeBridge).toHaveBeenCalledWith(expect.objectContaining({
      hasSupportedTestCommand: false,
      runId: "run-1",
    }));
  });
});

function stateWithPlan(testsToRun: string[]) {
  return {
    activePlan: plan(testsToRun),
    activeProject: project(),
    activeRun: run(),
    setAgentRuns: vi.fn(),
    setThreads: vi.fn(),
    setThreadState: vi.fn(),
  };
}

function plan(testsToRun: string[]): PlanView {
  return {
    decision: "approved",
    explore: {
      architectureSummary: "",
      projectCommands: [],
      relevantFiles: [],
      relevantSymbols: [],
      risks: [],
      suggestedNextSteps: [],
      unknowns: [],
    },
    filesLikelyInvolved: [],
    goalUnderstanding: "test",
    permissionsNeeded: [],
    risks: [],
    rollbackStrategy: "",
    steps: [],
    testsToRun,
    threadId: "thread-1",
  };
}

function project(): WorkspaceProject {
  return {
    approvalPolicy: "manual",
    approvedRoots: ["C:\\repo"],
    git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
    id: "project-1",
    indexedFiles: [],
    isolation: { detail: "none", label: "none", mode: "none" },
    lastOpenedLabel: "now",
    name: "Project",
    path: "C:\\repo",
    pinned: true,
    rulesFiles: [],
  };
}

function run(): AgentRunView {
  return {
    artifacts: [],
    createdAt: "2026-06-08T00:00:00.000Z",
    events: [],
    evidence: [],
    goal: "test",
    id: "run-1",
    metrics: { approvalCount: 0, artifactCount: 0, commandCount: 0, eventCount: 0, evidenceCount: 0, nodeCount: 0 },
    mode: "build",
    nodes: [],
    status: "waiting_for_approval",
    threadId: "thread-1",
    updatedAt: "2026-06-08T00:00:00.000Z",
  };
}
