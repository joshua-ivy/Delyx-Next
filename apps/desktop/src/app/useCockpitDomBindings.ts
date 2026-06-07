import { useEffect, type Dispatch, type SetStateAction } from "react";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { createPlanFromThread } from "../features/plans/planBuilder";
import type { PlanDecision, PlanView } from "../features/plans/planTypes";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { notifyLocalAction } from "./ShellPreferenceController";
import { createPlanApprovalProposal, upsertActionProposal } from "./appShellApprovalActions";
import { updateRunsForThreadStatus } from "./appShellRunActions";
import { modeForThreadStatus, upsertPlan } from "./appShellThreadActions";

interface CockpitDomBindingState {
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  cockpitHtml: string;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setActiveThreadId: Dispatch<SetStateAction<string | undefined>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setPaletteOpen: Dispatch<SetStateAction<boolean>>;
  setPlans: Dispatch<SetStateAction<PlanView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadOpen: Dispatch<SetStateAction<boolean>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  setWorkspaceOpen: Dispatch<SetStateAction<boolean>>;
}

export function useCockpitDomBindings(state: CockpitDomBindingState) {
  useEffect(() => {
    const commandButton = document.querySelector(".command-trigger");
    const projectButton = document.querySelector('.rail .rnav[title="Projects"]');
    const threadButton = document.querySelector(".side-h .add");
    const planCreate = document.querySelector(".plan-create");
    const planApprove = document.querySelector(".plan-approve");
    const planRevise = document.querySelector(".plan-revise");
    const planCancel = document.querySelector(".plan-cancel");
    const approvalApproveButtons = Array.from(document.querySelectorAll<HTMLElement>(".approval-approve-once[data-proposal-id]"));
    const approvalDenyButtons = Array.from(document.querySelectorAll<HTMLElement>(".approval-deny[data-proposal-id]"));
    const reviewReviseButtons = Array.from(document.querySelectorAll(".review-revise"));
    const cards = Array.from(document.querySelectorAll<HTMLElement>(".tcard[data-thread-id]"));
    const openProject = () => state.setWorkspaceOpen(true);
    const openThread = () => state.setThreadOpen(true);
    const openPalette = () => state.setPaletteOpen(true);
    const createPlan = () => {
      const activeThread = state.activeThread;
      if (!activeThread) {
        state.setThreadState("empty");
        return;
      }
      state.setPlans((current) => upsertPlan(current, createPlanFromThread(activeThread, state.activeProject)));
      updateThreadAndRunStatus(state, activeThread, "planning");
    };
    const updatePlanDecision = (decision: PlanDecision) => {
      if (!state.activePlan) {
        state.setThreadState(state.activeThread ? "ready" : "empty");
        return;
      }
      state.setPlans((current) => current.map((plan) => (
        plan.threadId === state.activePlan?.threadId ? { ...plan, decision } : plan
      )));
      if (decision === "approved" && state.activeThread) {
        const proposal = createPlanApprovalProposal(state.activePlan, state.activeThread, state.activeRun, state.activeProject);
        state.setActionProposals((current) => upsertActionProposal(current, proposal));
        updateThreadAndRunStatus(state, state.activeThread, "waiting_for_approval");
      }
    };
    const selectThread = (event: Event) => {
      const threadId = (event.currentTarget as HTMLElement).dataset.threadId;
      if (threadId) {
        state.setActiveThreadId(threadId);
        state.setThreadState("ready");
      }
    };
    const activateOnKeyboard = (event: Event) => {
      const key = (event as KeyboardEvent).key;
      if (key === "Enter" || key === " ") {
        event.preventDefault();
        (event.currentTarget as HTMLElement).click();
      }
    };
    const approvePlan = () => updatePlanDecision("approved");
    const revisePlan = () => updatePlanDecision("revision_requested");
    const cancelPlan = () => updatePlanDecision("cancelled");
    const approveProposal = (event: Event) => updateProposalStatus(state, event, "approved");
    const denyProposal = (event: Event) => updateProposalStatus(state, event, "denied");
    bindAccessibility(commandButton, projectButton, threadButton, [planCreate, planApprove, planRevise, planCancel]);
    commandButton?.addEventListener("click", openPalette);
    commandButton?.addEventListener("keydown", activateOnKeyboard);
    projectButton?.addEventListener("click", openProject);
    projectButton?.addEventListener("keydown", activateOnKeyboard);
    threadButton?.addEventListener("click", openThread);
    threadButton?.addEventListener("keydown", activateOnKeyboard);
    planCreate?.addEventListener("click", createPlan);
    planCreate?.addEventListener("keydown", activateOnKeyboard);
    planApprove?.addEventListener("click", approvePlan);
    planApprove?.addEventListener("keydown", activateOnKeyboard);
    planRevise?.addEventListener("click", revisePlan);
    planRevise?.addEventListener("keydown", activateOnKeyboard);
    planCancel?.addEventListener("click", cancelPlan);
    planCancel?.addEventListener("keydown", activateOnKeyboard);
    bindProposalButtons(approvalApproveButtons, approveProposal, activateOnKeyboard);
    bindProposalButtons(approvalDenyButtons, denyProposal, activateOnKeyboard);
    bindReviewButtons(reviewReviseButtons, revisePlan, activateOnKeyboard);
    bindThreadCards(cards, selectThread, activateOnKeyboard);
    return () => {
      commandButton?.removeEventListener("click", openPalette);
      commandButton?.removeEventListener("keydown", activateOnKeyboard);
      projectButton?.removeEventListener("click", openProject);
      projectButton?.removeEventListener("keydown", activateOnKeyboard);
      threadButton?.removeEventListener("click", openThread);
      threadButton?.removeEventListener("keydown", activateOnKeyboard);
      planCreate?.removeEventListener("click", createPlan);
      planCreate?.removeEventListener("keydown", activateOnKeyboard);
      planApprove?.removeEventListener("click", approvePlan);
      planApprove?.removeEventListener("keydown", activateOnKeyboard);
      planRevise?.removeEventListener("click", revisePlan);
      planRevise?.removeEventListener("keydown", activateOnKeyboard);
      planCancel?.removeEventListener("click", cancelPlan);
      planCancel?.removeEventListener("keydown", activateOnKeyboard);
      unbindProposalButtons(approvalApproveButtons, approveProposal, activateOnKeyboard);
      unbindProposalButtons(approvalDenyButtons, denyProposal, activateOnKeyboard);
      unbindReviewButtons(reviewReviseButtons, revisePlan, activateOnKeyboard);
      unbindThreadCards(cards, selectThread, activateOnKeyboard);
    };
  }, [state]);
}

function updateProposalStatus(state: CockpitDomBindingState, event: Event, status: "approved" | "denied") {
  const proposalId = (event.currentTarget as HTMLElement).dataset.proposalId;
  if (!proposalId) {
    return;
  }
  state.setActionProposals((current) => current.map((proposal) => (
    proposal.id === proposalId ? { ...proposal, status } : proposal
  )));
  if (state.activeThread) {
    updateThreadAndRunStatus(state, state.activeThread, status === "approved" ? "building" : "blocked");
  }
  notifyLocalAction(status === "approved" ? "Approval granted for this run" : "Approval denied; run blocked", status === "approved" ? "success" : "warning");
}

function updateThreadAndRunStatus(state: CockpitDomBindingState, activeThread: TaskThread, status: ThreadStatus) {
  const now = new Date().toISOString();
  state.setAgentRuns((current) => updateRunsForThreadStatus(current, activeThread, status, now));
  state.setThreads((current) => current.map((thread) => (
    thread.id === activeThread.id ? { ...thread, mode: modeForThreadStatus(status), status, updatedAt: now } : thread
  )));
}

function bindProposalButtons(buttons: HTMLElement[], updateStatus: (event: Event) => void, activateOnKeyboard: (event: Event) => void) {
  buttons.forEach((button) => {
    button.addEventListener("click", updateStatus);
    button.addEventListener("keydown", activateOnKeyboard);
  });
}

function unbindProposalButtons(buttons: HTMLElement[], updateStatus: (event: Event) => void, activateOnKeyboard: (event: Event) => void) {
  buttons.forEach((button) => {
    button.removeEventListener("click", updateStatus);
    button.removeEventListener("keydown", activateOnKeyboard);
  });
}

function bindAccessibility(commandButton: Element | null, projectButton: Element | null, threadButton: Element | null, planButtons: (Element | null)[]) {
  projectButton?.setAttribute("role", "button");
  projectButton?.setAttribute("tabindex", "0");
  projectButton?.setAttribute("aria-label", "Open workspace manager");
  threadButton?.setAttribute("role", "button");
  threadButton?.setAttribute("tabindex", "0");
  threadButton?.setAttribute("aria-label", "Open thread manager");
  planButtons.forEach((button) => {
    button?.setAttribute("role", "button");
    button?.setAttribute("tabindex", "0");
  });
  planButtons[0]?.setAttribute("aria-label", "Create plan");
  planButtons[1]?.setAttribute("aria-label", "Approve plan");
  planButtons[2]?.setAttribute("aria-label", "Revise plan");
  planButtons[3]?.setAttribute("aria-label", "Cancel plan");
  commandButton?.setAttribute("role", "button");
  commandButton?.setAttribute("tabindex", "0");
  commandButton?.setAttribute("aria-label", "Open command palette");
}

function bindReviewButtons(buttons: Element[], revisePlan: () => void, activateOnKeyboard: (event: Event) => void) {
  buttons.forEach((button) => {
    button.setAttribute("role", "button");
    button.setAttribute("tabindex", "0");
    button.setAttribute("aria-label", "Ask Delyx to revise this finding");
    button.addEventListener("click", revisePlan);
    button.addEventListener("keydown", activateOnKeyboard);
  });
}

function unbindReviewButtons(buttons: Element[], revisePlan: () => void, activateOnKeyboard: (event: Event) => void) {
  buttons.forEach((button) => {
    button.removeEventListener("click", revisePlan);
    button.removeEventListener("keydown", activateOnKeyboard);
  });
}

function bindThreadCards(cards: HTMLElement[], selectThread: (event: Event) => void, activateOnKeyboard: (event: Event) => void) {
  cards.forEach((card) => {
    card.setAttribute("role", "button");
    card.setAttribute("tabindex", "0");
    card.addEventListener("click", selectThread);
    card.addEventListener("keydown", activateOnKeyboard);
  });
}

function unbindThreadCards(cards: HTMLElement[], selectThread: (event: Event) => void, activateOnKeyboard: (event: Event) => void) {
  cards.forEach((card) => {
    card.removeEventListener("click", selectThread);
    card.removeEventListener("keydown", activateOnKeyboard);
  });
}
