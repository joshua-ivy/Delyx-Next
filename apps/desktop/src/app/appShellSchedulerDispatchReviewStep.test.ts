import { beforeEach, describe, expect, it, vi } from "vitest";

import { loadReviewSnapshot } from "../features/review/reviewClient";
import { type AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import { runReviewSchedulerStepOverBridge } from "../features/runs/agentSchedulerStepClient";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";

vi.mock("./appShellFinalAnswerActions", () => ({ recordFinalSupportForActiveThread: vi.fn() }));
vi.mock("./appShellOllamaPatchActions", () => ({ proposeApprovedPlanPatchWithOllama: vi.fn() }));
vi.mock("./appShellPatchActions", () => ({ applyApprovedPatchForActiveRun: vi.fn() }));
vi.mock("./appShellTestActions", () => ({ runTestsForActiveRun: vi.fn() }));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));
vi.mock("../features/review/reviewClient", () => ({ loadReviewSnapshot: vi.fn() }));
vi.mock("../features/runs/agentExecutorClient", () => ({ scheduleNextRunActionOverBridge: vi.fn() }));
vi.mock("../features/runs/agentSchedulerStepClient", () => ({ runReviewSchedulerStepOverBridge: vi.fn() }));
vi.mock("../features/threads/threadClient", () => ({ loadThreadRunSnapshot: vi.fn() }));

const loadReviews = vi.mocked(loadReviewSnapshot);
const loadSnapshot = vi.mocked(loadThreadRunSnapshot);
const runReviewStep = vi.mocked(runReviewSchedulerStepOverBridge);

beforeEach(() => {
  vi.clearAllMocks();
  runReviewStep.mockResolvedValue({
    message: "Review report review-1 captured with 0 finding(s).",
    reviewReportId: "review-1",
    runId: "run-1",
    status: "completed",
  });
  loadReviews.mockResolvedValue([{ decision: "approved", findings: [], id: "review-1", runId: "run-1" } as never]);
  loadSnapshot.mockResolvedValue({ runs: [{ id: "run-1" }] as never, threads: [] });
});

describe("dispatchSchedulerDecision review step", () => {
  it("runs scheduler review through Rust without renderer-owned artifacts", async () => {
    const handled = await dispatchSchedulerDecision(state(), decision("run_review"));

    expect(handled).toBe(true);
    expect(runReviewStep).toHaveBeenCalledWith(expect.objectContaining({ runId: "run-1" }));
    expect(runReviewStep.mock.calls[0]?.[0]).not.toHaveProperty("patches");
    expect(runReviewStep.mock.calls[0]?.[0]).not.toHaveProperty("tests");
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
