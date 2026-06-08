import { beforeEach, describe, expect, it, vi } from "vitest";

import { type AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";
import { runSchedulerDriver } from "./appShellSchedulerDriveActions";

vi.mock("./appShellFinalAnswerActions", () => ({ recordFinalSupportForActiveThread: vi.fn() }));
vi.mock("./appShellOllamaPatchActions", () => ({ proposeApprovedPlanPatchWithOllama: vi.fn() }));
vi.mock("./appShellPatchActions", () => ({ applyApprovedPatchForActiveRun: vi.fn() }));
vi.mock("./appShellTestActions", () => ({ runTestsForActiveRun: vi.fn() }));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));
vi.mock("../features/runs/agentExecutorClient", () => ({ scheduleNextRunActionOverBridge: vi.fn() }));
vi.mock("./appShellSchedulerDriveActions", async () => {
  const actual = await vi.importActual<typeof import("./appShellSchedulerDriveActions")>("./appShellSchedulerDriveActions");
  return { ...actual, runSchedulerDriver: vi.fn(), driverReloadedState: vi.fn((state) => state) };
});

const runDriver = vi.mocked(runSchedulerDriver);

beforeEach(() => {
  vi.clearAllMocks();
  runDriver.mockResolvedValue(undefined);
});

describe("dispatchSchedulerDecision review step", () => {
  it("runs scheduler review through the Rust driver", async () => {
    const handled = await dispatchSchedulerDecision(state(), decision("run_review"));

    expect(handled).toBe(true);
    expect(runDriver).toHaveBeenCalledWith(expect.objectContaining({ activeRun: { id: "run-1" } }));
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
