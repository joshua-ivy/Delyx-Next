import { describe, expect, it } from "vitest";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { shouldResumeAfterApprovalDecision } from "./focusApprovalDecision";

describe("shouldResumeAfterApprovalDecision", () => {
  it("allows resume after one approved proposal when no other approval is pending", () => {
    const decided = approval("approval-1", "approved");

    expect(shouldResumeAfterApprovalDecision(decided, [decided], decided.id)).toBe(true);
  });

  it("keeps the run waiting when another approval is still pending", () => {
    const decided = approval("approval-1", "approved");
    const pending = approval("approval-2", "pending");

    expect(shouldResumeAfterApprovalDecision(decided, [decided, pending], decided.id)).toBe(false);
  });

  it("does not resume after a denied approval", () => {
    const decided = approval("approval-1", "denied");

    expect(shouldResumeAfterApprovalDecision(decided, [decided], decided.id)).toBe(false);
  });
});

function approval(id: string, status: ActionProposalView["status"]): ActionProposalView {
  return {
    actionType: "edit_file",
    expectedResult: "Do the approved local work.",
    expiresAt: "2999-01-01T00:00:00.000Z",
    id,
    nodeId: `${id}-node`,
    rationale: "Test approval policy.",
    requiredPermission: "file_write",
    riskLabel: "high",
    rollbackPlan: "Use the existing checkpoint.",
    runId: "run-1",
    scope: {
      kind: "file",
      paths: ["src/example.ts"],
      projectId: "project-1",
      root: "C:\\repo",
      summary: "Edit one file.",
    },
    status,
  };
}
