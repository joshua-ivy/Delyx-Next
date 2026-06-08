import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
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

function renderThread({
  approvalStatus,
  onApplyPatch,
}: {
  approvalStatus: ActionProposalView["status"];
  onApplyPatch: (patchId: string) => void;
}) {
  return render(
    <FocusThread
      activePlan={undefined}
      mode="build"
      model="qwen3-coder:30b"
      onApplyPatch={onApplyPatch}
      onApprovePlan={vi.fn()}
      onCreatePlan={vi.fn()}
      onDecideProposal={vi.fn()}
      onModeChange={vi.fn()}
      onOpenPalette={vi.fn()}
      onRunReview={vi.fn()}
      onSend={vi.fn()}
      patches={[patch()]}
      proposals={[approval(approvalStatus)]}
      reviews={[]}
      run={undefined}
      tests={[]}
      thread={thread()}
    />,
  );
}

function thread(): TaskThread {
  return {
    archived: false,
    createdAt: "2026-06-08T00:00:00.000Z",
    createdLabel: "now",
    goal: "Apply a real patch",
    id: "thread-1",
    messages: [{ body: "Apply this change", role: "user" }],
    mode: "build",
    projectId: "project-1",
    runIds: ["run-1"],
    status: "building",
    title: "Patch apply",
    updatedAt: "2026-06-08T00:00:00.000Z",
  };
}

function patch(): PatchProposalView {
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
    status: "proposed",
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
