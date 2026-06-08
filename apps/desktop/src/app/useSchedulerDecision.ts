import { useEffect, useState } from "react";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import {
  scheduleNextRunActionOverBridge,
  type AgentScheduleDecisionView,
} from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { patchDraftApprovalId } from "./appShellPatchDraftDecision";
import { activeTestApprovalId } from "./appShellTestApprovalDecision";
import { firstRunnableTestCommand } from "./testCommand";

export function useSchedulerDecision({
  activePlan,
  activeProject,
  activeRun,
  patches,
  proposals,
  reviews,
  tests,
}: {
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  patches: PatchProposalView[];
  proposals: ActionProposalView[];
  reviews: ReviewReportView[];
  tests: TestArtifactView[];
}) {
  const [decision, setDecision] = useState<AgentScheduleDecisionView | undefined>();

  useEffect(() => {
    if (!activeRun) {
      setDecision(undefined);
      return undefined;
    }
    let cancelled = false;
    void scheduleNextRunActionOverBridge({
      hasSupportedTestCommand: Boolean(firstRunnableTestCommand(activePlan?.testsToRun)),
      nowMs: Date.now(),
      patchDraftApprovalId: patchDraftApprovalId({ actionProposals: proposals, activePlan, activeProject, activeRun, patches }),
      runId: activeRun.id,
      testApprovalId: activeTestApprovalId({ actionProposals: proposals, activePlan, activeRun }),
    }).then((next) => {
      if (!cancelled) {
        setDecision(next);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [
    activePlan?.threadId,
    activePlan?.testsToRun,
    activeProject.indexedFiles,
    activeRun?.id,
    activeRun?.status,
    activeRun?.updatedAt,
    patchSignature(patches),
    proposalSignature(proposals),
    reviewSignature(reviews),
    testSignature(tests),
  ]);

  return decision;
}

function patchSignature(items: PatchProposalView[]) {
  return items.map((item) => `${item.id}:${item.status}:${item.approvalId}`).join("|");
}

function proposalSignature(items: ActionProposalView[]) {
  return items.map((item) => `${item.id}:${item.status}`).join("|");
}

function reviewSignature(items: ReviewReportView[]) {
  return items.map((item) => `${item.id}:${item.decision}`).join("|");
}

function testSignature(items: TestArtifactView[]) {
  return items.map((item) => `${item.id}:${item.status}:${item.exitCode}`).join("|");
}
