import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import { FocusThread } from "./FocusThread";

afterEach(cleanup);

describe("FocusThread broad runtime states", () => {
  it("renders a pending plan and queues approval from the visible plan block", () => {
    const onApprovePlan = vi.fn();
    renderThread({ activePlan: plan("pending"), onApprovePlan, proposals: [] });

    fireEvent.click(screen.getByRole("button", { name: "Queue approval" }));

    expect(screen.getByText("Plan / pending")).not.toBeNull();
    expect(onApprovePlan).toHaveBeenCalledTimes(1);
  });

  it("renders diff, test, review, and evidence support from real artifacts", () => {
    renderThread({
      activePlan: plan("approved"),
      patches: [patch()],
      reviews: [review("accepted")],
      run: run("failed", { evidence: 1, event: "Test command failed with exit 1." }),
      tests: [testArtifact("failed")],
    });

    expect(screen.getByText("Failed")).not.toBeNull();
    expect(screen.getByText("Test command failed with exit 1.")).not.toBeNull();
    expect(screen.getByText("src/app.ts")).not.toBeNull();
    expect(screen.getByText("npm test / failed")).not.toBeNull();
    expect(screen.getByText("Review / accepted")).not.toBeNull();
    expect(screen.getByText("Untested: 1 evidence receipt(s), 0 passed test receipt(s). No new claims are generated.")).not.toBeNull();
  });

  it("keeps blocked and expired approval states visible without approval actions", () => {
    renderThread({
      proposals: [approval("expired")],
      run: run("blocked", { event: "Patch approval expired before execution." }),
    });

    expect(screen.getByText("Blocked")).not.toBeNull();
    expect(screen.getByText("Patch approval expired before execution.")).not.toBeNull();
    expect(screen.getByText("Expired; request a fresh approval before this can run.")).not.toBeNull();
    expect(screen.queryByRole("button", { name: /Approve once/ })).toBeNull();
  });
});

function renderThread({
  activePlan,
  onApprovePlan = vi.fn(),
  patches = [],
  proposals = [],
  reviews = [],
  run: activeRun = run("running"),
  tests = [],
}: {
  activePlan?: PlanView;
  onApprovePlan?: () => void;
  patches?: PatchProposalView[];
  proposals?: ActionProposalView[];
  reviews?: ReviewReportView[];
  run?: AgentRunView;
  tests?: TestArtifactView[];
} = {}) {
  return render(
    <FocusThread
      activePlan={activePlan}
      mode="build"
      model="qwen3-coder:30b"
      onApplyPatch={vi.fn()}
      onApprovePlan={onApprovePlan}
      onDecideProposal={vi.fn()}
      onModeChange={vi.fn()}
      onOpenPalette={vi.fn()}
      onRecordFinal={vi.fn()}
      onRequestRepair={vi.fn()}
      onResumeRun={vi.fn()}
      onRunReview={vi.fn()}
      onRunTests={vi.fn()}
      onSend={vi.fn()}
      patches={patches}
      proposals={proposals}
      reviews={reviews}
      run={activeRun}
      schedulerDecision={undefined}
      tests={tests}
      thread={thread()}
    />,
  );
}

function thread(): TaskThread {
  return {
    activeRunId: "run-1",
    archived: false,
    createdAt: "2026-06-08T00:00:00.000Z",
    createdLabel: "now",
    goal: "Repair parser",
    id: "thread-1",
    messages: [{ body: "Repair parser", role: "user" }, { body: "Here is the current result.", role: "assistant" }],
    mode: "build",
    projectId: "project-1",
    runIds: ["run-1"],
    status: "building",
    title: "Repair parser",
    updatedAt: "2026-06-08T00:00:00.000Z",
  };
}

function run(status: AgentRunView["status"], { evidence = 0, event = "Working locally." } = {}): AgentRunView {
  return {
    artifacts: [],
    createdAt: "2026-06-08T00:00:00.000Z",
    events: [{ createdAt: "2026-06-08T00:01:00.000Z", id: "event-1", kind: "runtime", message: event, runId: "run-1" }],
    evidence: Array.from({ length: evidence }, (_, index) => ({
      id: `evidence-${index}`,
      retrievedAt: "2026-06-08T00:01:00.000Z",
      runId: "run-1",
      sourceId: "patch-1",
      sourceKind: "diff",
    })),
    goal: "Repair parser",
    id: "run-1",
    metrics: { approvalCount: 0, artifactCount: 0, commandCount: 0, eventCount: 1, evidenceCount: evidence, nodeCount: 0 },
    mode: "build",
    nodes: [],
    projectId: "project-1",
    status,
    threadId: "thread-1",
    updatedAt: "2026-06-08T00:01:00.000Z",
  };
}

function plan(decision: PlanView["decision"]): PlanView {
  return {
    decision,
    explore: { architectureSummary: "React app.", projectCommands: ["npm test"], relevantFiles: ["src/app.ts"], relevantSymbols: [], risks: [], suggestedNextSteps: [], unknowns: [] },
    filesLikelyInvolved: ["src/app.ts"],
    goalUnderstanding: "Repair parser.",
    permissionsNeeded: ["edit_file"],
    risks: [],
    rollbackStrategy: "Restore checkpoint.",
    steps: ["Patch parser", "Run tests"],
    testsToRun: ["npm test"],
    threadId: "thread-1",
  };
}

function patch(): PatchProposalView {
  return {
    approvalId: "approval-1",
    checkpointFiles: [],
    files: [{ after: "const value = 2;\n", before: "const value = 1;\n", changeKind: "modify", diff: [{ kind: "added", text: "const value = 2;" }], path: "src/app.ts" }],
    id: "patch-1",
    runId: "run-1",
    status: "proposed",
  };
}

function testArtifact(status: TestArtifactView["status"]): TestArtifactView {
  return {
    command: "npm test",
    completedAt: "2026-06-08T00:02:00.000Z",
    cwd: "C:/repo",
    durationMs: 120,
    exitCode: status === "passed" ? 0 : 1,
    failureSummary: status === "failed" ? "1 failing test" : undefined,
    id: "test-1",
    runId: "run-1",
    startedAt: "2026-06-08T00:01:59.000Z",
    status,
    stderr: "",
    stdout: "",
  };
}

function review(decision: ReviewReportView["decision"]): ReviewReportView {
  return {
    decision,
    evidenceSummary: "Stored review used diff and test receipts.",
    findings: [],
    id: "review-1",
    mode: "read_only",
    riskSummary: "No findings.",
    runId: "run-1",
    testSummary: "Tests reviewed.",
  };
}

function approval(status: ActionProposalView["status"]): ActionProposalView {
  return {
    actionType: "edit_file",
    expectedResult: "Write one scoped file.",
    expiresAt: "2026-06-08T00:30:00.000Z",
    id: "approval-1",
    nodeId: "node-1",
    rationale: "Patch parser.",
    requiredPermission: "edit_file",
    riskLabel: "high",
    rollbackPlan: "Restore checkpoint.",
    runId: "run-1",
    scope: { kind: "file", paths: ["src/app.ts"], projectId: "project-1", summary: "Edit src/app.ts" },
    status,
  };
}
