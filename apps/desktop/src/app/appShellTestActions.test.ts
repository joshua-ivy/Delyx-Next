import { beforeEach, describe, expect, it, vi } from "vitest";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
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
const proposeApproval = vi.mocked(proposeApprovalOverBridge);
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
  proposeApproval.mockImplementation(async (proposal) => proposal);
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

  it("does not duplicate a pending test approval", async () => {
    await runTestsForActiveRun(state({ actionProposals: [testApproval("pending")] }));

    expect(proposeApproval).not.toHaveBeenCalled();
    expect(executeTest).not.toHaveBeenCalled();
    expect(notify).toHaveBeenCalledWith("Approve the test command, then run tests again", "warning");
  });

  it("does not requeue a denied test approval", async () => {
    await runTestsForActiveRun(state({ actionProposals: [testApproval("denied")] }));

    expect(proposeApproval).not.toHaveBeenCalled();
    expect(executeTest).not.toHaveBeenCalled();
    expect(notify).toHaveBeenCalledWith(
      "Test approval was denied; Delyx will not run that command",
      "warning",
    );
  });

  it("queues a fresh approval after an expired test approval", async () => {
    await runTestsForActiveRun(state({ actionProposals: [testApproval("expired")] }));

    expect(proposeApproval).toHaveBeenCalledWith(expect.objectContaining({
      id: expect.stringMatching(/^approval-run-1-test-npm-test-\d+$/),
      nodeId: "run-1-test-npm-test",
      status: "pending",
    }));
    expect(executeTest).not.toHaveBeenCalled();
  });
});

function state({
  actionProposals = [testApproval("approved")],
  schedulerTestApprovalId,
}: {
  actionProposals?: ReturnType<typeof testApproval>[];
  schedulerTestApprovalId?: string;
} = {}) {
  return {
    actionProposals,
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

function testApproval(status: "approved" | "denied" | "expired" | "pending") {
  return {
    actionType: "run_terminal" as const,
    expectedResult: "Run tests.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id: status === "approved" ? "approval-test" : "approval-run-1-test-npm-test",
    nodeId: status === "approved" ? "node-test" : "run-1-test-npm-test",
    rationale: "Validate patch.",
    requiredPermission: "terminal_command",
    riskLabel: "medium" as const,
    runId: "run-1",
    scope: { commands: ["npm test"], kind: "terminal" as const, root: "C:\\repo", summary: "Run tests" },
    status,
  };
}
