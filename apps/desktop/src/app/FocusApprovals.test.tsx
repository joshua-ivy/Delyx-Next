import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { FocusApprovalBlock, visibleApprovalProposals } from "./FocusApprovals";

afterEach(cleanup);

describe("FocusApprovalBlock", () => {
  it("lets pending approvals be approved or denied", () => {
    const onDecide = vi.fn();
    render(<FocusApprovalBlock onDecideProposal={onDecide} proposals={[approval("pending")]} />);

    fireEvent.click(screen.getByRole("button", { name: /Approve once/ }));
    fireEvent.click(screen.getByRole("button", { name: /Deny/ }));

    expect(onDecide).toHaveBeenNthCalledWith(1, "approval-pending", "approved");
    expect(onDecide).toHaveBeenNthCalledWith(2, "approval-pending", "denied");
  });

  it("shows denied and expired approvals without action buttons", () => {
    render(<FocusApprovalBlock onDecideProposal={vi.fn()} proposals={[approval("denied"), approval("expired")]} />);

    expect(screen.getByText("Denied; Delyx will not execute this action.")).not.toBeNull();
    expect(screen.getByText("Expired; request a fresh approval before this can run.")).not.toBeNull();
    expect(screen.queryByRole("button", { name: /Approve once/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /Deny/ })).toBeNull();
  });

  it("keeps non-approved approvals visible", () => {
    const visible = visibleApprovalProposals([
      approval("approved"),
      approval("pending"),
      approval("denied"),
      approval("expired"),
    ]);

    expect(visible.map((item) => item.status)).toEqual(["pending", "denied", "expired"]);
  });
});

function approval(status: ActionProposalView["status"]): ActionProposalView {
  return {
    actionType: "edit_file",
    expectedResult: "Draft or apply one scoped change.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id: `approval-${status}`,
    nodeId: `node-${status}`,
    rationale: "User-visible approval state.",
    requiredPermission: "edit_file",
    riskLabel: "high",
    rollbackPlan: "Use the checkpoint receipt.",
    runId: "run-1",
    scope: { kind: "file", paths: ["src/main.ts"], projectId: "project-1", summary: "Edit src/main.ts" },
    status,
  };
}
