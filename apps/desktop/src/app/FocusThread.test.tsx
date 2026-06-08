import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import { FocusThread } from "./FocusThread";

afterEach(cleanup);

describe("FocusThread patch actions", () => {
  it("shows apply for a proposed patch with a matching approved approval", () => {
    const onApplyPatch = vi.fn();
    renderThread({ approvalStatus: "approved", onApplyPatch });

    fireEvent.click(screen.getByRole("button", { name: "Apply patch" }));

    expect(onApplyPatch).toHaveBeenCalledWith("patch-1");
  });

  it("hides apply for a proposed patch without approved approval", () => {
    renderThread({ approvalStatus: "pending", onApplyPatch: vi.fn() });

    expect(screen.queryByRole("button", { name: "Apply patch" })).toBeNull();
  });
});

describe("FocusThread test actions", () => {
  it("shows run tests only after an applied patch and supported plan command", () => {
    const onRunTests = vi.fn();
    renderThread({
      activePlan: plan(),
      approvalStatus: "pending",
      onApplyPatch: vi.fn(),
      onRunTests,
      patches: [patch("applied")],
    });

    fireEvent.click(screen.getByRole("button", { name: /Run tests/ }));

    expect(onRunTests).toHaveBeenCalledTimes(1);
  });
});

describe("FocusThread final support", () => {
  it("records final support from an existing assistant answer", () => {
    const onRecordFinal = vi.fn();
    renderThread({
      approvalStatus: "pending",
      messages: [
        { body: "Did it pass?", role: "user" },
        { body: "The patch is ready for review.", role: "assistant" },
      ],
      onApplyPatch: vi.fn(),
      onRecordFinal,
      run: runningRun(),
    });

    fireEvent.click(screen.getByRole("button", { name: /Record final support/ }));

    expect(onRecordFinal).toHaveBeenCalledTimes(1);
  });

  it("shows final support receipt counts when an outcome exists", () => {
    renderThread({
      approvalStatus: "pending",
      onApplyPatch: vi.fn(),
      run: runWithOutcome(),
    });

    expect(screen.getByText("Final support / succeeded")).not.toBeNull();
    expect(screen.getByText("1 evidence receipt(s), 1 passed test receipt(s)")).not.toBeNull();
  });
});

describe("FocusThread live run placement", () => {
  it("places live run activity between the latest user message and assistant reply", () => {
    const { container } = renderThread({
      approvalStatus: "pending",
      messages: [
        { body: "Need a CLI tool", role: "user" },
        { body: "I can help with that.", role: "assistant" },
      ],
      onApplyPatch: vi.fn(),
      run: runningRun(),
    });

    const user = screen.getByText("Need a CLI tool");
    const running = screen.getByText("Running").closest(".focus-activity");
    const assistant = screen.getByText("I can help with that.");

    expect(running).not.toBeNull();
    expect(container.textContent).toContain("Thinking through the latest instruction");
    expect(user.compareDocumentPosition(running as Element) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect((running as Element).compareDocumentPosition(assistant) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });
});

describe("FocusThread artifact visibility", () => {
  it("does not render empty plan, diff, test, or review placeholders", () => {
    renderThread({
      approvalStatus: "pending",
      onApplyPatch: vi.fn(),
      patches: [],
      proposals: [],
    });

    expect(screen.queryByText("No plan yet")).toBeNull();
    expect(screen.queryByText("Unified diff artifact")).toBeNull();
    expect(screen.queryByText("Run review")).toBeNull();
    expect(screen.queryByRole("button", { name: "Apply patch" })).toBeNull();
  });
});

function renderThread({
  activePlan,
  approvalStatus,
  messages,
  onApplyPatch,
  onRecordFinal = vi.fn(),
  onRunTests = vi.fn(),
  patches = [patch()],
  proposals,
  run,
}: {
  activePlan?: PlanView;
  approvalStatus: ActionProposalView["status"];
  messages?: TaskThread["messages"];
  onApplyPatch: (patchId: string) => void;
  onRecordFinal?: () => void;
  onRunTests?: () => void;
  patches?: PatchProposalView[];
  proposals?: ActionProposalView[];
  run?: AgentRunView;
}) {
  return render(
    <FocusThread
      activePlan={activePlan}
      mode="build"
      model="qwen3-coder:30b"
      onApplyPatch={onApplyPatch}
      onApprovePlan={vi.fn()}
      onDecideProposal={vi.fn()}
      onModeChange={vi.fn()}
      onOpenPalette={vi.fn()}
      onRecordFinal={onRecordFinal}
      onRunReview={vi.fn()}
      onRunTests={onRunTests}
      onSend={vi.fn()}
      patches={patches}
      proposals={proposals ?? [approval(approvalStatus)]}
      reviews={[]}
      run={run}
      tests={[]}
      thread={thread(messages)}
    />,
  );
}

function thread(messages: TaskThread["messages"] = [{ body: "Apply this change", role: "user" }]): TaskThread {
  return {
    archived: false,
    createdAt: "2026-06-08T00:00:00.000Z",
    createdLabel: "now",
    goal: "Apply a real patch",
    id: "thread-1",
    messages,
    mode: "build",
    projectId: "project-1",
    runIds: ["run-1"],
    status: "building",
    title: "Patch apply",
    updatedAt: "2026-06-08T00:00:00.000Z",
  };
}

function runningRun(): AgentRunView {
  return {
    artifacts: [],
    createdAt: "2026-06-08T00:00:00.000Z",
    events: [{
      createdAt: "2026-06-08T00:01:00.000Z",
      id: "event-1",
      kind: "model_call.started",
      message: "Ollama request sent to qwen3-coder:30b.",
      runId: "run-1",
    }],
    evidence: [],
    goal: "Need a CLI tool",
    id: "run-1",
    metrics: {
      approvalCount: 0,
      artifactCount: 0,
      commandCount: 0,
      eventCount: 1,
      evidenceCount: 0,
      nodeCount: 1,
    },
    mode: "build",
    nodes: [],
    projectId: "project-1",
    status: "running",
    threadId: "thread-1",
    updatedAt: "2026-06-08T00:01:00.000Z",
  };
}

function runWithOutcome(): AgentRunView {
  return {
    ...runningRun(),
    outcome: {
      evidenceRecordIds: ["evidence-1"],
      status: "succeeded",
      summary: "The patch is ready for review.",
      testArtifactIds: ["test-1"],
    },
    status: "succeeded",
  };
}

function plan(): PlanView {
  return {
    decision: "approved",
    explore: {
      architectureSummary: "TypeScript project.",
      projectCommands: [".\\.tools\\npm.cmd test"],
      relevantFiles: ["src/app.ts"],
      relevantSymbols: [],
      risks: [],
      suggestedNextSteps: [],
      unknowns: [],
    },
    filesLikelyInvolved: ["src/app.ts"],
    goalUnderstanding: "Apply and test the patch.",
    permissionsNeeded: ["approval required before terminal commands"],
    risks: [],
    rollbackStrategy: "Restore checkpoint.",
    steps: ["Apply patch", "Run tests"],
    testsToRun: [".\\.tools\\npm.cmd test"],
    threadId: "thread-1",
  };
}

function patch(status: PatchProposalView["status"] = "proposed"): PatchProposalView {
  return {
    approvalId: "approval-1",
    checkpointFiles: [],
    files: [{
      after: "const value = 2;\n",
      before: "const value = 1;\n",
      diff: [{ kind: "added", text: "const value = 2;" }],
      path: "src/app.ts",
    }],
    id: "patch-1",
    runId: "run-1",
    status,
  };
}

function approval(status: ActionProposalView["status"]): ActionProposalView {
  return {
    actionType: "edit_file",
    expectedResult: "Apply the approved patch.",
    expiresAt: "2026-06-08T00:30:00.000Z",
    id: "approval-1",
    nodeId: "node-1",
    rationale: "User approved the scoped change.",
    requiredPermission: "edit_file",
    riskLabel: "high",
    rollbackPlan: "Restore the checkpoint.",
    runId: "run-1",
    scope: { kind: "file", paths: ["src/app.ts"], projectId: "project-1", summary: "Patch file" },
    status,
  };
}
