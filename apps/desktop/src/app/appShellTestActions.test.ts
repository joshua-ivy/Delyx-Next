import { beforeEach, describe, expect, it, vi } from "vitest";

import { executeTestRunNodeOverBridge } from "../features/runs/agentExecutorClient";
import { loadTestSnapshot } from "../features/tests/testClient";
import { loadThreadRunSnapshot, updateThreadStatusOverBridge } from "../features/threads/threadClient";
import { notifyLocalAction } from "./ShellPreferenceController";
import { runTestsForActiveRun } from "./appShellTestActions";

vi.mock("../features/approvals/approvalClient", () => ({ proposeApprovalOverBridge: vi.fn() }));
vi.mock("../features/runs/agentExecutorClient", () => ({ executeTestRunNodeOverBridge: vi.fn() }));
vi.mock("../features/tests/testClient", () => ({ loadTestSnapshot: vi.fn() }));
vi.mock("../features/threads/threadClient", () => ({
  loadThreadRunSnapshot: vi.fn(),
  updateThreadStatusOverBridge: vi.fn(),
}));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));

const executeTest = vi.mocked(executeTestRunNodeOverBridge);
const loadTests = vi.mocked(loadTestSnapshot);
const loadSnapshot = vi.mocked(loadThreadRunSnapshot);
const notify = vi.mocked(notifyLocalAction);
const updateThread = vi.mocked(updateThreadStatusOverBridge);

beforeEach(() => {
  vi.clearAllMocks();
  executeTest.mockResolvedValue({
    message: "Test artifact test-1 passed.",
    runId: "run-1",
    status: "completed",
    testArtifactId: "test-1",
  });
  loadTests.mockResolvedValue([]);
  loadSnapshot.mockResolvedValue({ runs: [], threads: [] });
  updateThread.mockResolvedValue(undefined);
});

describe("runTestsForActiveRun", () => {
  it("uses the scheduler-selected approved test approval id", async () => {
    await runTestsForActiveRun(state({ schedulerTestApprovalId: "approval-test" }));

    expect(executeTest).toHaveBeenCalledWith(expect.objectContaining({
      approvalId: "approval-test",
      args: ["test"],
      program: "npm",
      runId: "run-1",
    }));
  });

  it("does not fall back when the scheduler-selected test approval is missing", async () => {
    await runTestsForActiveRun(state({ schedulerTestApprovalId: "missing-approval" }));

    expect(executeTest).not.toHaveBeenCalled();
    expect(notify).toHaveBeenCalledWith(
      "Scheduler-selected test approval is no longer executable",
      "warning",
    );
  });
});

function state({ schedulerTestApprovalId }: { schedulerTestApprovalId?: string } = {}) {
  return {
    actionProposals: [testApproval()],
    activePlan: {
      decision: "approved" as const,
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
      goalUnderstanding: "Test.",
      permissionsNeeded: [],
      risks: [],
      rollbackStrategy: "",
      steps: [],
      testsToRun: ["npm test"],
      threadId: "thread-1",
    },
    activeProject: {
      approvalPolicy: "manual",
      approvedRoots: ["C:\\repo"],
      git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
      id: "project-1",
      indexedFiles: [],
      isolation: { detail: "none", label: "none", mode: "none" as const },
      lastOpenedLabel: "now",
      name: "Project",
      path: "C:\\repo",
      pinned: true,
      rulesFiles: [],
    },
    activeRun: { id: "run-1" },
    activeThread: { id: "thread-1" },
    patches: [],
    schedulerConfirmedRunTests: true,
    schedulerTestApprovalId,
    setActionProposals: vi.fn(),
    setAgentRuns: vi.fn(),
    setTests: vi.fn(),
    setThreadState: vi.fn(),
    setThreads: vi.fn(),
  } as never;
}

function testApproval() {
  return {
    actionType: "run_terminal" as const,
    expectedResult: "Run tests.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id: "approval-test",
    nodeId: "node-test",
    rationale: "Validate patch.",
    requiredPermission: "terminal_command",
    riskLabel: "medium" as const,
    runId: "run-1",
    scope: { commands: ["npm test"], kind: "terminal" as const, root: "C:\\repo", summary: "Run tests" },
    status: "approved" as const,
  };
}
