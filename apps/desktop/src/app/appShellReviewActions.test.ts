import { beforeEach, describe, expect, it, vi } from "vitest";

import { requestReviewRevisionOverBridge } from "../features/runs/agentExecutorClient";
import { loadReviewSnapshot } from "../features/review/reviewClient";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import { notifyLocalAction } from "./ShellPreferenceController";
import { requestRepairForReviewFinding } from "./appShellReviewActions";
import type { ReviewRepairState } from "./appShellReviewActions";

vi.mock("../features/runs/agentExecutorClient", () => ({
  executeReviewNodeOverBridge: vi.fn(),
  requestReviewRevisionOverBridge: vi.fn(),
}));
vi.mock("../features/review/reviewClient", () => ({ loadReviewSnapshot: vi.fn() }));
vi.mock("../features/threads/threadClient", () => ({
  loadThreadRunSnapshot: vi.fn(),
  updateThreadStatusOverBridge: vi.fn(),
}));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));

const requestRevision = vi.mocked(requestReviewRevisionOverBridge);
const loadReviews = vi.mocked(loadReviewSnapshot);
const loadSnapshot = vi.mocked(loadThreadRunSnapshot);
const notify = vi.mocked(notifyLocalAction);

beforeEach(() => {
  vi.clearAllMocks();
  requestRevision.mockResolvedValue({
    findingId: "finding-1",
    message: "Repair requested from review finding; next flow is plan -> build.",
    nextFlow: ["plan", "build"],
    reviewReportId: "review-1",
    runId: "run-1",
    status: "revise_requested",
  });
  loadReviews.mockResolvedValue([{ ...review(), decision: "revise_requested" }]);
  loadSnapshot.mockResolvedValue({ runs: [run() as never], threads: [thread() as never] });
});

describe("requestRepairForReviewFinding", () => {
  it("requests repair for an existing review finding and reloads receipts", async () => {
    const state = reviewState();

    await requestRepairForReviewFinding(state, "review-1", "finding-1");

    expect(requestRevision).toHaveBeenCalledWith("run-1", "review-1", "finding-1");
    expect(loadReviews).toHaveBeenCalledWith("run-1");
    expect(loadSnapshot).toHaveBeenCalledWith("project-1");
    expect(state.setReviews).toHaveBeenCalled();
    expect(state.setAgentRuns).toHaveBeenCalledWith([run()]);
    expect(state.setThreads).toHaveBeenCalledWith([thread()]);
    expect(notify).toHaveBeenCalledWith(expect.stringContaining("Repair requested"), "success");
  });

  it("does not call the bridge for a missing finding", async () => {
    await requestRepairForReviewFinding(reviewState(), "review-1", "missing");

    expect(requestRevision).not.toHaveBeenCalled();
    expect(notify).toHaveBeenCalledWith(
      "Select a real review finding before requesting repair",
      "warning",
    );
  });
});

function reviewState(): ReviewRepairState {
  return {
    activeProject: { id: "project-1" } as never,
    activeRun: { id: "run-1" } as never,
    activeThread: { id: "thread-1" } as never,
    reviews: [review()],
    setAgentRuns: vi.fn(),
    setReviews: vi.fn(),
    setThreadState: vi.fn(),
    setThreads: vi.fn(),
  };
}

function review() {
  return {
    decision: "pending" as const,
    evidenceSummary: "Stored diff receipts.",
    findings: [{
      detail: "Runtime panic risk in new code.",
      filePath: "src/main.rs",
      hunkLabel: "patch-1:0",
      id: "finding-1",
      priority: "p1" as const,
      riskLabel: "panic",
      suggestedFix: "Handle the None/Err case explicitly.",
      title: "Added unwrap can panic",
    }],
    id: "review-1",
    mode: "read_only" as const,
    riskSummary: "1 prioritized finding.",
    runId: "run-1",
    testSummary: "Tests passed.",
  };
}

function run() {
  return { id: "run-1" };
}

function thread() {
  return { id: "thread-1" };
}
