import { beforeEach, describe, expect, it, vi } from "vitest";

import { scheduleNextRunActionOverBridge, type AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import { runTestSchedulerStepOverBridge } from "../features/runs/agentSchedulerStepClient";
import { loadTestSnapshot } from "../features/tests/testClient";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";
import { runTestsForActiveRun } from "./appShellTestActions";

vi.mock("./appShellFinalAnswerActions", () => ({ recordFinalSupportForActiveThread: vi.fn() }));
vi.mock("./appShellOllamaPatchActions", () => ({ proposeApprovedPlanPatchWithOllama: vi.fn() }));
vi.mock("./appShellPatchActions", () => ({ applyApprovedPatchForActiveRun: vi.fn() }));
vi.mock("./appShellReviewActions", () => ({ runReviewForActiveRun: vi.fn() }));
vi.mock("./appShellTestActions", () => ({ runTestsForActiveRun: vi.fn() }));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));
vi.mock("../features/runs/agentExecutorClient", () => ({ scheduleNextRunActionOverBridge: vi.fn() }));
vi.mock("../features/runs/agentSchedulerStepClient", () => ({ runTestSchedulerStepOverBridge: vi.fn() }));
vi.mock("../features/tests/testClient", () => ({ loadTestSnapshot: vi.fn() }));
vi.mock("../features/threads/threadClient", () => ({ loadThreadRunSnapshot: vi.fn() }));

const loadSnapshot = vi.mocked(loadThreadRunSnapshot);
const loadTests = vi.mocked(loadTestSnapshot);
const runTestStep = vi.mocked(runTestSchedulerStepOverBridge);
const runTests = vi.mocked(runTestsForActiveRun);
const scheduleNext = vi.mocked(scheduleNextRunActionOverBridge);

beforeEach(() => {
  vi.clearAllMocks();
  runTestStep.mockResolvedValue({
    message: "Test artifact test-artifact-1 passed.",
    runId: "run-1",
    status: "completed",
    testArtifactId: "test-artifact-1",
  });
  loadTests.mockResolvedValue([{
    approvalId: "approval-test",
    command: "cargo test --help",
    completedAt: "2026-06-08T01:00:01.000Z",
    cwd: "C:\\repo",
    durationMs: 1,
    execEvents: [],
    exitCode: 0,
    failureSummary: undefined,
    id: "test-artifact-1",
    outputTruncated: false,
    parsedFailures: undefined,
    runId: "run-1",
    startedAt: "2026-06-08T01:00:00.000Z",
    status: "passed",
    stderr: "",
    stdout: "",
  }]);
  loadSnapshot.mockResolvedValue({ runs: [{ id: "run-1" }] as never, threads: [] });
  scheduleNext.mockResolvedValue(undefined);
});

describe("dispatchSchedulerDecision test step", () => {
  it("runs approved scheduler test decisions through the Rust scheduler step", async () => {
    const handled = await dispatchSchedulerDecision(state(), {
      ...decision("run_tests"),
      approvalIds: ["approval-test"],
    });

    expect(handled).toBe(true);
    expect(runTestStep).toHaveBeenCalledWith(expect.objectContaining({ runId: "run-1" }));
    expect(runTestStep.mock.calls[0]?.[0]).not.toHaveProperty("approvalId");
    expect(runTestStep.mock.calls[0]?.[0]).not.toHaveProperty("program");
    expect(runTestStep.mock.calls[0]?.[0]).not.toHaveProperty("approvedRoots");
    expect(runTests).not.toHaveBeenCalled();
  });
});

function state() {
  return {
    actionProposals: [],
    activePlan: undefined,
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
    activeRun: { id: "run-1" } as never,
    activeThread: undefined,
    modelSettings: { providers: [], routes: [], selectedProviderId: "ollama-local" },
    patches: [],
    reviews: [],
    setActionProposals: vi.fn(),
    setAgentRuns: vi.fn(),
    setPatches: vi.fn(),
    setReviews: vi.fn(),
    setTests: vi.fn(),
    setThreadState: vi.fn(),
    setThreads: vi.fn(),
    tests: [],
  };
}

function decision(kind: AgentScheduleDecisionView["kind"]): AgentScheduleDecisionView {
  return {
    approvalIds: [],
    kind,
    message: "Scheduler decision.",
    patchCount: 0,
    runId: "run-1",
    testCount: 0,
  };
}
