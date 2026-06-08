import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { decideApprovalAndMaybeResume, resumeAndDispatchSchedulerRun } from "./appShellApprovalDecisionActions";
import { resumeSchedulerRun } from "./appShellSchedulerActions";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";
import { decideFocusApproval, shouldResumeAfterApprovalDecision } from "./focusApprovalDecision";

vi.mock("./appShellSchedulerActions", () => ({
  resumeSchedulerRun: vi.fn(),
}));

vi.mock("./appShellSchedulerDispatch", () => ({
  dispatchSchedulerDecision: vi.fn(),
}));

vi.mock("./focusApprovalDecision", () => ({
  decideFocusApproval: vi.fn(),
  shouldResumeAfterApprovalDecision: vi.fn(),
}));

const decideApproval = vi.mocked(decideFocusApproval);
const dispatchDecision = vi.mocked(dispatchSchedulerDecision);
const shouldResume = vi.mocked(shouldResumeAfterApprovalDecision);
const resumeRun = vi.mocked(resumeSchedulerRun);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("decideApprovalAndMaybeResume", () => {
  it("resumes through the scheduler when the final approval is ready", async () => {
    const decided = approval("approved");
    const pending = approval("pending");
    decideApproval.mockResolvedValue(decided);
    resumeRun.mockResolvedValue(decision());
    shouldResume.mockReturnValue(true);

    const state = actionState([pending]);
    await decideApprovalAndMaybeResume(state, decided.id, "approved");

    expect(decideApproval).toHaveBeenCalledWith(state, decided.id, "approved");
    expect(resumeRun).toHaveBeenCalledWith(expect.objectContaining({
      actionProposals: [decided],
    }));
    expect(dispatchDecision).toHaveBeenCalledWith(expect.objectContaining({
      actionProposals: [decided],
    }), decision());
  });

  it("does not resume when approval policy says more approvals are pending", async () => {
    const decided = approval("approved");
    decideApproval.mockResolvedValue(decided);
    resumeRun.mockResolvedValue(decision());
    shouldResume.mockReturnValue(false);

    await decideApprovalAndMaybeResume(actionState([decided]), decided.id, "approved");

    expect(resumeRun).not.toHaveBeenCalled();
    expect(dispatchDecision).not.toHaveBeenCalled();
  });

  it("manual resume dispatches the returned scheduler decision", async () => {
    const state = actionState([approval("approved")]);
    resumeRun.mockResolvedValue(decision());

    await resumeAndDispatchSchedulerRun(state);

    expect(resumeRun).toHaveBeenCalledWith(state);
    expect(dispatchDecision).toHaveBeenCalledWith(state, decision());
  });
});

function actionState(actionProposals: ActionProposalView[]) {
  return {
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
    activeRun: undefined,
    activeThread: undefined,
    actionProposals,
    modelSettings: {
      providers: [],
      routes: [],
      selectedProviderId: "ollama-local",
    },
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

function decision() {
  return {
    approvalIds: [],
    kind: "run_tests" as const,
    message: "Tests are next.",
    patchCount: 0,
    runId: "run-1",
    testCount: 0,
  };
}

function approval(status: ActionProposalView["status"]): ActionProposalView {
  return {
    actionType: "edit_file",
    expectedResult: "Do local work.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id: "approval-1",
    nodeId: "node-1",
    rationale: "Approve work.",
    requiredPermission: "write_file",
    riskLabel: "high",
    runId: "run-1",
    scope: { kind: "file", summary: "Edit one file." },
    status,
  };
}
