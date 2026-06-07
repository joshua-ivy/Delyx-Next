import type { Dispatch, SetStateAction } from "react";

import { notifyLocalAction, requestThemeToggle } from "./ShellPreferenceController";
import {
  createPlanApprovalProposal,
  expirePendingProposalsForRun,
  upsertActionProposal,
} from "./appShellApprovalActions";
import { recordApprovalProposalForRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { modeForThreadStatus, upsertPlan } from "./appShellThreadActions";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { refreshOllamaSettings } from "../features/models/ollamaClient";
import type { ModelSettingsView } from "../features/models/modelTypes";
import { createPlanFromThread } from "../features/plans/planBuilder";
import type { PlanDecision, PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export const paletteCommands = [
  { detail: "Open approved roots, Git facts, and workspace states.", id: "workspace.open", label: "Open workspace" },
  { detail: "Open thread manager for create, archive, and status controls.", id: "threads.open", label: "Open threads" },
  { detail: "Switch between dark and light Command Deck themes.", id: "theme.toggle", label: "Toggle light / dark" },
  { detail: "Check 127.0.0.1:11434 and load local Ollama models.", id: "models.ollama.refresh", label: "Refresh Ollama models" },
  { detail: "Create a read-only plan from the active thread.", id: "plan.create", label: "Create plan" },
  { detail: "Approve the active plan in UI state only.", id: "plan.approve", label: "Approve plan" },
  { detail: "Request revision for the active plan.", id: "plan.revise", label: "Revise plan" },
  { detail: "Cancel the active plan.", id: "plan.cancel", label: "Cancel plan" },
  { detail: "Show the thread empty state without deleting project facts.", id: "state.threads.empty", label: "Show empty threads" },
] as const;

export interface AppShellCommandContext {
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  modelSettings: ModelSettingsView;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setActiveThreadId: Dispatch<SetStateAction<string | undefined>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setModelSettings: Dispatch<SetStateAction<ModelSettingsView>>;
  setPlans: Dispatch<SetStateAction<PlanView[]>>;
  setThreadOpen: Dispatch<SetStateAction<boolean>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  setWorkspaceOpen: Dispatch<SetStateAction<boolean>>;
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
    case "theme.toggle":
      requestThemeToggle();
      break;
    case "models.ollama.refresh":
      void refreshOllamaModels(context);
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
  }
}

async function refreshOllamaModels(context: AppShellCommandContext) {
  const settings = await refreshOllamaSettings(context.modelSettings);
  context.setModelSettings(settings);
  const ollama = settings.providers.find((provider) => provider.id === "ollama-local");
  notifyLocalAction(ollama?.status === "ready" ? `Ollama ready: ${ollama.models.length} model(s)` : ollama?.detail ?? "Ollama unavailable", ollama?.status === "ready" ? "success" : "warning");
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
  } else if (activeThread) {
    expireRunProposals(context, activeThread);
    moveThreadAndRun(context, activeThread, decision === "cancelled" ? "blocked" : "planning");
  }
  notifyLocalAction(message, "success");
}

function expireRunProposals(context: AppShellCommandContext, activeThread: TaskThread) {
  const runId = context.activeRun?.id ?? activeThread.activeRunId;
  if (runId) {
    context.setActionProposals((current) => expirePendingProposalsForRun(current, runId));
  }
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
