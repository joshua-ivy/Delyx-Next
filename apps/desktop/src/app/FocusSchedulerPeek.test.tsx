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
});

function renderScheduler({
  decision,
  onApplyPatch = vi.fn(),
  onResumeRun = vi.fn(),
}: Pick<ComponentProps<typeof FocusSchedulerPeek>, "decision"> & {
  onApplyPatch?: (patchId: string) => void;
  onResumeRun?: () => void;
}) {
  return render(
    <FocusSchedulerPeek
      decision={decision}
      onApplyPatch={onApplyPatch}
      onRecordFinal={vi.fn()}
      onResumeRun={onResumeRun}
      onRunReview={vi.fn()}
      onRunTests={vi.fn()}
    />,
  );
}
