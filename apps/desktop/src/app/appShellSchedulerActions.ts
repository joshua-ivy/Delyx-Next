import type { Dispatch, SetStateAction } from "react";

import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import { resumeWaitingRunOverBridge } from "../features/runs/agentExecutorClient";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { patchDraftApprovalId } from "./appShellPatchDraftDecision";
import { patchApplyApprovalIdForScheduler } from "./patchApplyApproval";
import { notifyLocalAction } from "./ShellPreferenceController";

interface ResumeRunState {
  actionProposals: ActionProposalView[];
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  patches: PatchProposalView[];
  reviews?: ReviewReportView[];
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function resumeSchedulerRun(state: ResumeRunState) {
  if (!state.activeRun) {
    notifyLocalAction("Create a run before resuming scheduler state", "warning");
    return undefined;
  }
  const decision = await resumeWaitingRunOverBridge({
    hasSupportedTestCommand: false,
    nowMs: Date.now(),
    patchApplyApprovalId: patchApplyApprovalIdForScheduler(state.actionProposals, state.patches),
    patchDraftApprovalId: patchDraftApprovalId({ ...state, reviews: state.reviews ?? [] }),
    runId: state.activeRun.id,
  });
  if (!decision) {
    notifyLocalAction("Desktop bridge is required to resume the run", "warning");
    return undefined;
  }
  const snapshot = await loadThreadRunSnapshot(state.activeProject.id);
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
  state.setThreadState("ready");
  notifyLocalAction(decision.message, successfulDecision(decision.kind) ? "success" : "warning");
  return decision;
}

function successfulDecision(kind: string) {
  return ["ready_for_final_support", "resume_after_approval", "run_patch_apply", "run_patch_draft", "run_review", "run_tests"].includes(kind);
}
