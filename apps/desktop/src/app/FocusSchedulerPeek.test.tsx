import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { FocusSchedulerPeek } from "./FocusThreadArtifacts";

afterEach(cleanup);

describe("FocusSchedulerPeek", () => {
  it("runs the scheduler-selected patch apply action", () => {
    const onApplyPatch = vi.fn();
    renderScheduler({
      decision: {
        approvalIds: [],
        kind: "run_patch_apply",
        message: "Patch proposal patch-1 is approved and ready to apply.",
        patchCount: 0,
        proposalId: "patch-1",
        runId: "run-1",
        testCount: 0,
      },
      onApplyPatch,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ apply patch/ }));

    expect(onApplyPatch).toHaveBeenCalledWith("patch-1");
  });

  it("queues scheduler-selected patch apply approval requests", () => {
    const onApplyPatch = vi.fn();
    renderScheduler({
      decision: {
        approvalIds: [],
        kind: "request_patch_apply_approval",
        message: "Patch proposal patch-1 needs apply approval before disk write.",
        patchCount: 0,
        proposalId: "patch-1",
        runId: "run-1",
        testCount: 0,
      },
      onApplyPatch,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ approve patch apply/ }));

    expect(onApplyPatch).toHaveBeenCalledWith("patch-1");
  });

  it("runs scheduler-selected patch drafting through resume dispatch", () => {
    const onResumeRun = vi.fn();
    renderScheduler({
      decision: {
        ...decision("run_patch_draft", "Approved plan approval-1 is ready for PatchDraftAgent."),
        approvalIds: ["approval-1"],
      },
      onResumeRun,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ draft patch/ }));

    expect(onResumeRun).toHaveBeenCalledTimes(1);
  });

  it("runs the scheduler-selected resume action", () => {
    const onResumeRun = vi.fn();
    renderScheduler({
      decision: {
        approvalIds: ["approval-1"],
        kind: "resume_after_approval",
        message: "Approval approval-1 is ready; run can resume.",
        patchCount: 0,
        runId: "run-1",
        testCount: 0,
      },
      onResumeRun,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ resume run/ }));

    expect(onResumeRun).toHaveBeenCalledTimes(1);
  });

  it("runs the scheduler-selected test action", () => {
    const onRunTests = vi.fn();
    renderScheduler({
      decision: decision("run_tests", "Tests are ready."),
      onRunTests,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ run tests/ }));

    expect(onRunTests).toHaveBeenCalledTimes(1);
  });

  it("runs the scheduler-selected review action", () => {
    const onRunReview = vi.fn();
    renderScheduler({
      decision: { ...decision("run_review", "Review is ready."), patchCount: 1, testCount: 1 },
      onRunReview,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ run review/ }));

    expect(onRunReview).toHaveBeenCalledTimes(1);
  });

  it("runs the scheduler-selected final support action", () => {
    const onRecordFinal = vi.fn();
    renderScheduler({
      decision: {
        ...decision("ready_for_final_support", "Final support can be recorded."),
        reviewReportId: "review-1",
      },
      onRecordFinal,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ final support/ }));

    expect(onRecordFinal).toHaveBeenCalledTimes(1);
  });

  it("renders repair-requested state without firing actions", () => {
    const onRecordFinal = vi.fn();
    renderScheduler({
      decision: {
        ...decision("repair_requested", "Repair requested from review review-1 finding finding-1."),
        findingId: "finding-1",
        reviewReportId: "review-1",
      },
      onRecordFinal,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ repair requested/ }));

    expect(onRecordFinal).not.toHaveBeenCalled();
  });

  it("renders wait states without triggering scheduler work", () => {
    const actions = {
      onApplyPatch: vi.fn(),
      onRecordFinal: vi.fn(),
      onResumeRun: vi.fn(),
      onRunReview: vi.fn(),
      onRunTests: vi.fn(),
    };
    renderScheduler({
      decision: {
        ...decision("wait_for_approval", "Waiting for one approval."),
        approvalIds: ["approval-1"],
      },
      ...actions,
    });

    fireEvent.click(screen.getByRole("button", { name: /Next \/ wait for approval/ }));

    expect(actions.onApplyPatch).not.toHaveBeenCalled();
    expect(actions.onRecordFinal).not.toHaveBeenCalled();
    expect(actions.onResumeRun).not.toHaveBeenCalled();
    expect(actions.onRunReview).not.toHaveBeenCalled();
    expect(actions.onRunTests).not.toHaveBeenCalled();
  });
});

function renderScheduler({
  decision,
  onApplyPatch = vi.fn(),
  onRecordFinal = vi.fn(),
  onResumeRun = vi.fn(),
  onRunReview = vi.fn(),
  onRunTests = vi.fn(),
}: Pick<ComponentProps<typeof FocusSchedulerPeek>, "decision"> & {
  onApplyPatch?: (patchId: string) => void;
  onRecordFinal?: () => void;
  onResumeRun?: () => void;
  onRunReview?: () => void;
  onRunTests?: () => void;
}) {
  return render(
    <FocusSchedulerPeek
      decision={decision}
      onApplyPatch={onApplyPatch}
      onRecordFinal={onRecordFinal}
      onResumeRun={onResumeRun}
      onRunReview={onRunReview}
      onRunTests={onRunTests}
    />,
  );
}

function decision(
  kind: NonNullable<ComponentProps<typeof FocusSchedulerPeek>["decision"]>["kind"],
  message: string,
) {
  return {
    approvalIds: [],
    kind,
    message,
    patchCount: 0,
    runId: "run-1",
    testCount: 0,
  };
}
