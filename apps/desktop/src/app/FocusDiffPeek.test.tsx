import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { FocusDiffPeek } from "./FocusDiffPeek";
import { patchApplyApprovalId } from "./patchApplyApproval";
import { patchRestoreApprovalId } from "./patchRestoreApproval";

afterEach(cleanup);

describe("FocusDiffPeek patch actions", () => {
  it("shows apply for a proposed patch with a matching approved apply approval", () => {
    const onPatchAction = vi.fn();
    renderPeek({ onPatchAction, proposals: [approval("approved"), applyApproval("approved")] });

    fireEvent.click(screen.getByRole("button", { name: "Apply patch" }));

    expect(onPatchAction).toHaveBeenCalledWith("patch-1");
  });

  it("requests apply approval before showing the write action", () => {
    const onPatchAction = vi.fn();
    renderPeek({ onPatchAction });

    fireEvent.click(screen.getByRole("button", { name: "Request apply approval" }));

    expect(onPatchAction).toHaveBeenCalledWith("patch-1");
    expect(screen.queryByRole("button", { name: "Apply patch" })).toBeNull();
  });

  it("hides apply for a proposed patch with pending apply approval", () => {
    renderPeek({ proposals: [approval("approved"), applyApproval("pending")] });

    expect(screen.queryByRole("button", { name: "Apply patch" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Request apply approval" })).toBeNull();
  });

  it("requests restore approval for an applied patch with checkpoint receipts", () => {
    const onPatchAction = vi.fn();
    renderPeek({ onPatchAction, patches: [patch("applied")] });

    fireEvent.click(screen.getByRole("button", { name: "Request restore approval" }));

    expect(onPatchAction).toHaveBeenCalledWith("patch-1");
    expect(screen.getByText("modify / applied / checkpoint-1")).not.toBeNull();
    expect(screen.getByText("Checkpoint files: src/app.ts")).not.toBeNull();
    expect(screen.getByText("Restore is allowed only while files still match this applied patch.")).not.toBeNull();
  });

  it("shows restore for an applied patch with a matching approved restore approval", () => {
    const onPatchAction = vi.fn();
    renderPeek({ onPatchAction, patches: [patch("applied")], proposals: [restoreApproval("approved")] });

    fireEvent.click(screen.getByRole("button", { name: "Restore checkpoint" }));

    expect(onPatchAction).toHaveBeenCalledWith("patch-1");
  });

  it("shows restore approval receipts and post-restore guidance", () => {
    renderPeek({ patches: [patch("restored")] });

    expect(screen.getByText("Restore approval: approval-patch-1-restore")).not.toBeNull();
    expect(screen.getByText("Review restored files before continuing.")).not.toBeNull();
  });

  it("shows stale restore failures from the run event stream", () => {
    renderPeek({ patches: [patch("applied")], run: runWithRestoreFailure() });

    expect(screen.getByText("Restore blocked: Patch restore blocked because a file changed since apply.")).not.toBeNull();
  });
});

function renderPeek({
  onPatchAction = vi.fn(),
  patches = [patch()],
  proposals = [approval("approved")],
  run,
}: {
  onPatchAction?: (patchId: string) => void;
  patches?: PatchProposalView[];
  proposals?: ActionProposalView[];
  run?: AgentRunView;
}) {
  return render(<FocusDiffPeek onPatchAction={onPatchAction} patches={patches} proposals={proposals} run={run} />);
}

function patch(status: PatchProposalView["status"] = "proposed"): PatchProposalView {
  const hasCheckpoint = status === "applied" || status === "restored";
  return {
    approvalId: "approval-1",
    checkpointFiles: hasCheckpoint ? [{ contents: "const value = 1;\n", path: "src/app.ts" }] : [],
    checkpointId: hasCheckpoint ? "checkpoint-1" : undefined,
    files: [{
      after: "const value = 2;\n",
      before: "const value = 1;\n",
      changeKind: "modify",
      diff: [{ kind: "added", text: "const value = 2;" }],
      path: "src/app.ts",
    }],
    id: "patch-1",
    restoreApprovalId: status === "restored" ? patchRestoreApprovalId("patch-1") : undefined,
    runId: "run-1",
    status,
  };
}

function runWithRestoreFailure(): AgentRunView {
  return {
    artifacts: [],
    createdAt: "2026-06-08T00:00:00.000Z",
    events: [{
      createdAt: "2026-06-08T00:01:00.000Z",
      id: "event-1",
      kind: "agent_executor.failed",
      message: "Patch restore blocked because a file changed since apply.",
      runId: "run-1",
    }],
    evidence: [],
    goal: "Restore a patch",
    id: "run-1",
    metrics: { approvalCount: 0, artifactCount: 0, commandCount: 0, eventCount: 1, evidenceCount: 0, nodeCount: 0 },
    mode: "build",
    nodes: [],
    status: "failed",
    threadId: "thread-1",
    updatedAt: "2026-06-08T00:01:00.000Z",
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

function applyApproval(status: ActionProposalView["status"]): ActionProposalView {
  return {
    ...approval(status),
    id: patchApplyApprovalId("patch-1"),
    nodeId: "run-1-patch-apply-patch-1",
  };
}

function restoreApproval(status: ActionProposalView["status"]): ActionProposalView {
  return {
    ...approval(status),
    expectedResult: "Restore patch proposal patch-1 from checkpoint receipts.",
    id: patchRestoreApprovalId("patch-1"),
    nodeId: "run-1-patch-restore-patch-1",
  };
}
