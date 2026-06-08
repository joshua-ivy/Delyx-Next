import { beforeEach, describe, expect, it, vi } from "vitest";

import { loadPatchSnapshot } from "../features/patches/patchClient";
import { loadReviewSnapshot } from "../features/review/reviewClient";
import { driveRunOverBridge } from "../features/runs/agentDriveClient";
import { loadTestSnapshot } from "../features/tests/testClient";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import {
  canRunSchedulerDriver,
  runSchedulerDriver,
} from "./appShellSchedulerDriveActions";

vi.mock("../features/patches/patchClient", () => ({ loadPatchSnapshot: vi.fn() }));
vi.mock("../features/review/reviewClient", () => ({ loadReviewSnapshot: vi.fn() }));
vi.mock("../features/runs/agentDriveClient", () => ({ driveRunOverBridge: vi.fn() }));
vi.mock("../features/tests/testClient", () => ({ loadTestSnapshot: vi.fn() }));
vi.mock("../features/threads/threadClient", () => ({ loadThreadRunSnapshot: vi.fn() }));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));

const drive = vi.mocked(driveRunOverBridge);
const loadPatches = vi.mocked(loadPatchSnapshot);
const loadReviews = vi.mocked(loadReviewSnapshot);
const loadSnapshot = vi.mocked(loadThreadRunSnapshot);
const loadTests = vi.mocked(loadTestSnapshot);

beforeEach(() => {
  vi.clearAllMocks();
  drive.mockResolvedValue({
    runId: "run-1",
    steps: [{ decision: "run_review", message: "Reviewed.", status: "completed" }],
    stoppedBecause: {
      approvalIds: [],
      kind: "needs_final_summary",
      message: "Final support needs an assistant answer.",
    },
  });
  loadPatches.mockResolvedValue([{ id: "patch-1", runId: "run-1", status: "applied" }] as never);
  loadTests.mockResolvedValue([{ id: "test-1", runId: "run-1", status: "passed" }] as never);
  loadReviews.mockResolvedValue([{ id: "review-1", runId: "run-1", findings: [] }] as never);
  loadSnapshot.mockResolvedValue({ runs: [{ id: "run-1" }] as never, threads: [] });
});

describe("scheduler driver action", () => {
  it("calls the Rust driver with run, clock, timeout, and existing final summary only", async () => {
    const state = schedulerState();

    await runSchedulerDriver(state);

    expect(drive).toHaveBeenCalledWith(expect.objectContaining({
      finalSummary: "Existing assistant answer.",
      runId: "run-1",
      timeoutMs: 5 * 60 * 1000,
    }));
    expect(drive.mock.calls[0]?.[0]).not.toHaveProperty("patches");
    expect(drive.mock.calls[0]?.[0]).not.toHaveProperty("tests");
    expect(drive.mock.calls[0]?.[0]).not.toHaveProperty("approvalIds");
    expect(state.setPatches).toHaveBeenCalledWith([{ id: "patch-1", runId: "run-1", status: "applied" }]);
    expect(state.setTests).toHaveBeenCalledWith([{ id: "test-1", runId: "run-1", status: "passed" }]);
    expect(state.setThreads).toHaveBeenCalledWith([]);
  });

  it("only lets approval-backed tests and driver-owned steps enter the driver", () => {
    expect(canRunSchedulerDriver(decision("run_tests", ["approval-test"]))).toBe(true);
    expect(canRunSchedulerDriver(decision("run_tests"))).toBe(false);
    expect(canRunSchedulerDriver(decision("run_patch_draft"))).toBe(false);
    expect(canRunSchedulerDriver(decision("run_review"))).toBe(true);
  });
});

function schedulerState() {
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
    activeThread: {
      messages: [
        { body: "User request.", role: "user" },
        { body: "Existing assistant answer.", role: "assistant" },
      ],
    } as never,
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

function decision(kind: string, approvalIds: string[] = []) {
  return {
    approvalIds,
    kind,
    message: "Scheduler decision.",
    patchCount: 0,
    runId: "run-1",
    testCount: 0,
  } as never;
}
