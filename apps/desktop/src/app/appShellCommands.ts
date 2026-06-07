import type { Dispatch, SetStateAction } from "react";

import { notifyLocalAction } from "./ShellPreferenceController";
import { createPlanApprovalProposal, upsertActionProposal } from "./appShellApprovalActions";
import { recordApprovalProposalForRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { modeForThreadStatus, upsertPlan } from "./appShellThreadActions";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { createPlanFromThread } from "../features/plans/planBuilder";
import type { PlanDecision, PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject, WorkspaceUiState } from "../features/workspace/workspaceTypes";

export const paletteCommands = [
  { detail: "Open approved roots, Git facts, and workspace states.", id: "workspace.open", label: "Open workspace" },
  { detail: "Open thread manager for create, archive, and status controls.", id: "threads.open", label: "Open threads" },
  { detail: "Create a read-only plan from the active thread.", id: "plan.create", label: "Create plan" },
  { detail: "Approve the active plan in UI state only.", id: "plan.approve", label: "Approve plan" },
  { detail: "Request revision for the active plan.", id: "plan.revise", label: "Revise plan" },
  { detail: "Cancel the active plan.", id: "plan.cancel", label: "Cancel plan" },
  { detail: "Show the thread empty state without deleting project facts.", id: "state.threads.empty", label: "Show empty threads" },
  { detail: "Show a workspace loading state for acceptance checks.", id: "state.workspace.loading", label: "Show workspace loading" },
  { detail: "Show a workspace error state for acceptance checks.", id: "state.workspace.error", label: "Show workspace error" },
] as const;

export interface AppShellCommandContext {
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setActiveThreadId: Dispatch<SetStateAction<string | undefined>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setPlans: Dispatch<SetStateAction<PlanView[]>>;
  setThreadOpen: Dispatch<SetStateAction<boolean>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  setWorkspaceOpen: Dispatch<SetStateAction<boolean>>;
  setWorkspaceState: Dispatch<SetStateAction<WorkspaceUiState>>;
}

export function runAppShellCommand(commandId: string, context: AppShellCommandContext) {
  switch (commandId) {
    case "workspace.open":
      context.setWorkspaceOpen(true);
      notifyLocalAction("Workspace manager opened");
      break;
    case "threads.open":
      context.setThreadOpen(true);
      notifyLocalAction("Thread manager opened");
      break;
    case "plan.create":
      createPlan(context);
      break;
    case "plan.approve":
      updatePlanDecision(context, "approved", "Plan approved in local UI state");
      break;
    case "plan.revise":
      updatePlanDecision(context, "revision_requested", "Plan revision requested in local UI state");
      break;
    case "plan.cancel":
      updatePlanDecision(context, "cancelled", "Plan cancelled in local UI state");
      break;
    case "state.threads.empty":
      context.setActiveThreadId(undefined);
      context.setAgentRuns([]);
      context.setActionProposals([]);
      context.setPlans([]);
      context.setThreads([]);
      context.setThreadState("empty");
      notifyLocalAction("Thread empty state shown");
      break;
    case "state.workspace.loading":
      context.setWorkspaceOpen(true);
      context.setWorkspaceState("loading");
      notifyLocalAction("Workspace loading state shown");
      break;
    case "state.workspace.error":
      context.setWorkspaceOpen(true);
      context.setWorkspaceState("error");
      notifyLocalAction("Workspace error state shown", "warning");
      break;
  }
}

function createPlan(context: AppShellCommandContext) {
  const activeThread = context.activeThread;
  if (!activeThread) {
    context.setThreadState("empty");
    notifyLocalAction("Create a thread before planning", "warning");
    return;
  }
  context.setPlans((current) => upsertPlan(current, createPlanFromThread(activeThread, context.activeProject)));
  moveThreadAndRunToPlanning(context, activeThread);
  notifyLocalAction("Plan created from active thread", "success");
}

function updatePlanDecision(context: AppShellCommandContext, decision: PlanDecision, message: string) {
  if (!context.activePlan) {
    context.setThreadState(context.activeThread ? "ready" : "empty");
    notifyLocalAction("Create a plan before changing its decision", "warning");
    return;
  }
  context.setPlans((current) => current.map((plan) => (
    plan.threadId === context.activePlan?.threadId ? { ...plan, decision } : plan
  )));
  const activeThread = context.activeThread;
  if (decision === "approved" && activeThread) {
    const proposal = createPlanApprovalProposal(context.activePlan, activeThread, context.activeRun, context.activeProject);
    context.setActionProposals((current) => upsertActionProposal(current, proposal));
    context.setAgentRuns((current) => recordApprovalProposalForRun(current, activeThread, proposal, new Date().toISOString()));
    moveThreadAndRun(context, activeThread, "waiting_for_approval");
  }
  notifyLocalAction(message, "success");
}

function moveThreadAndRunToPlanning(context: AppShellCommandContext, activeThread: TaskThread) {
  moveThreadAndRun(context, activeThread, "planning");
}

function moveThreadAndRun(context: AppShellCommandContext, activeThread: TaskThread, status: ThreadStatus) {
  const now = new Date().toISOString();
  context.setAgentRuns((current) => updateRunsForThreadStatus(current, activeThread, status, now));
  context.setThreads((current) => current.map((thread) => (
    thread.id === activeThread.id ? { ...thread, mode: modeForThreadStatus(status), status, updatedAt: now } : thread
  )));
}
