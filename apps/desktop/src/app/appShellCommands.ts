import type { Dispatch, SetStateAction } from "react";

import { notifyLocalAction, requestThemeToggle } from "./ShellPreferenceController";
import {
  createPlanApprovalProposal,
  expirePendingProposalsForRun,
  upsertActionProposal,
} from "./appShellApprovalActions";
import { previewExternalAgentContractForRun } from "./appShellExternalAgentActions";
import { recordApprovalProposalForRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { createPlanWithOllama } from "./appShellOllamaPlanActions";
import { modeForThreadStatus } from "./appShellThreadActions";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import { refreshOllamaSettings } from "../features/models/ollamaClient";
import type { ModelSettingsView } from "../features/models/modelTypes";
import type { PlanDecision, PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export const paletteCommands = [
  { detail: "Open approved roots, Git facts, and workspace states.", id: "workspace.open", label: "Open workspace" },
  { detail: "Open thread manager for create, archive, and status controls.", id: "threads.open", label: "Open threads" },
  { detail: "Switch between dark and light Command Deck themes.", id: "theme.toggle", label: "Toggle light / dark" },
  { detail: "Check 127.0.0.1:11434 and load local Ollama models.", id: "models.ollama.refresh", label: "Refresh Ollama models" },
  { detail: "Ask local Ollama to draft a read-only plan from the active thread.", id: "plan.create", label: "Create plan" },
  { detail: "Queue a scoped build approval proposal for the active plan.", id: "plan.approve", label: "Approve plan" },
  { detail: "Request revision for the active plan.", id: "plan.revise", label: "Revise plan" },
  { detail: "Cancel the active plan.", id: "plan.cancel", label: "Cancel plan" },
  { detail: "Preview a read-only Codex CLI command contract without launching it.", id: "external.codex.preview", label: "Preview Codex contract" },
  { detail: "Preview a read-only Claude Code command contract without launching it.", id: "external.claude.preview", label: "Preview Claude contract" },
] as const;

export interface AppShellCommandContext {
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  modelSettings: ModelSettingsView;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setExternalAgentState: Dispatch<SetStateAction<ExternalAgentStateView>>;
  setModelSettings: Dispatch<SetStateAction<ModelSettingsView>>;
  setPlans: Dispatch<SetStateAction<PlanView[]>>;
  setThreadOpen: Dispatch<SetStateAction<boolean>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  setWorkspaceOpen: Dispatch<SetStateAction<boolean>>;
  threads: TaskThread[];
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
      updatePlanDecision(context, "approved", "Plan approval proposal queued");
      break;
    case "plan.revise":
      updatePlanDecision(context, "revision_requested", "Plan revision requested");
      break;
    case "plan.cancel":
      updatePlanDecision(context, "cancelled", "Plan cancelled");
      break;
    case "external.codex.preview":
      void previewExternalAgentContractForRun(context, "codex_cli");
      break;
    case "external.claude.preview":
      void previewExternalAgentContractForRun(context, "claude_code");
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
  void createPlanWithOllama(context);
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

function moveThreadAndRun(context: AppShellCommandContext, activeThread: TaskThread, status: ThreadStatus) {
  const now = new Date().toISOString();
  context.setAgentRuns((current) => updateRunsForThreadStatus(current, activeThread, status, now));
  context.setThreads((current) => current.map((thread) => (
    thread.id === activeThread.id ? { ...thread, mode: modeForThreadStatus(status), status, updatedAt: now } : thread
  )));
}
