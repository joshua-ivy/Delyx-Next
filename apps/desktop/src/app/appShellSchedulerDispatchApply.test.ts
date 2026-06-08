import { beforeEach, describe, expect, it, vi } from "vitest";

import { loadPatchSnapshot } from "../features/patches/patchClient";
import type { PatchProposalView } from "../features/patches/patchTypes";
import {
  runPatchApplySchedulerStepOverBridge,
  scheduleNextRunActionOverBridge,
  type AgentScheduleDecisionView,
} from "../features/runs/agentExecutorClient";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import { applyApprovedPatchForActiveRun } from "./appShellPatchActions";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";

vi.mock("./appShellFinalAnswerActions", () => ({ recordFinalSupportForActiveThread: vi.fn() }));
vi.mock("./appShellOllamaPatchActions", () => ({ proposeApprovedPlanPatchWithOllama: vi.fn() }));
vi.mock("./appShellPatchActions", () => ({ applyApprovedPatchForActiveRun: vi.fn() }));
vi.mock("./appShellReviewActions", () => ({ runReviewForActiveRun: vi.fn() }));
vi.mock("./appShellTestActions", () => ({ runTestsForActiveRun: vi.fn() }));
vi.mock("../features/patches/patchClient", () => ({ loadPatchSnapshot: vi.fn() }));
vi.mock("../features/threads/threadClient", () => ({ loadThreadRunSnapshot: vi.fn() }));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));
vi.mock("../features/runs/agentExecutorClient", () => ({
  runPatchApplySchedulerStepOverBridge: vi.fn(),
  scheduleNextRunActionOverBridge: vi.fn(),
}));

const applyPatch = vi.mocked(applyApprovedPatchForActiveRun);
const loadPatches = vi.mocked(loadPatchSnapshot);
const loadSnapshot = vi.mocked(loadThreadRunSnapshot);
const runApplyStep = vi.mocked(runPatchApplySchedulerStepOverBridge);
const scheduleNext = vi.mocked(scheduleNextRunActionOverBridge);

beforeEach(() => {
  vi.clearAllMocks();
  runApplyStep.mockResolvedValue({
    message: "Patch proposal patch-1 applied.",
    patchId: "patch-1",
    runId: "run-1",
    status: "completed",
  });
  loadPatches.mockResolvedValue([patchView("applied")]);
  loadSnapshot.mockResolvedValue({ runs: [{ id: "run-1" }] as never, threads: [] });
  scheduleNext.mockResolvedValue(undefined);
});

describe("dispatchSchedulerDecision patch apply", () => {
  it("dispatches approved apply decisions through the Rust scheduler step", async () => {
    const patch = patchView();

    const handled = await dispatchSchedulerDecision(state(patch), {
      ...decision("run_patch_apply"),
      approvalIds: ["approval-apply"],
      proposalId: patch.id,
    });

    expect(handled).toBe(true);
    expect(runApplyStep).toHaveBeenCalledWith(expect.objectContaining({ runId: "run-1" }));
    expect(runApplyStep.mock.calls[0]?.[0]).not.toHaveProperty("approvalId");
    expect(runApplyStep.mock.calls[0]?.[0]).not.toHaveProperty("approvedRoots");
    expect(applyPatch).not.toHaveBeenCalled();
  });

  it("queues apply approval request decisions with the matching patch", async () => {
    const patch = patchView();

    const handled = await dispatchSchedulerDecision(state(patch), {
      ...decision("request_patch_apply_approval"),
      proposalId: patch.id,
    });

    expect(handled).toBe(true);
    expect(applyPatch).toHaveBeenCalledWith(expect.objectContaining({ patch }));
  });
});

function state(patch: PatchProposalView) {
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
    patches: [patch],
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

function patchView(status: PatchProposalView["status"] = "proposed"): PatchProposalView {
  return {
    approvalId: "approval-draft",
    checkpointFiles: [],
    files: [],
    id: "patch-1",
    runId: "run-1",
    status,
  };
}
