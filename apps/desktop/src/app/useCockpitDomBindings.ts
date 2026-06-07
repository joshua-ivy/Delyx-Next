import { useEffect, type Dispatch, type SetStateAction } from "react";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { decideApprovalOverBridge } from "../features/approvals/approvalClient";
import { notifyLocalAction } from "./ShellPreferenceController";
import { recordApprovalDecisionForRun } from "./appShellRunActions";
import { bindComposerForm } from "./cockpitComposerBindings";
import { bindPlanControls, requestPlanRevision } from "./cockpitPlanBindings";
import { updateThreadAndRunStatus } from "./cockpitStateTransitions";

export interface CockpitDomBindingState {
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  actionProposals: ActionProposalView[];
  cockpitHtml: string;
  modelSettings: ModelSettingsView;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setActiveThreadId: Dispatch<SetStateAction<string | undefined>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setPaletteOpen: Dispatch<SetStateAction<boolean>>;
  setPlans: Dispatch<SetStateAction<PlanView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadOpen: Dispatch<SetStateAction<boolean>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  setWorkspaceOpen: Dispatch<SetStateAction<boolean>>;
  threads: TaskThread[];
}

export function useCockpitDomBindings(state: CockpitDomBindingState) {
  useEffect(() => {
    const commandButton = document.querySelector(".command-trigger");
    const projectButton = document.querySelector(".project-trigger") ?? document.querySelector('.rail .rnav[title="Projects"]');
    const threadButton = document.querySelector(".thread-trigger") ?? document.querySelector(".side-h .add");
    const composerForm = document.querySelector(".deck-comp-form");
    const terminalButton = document.querySelector(".deck-termbtn");
    const diffTabs = Array.from(document.querySelectorAll<HTMLElement>(".deck-ftab[data-diff-file]"));
    const approvalApproveButtons = Array.from(document.querySelectorAll<HTMLElement>(".approval-approve-once[data-proposal-id]"));
    const approvalDenyButtons = Array.from(document.querySelectorAll<HTMLElement>(".approval-deny[data-proposal-id]"));
    const reviewReviseButtons = Array.from(document.querySelectorAll(".review-revise"));
    const cards = Array.from(document.querySelectorAll<HTMLElement>(".tcard[data-thread-id]"));
    const openProject = () => state.setWorkspaceOpen(true);
    const openThread = () => state.setThreadOpen(true);
    const openPalette = () => state.setPaletteOpen(true);
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
    const approveProposal = (event: Event) => { void updateProposalStatus(state, event, "approved"); };
    const denyProposal = (event: Event) => { void updateProposalStatus(state, event, "denied"); };
    const cleanupPlanControls = bindPlanControls(state, activateOnKeyboard);
    const cleanupComposer = bindComposerForm(state, composerForm);
    const cleanupTerminal = bindTerminalToggle(terminalButton);
    const cleanupDiffTabs = bindDiffTabs(diffTabs, activateOnKeyboard);
    const requestReviewRevision = () => requestPlanRevision(state);
    bindAccessibility(commandButton, projectButton, threadButton);
    commandButton?.addEventListener("click", openPalette);
    commandButton?.addEventListener("keydown", activateOnKeyboard);
    projectButton?.addEventListener("click", openProject);
    projectButton?.addEventListener("keydown", activateOnKeyboard);
    threadButton?.addEventListener("click", openThread);
    threadButton?.addEventListener("keydown", activateOnKeyboard);
    bindProposalButtons(approvalApproveButtons, approveProposal, activateOnKeyboard);
    bindProposalButtons(approvalDenyButtons, denyProposal, activateOnKeyboard);
    bindReviewButtons(reviewReviseButtons, requestReviewRevision, activateOnKeyboard);
    bindThreadCards(cards, selectThread, activateOnKeyboard);
    return () => {
      commandButton?.removeEventListener("click", openPalette);
      commandButton?.removeEventListener("keydown", activateOnKeyboard);
      projectButton?.removeEventListener("click", openProject);
      projectButton?.removeEventListener("keydown", activateOnKeyboard);
      threadButton?.removeEventListener("click", openThread);
      threadButton?.removeEventListener("keydown", activateOnKeyboard);
      cleanupPlanControls();
      cleanupComposer();
      cleanupTerminal();
      cleanupDiffTabs();
      unbindProposalButtons(approvalApproveButtons, approveProposal, activateOnKeyboard);
      unbindProposalButtons(approvalDenyButtons, denyProposal, activateOnKeyboard);
      unbindReviewButtons(reviewReviseButtons, requestReviewRevision, activateOnKeyboard);
      unbindThreadCards(cards, selectThread, activateOnKeyboard);
    };
  }, [state]);
}

function bindTerminalToggle(button: Element | null) {
  const term = document.querySelector(".deck-term");
  if (!(button instanceof HTMLButtonElement) || !(term instanceof HTMLElement)) {
    return () => undefined;
  }
  const toggle = () => {
    const open = term.hidden;
    term.hidden = !open;
    button.classList.toggle("on", open);
    button.ariaExpanded = `${open}`;
    const label = button.querySelector(".deck-termbtn-x");
    if (label) {
      label.textContent = open ? "hide" : "terminal";
    }
  };
  button.addEventListener("click", toggle);
  return () => button.removeEventListener("click", toggle);
}

function bindDiffTabs(tabs: HTMLElement[], activateOnKeyboard: (event: Event) => void) {
  const select = (event: Event) => {
    const id = (event.currentTarget as HTMLElement).dataset.diffFile;
    if (!id) {
      return;
    }
    document.querySelectorAll<HTMLElement>(".deck-ftab[data-diff-file]").forEach((tab) => {
      tab.classList.toggle("on", tab.dataset.diffFile === id);
    });
    document.querySelectorAll<HTMLElement>(".deck-diff-file-panel[data-diff-file]").forEach((panel) => {
      panel.hidden = panel.dataset.diffFile !== id;
    });
  };
  tabs.forEach((tab) => {
    tab.setAttribute("aria-label", `Show diff for ${tab.textContent?.trim() ?? "file"}`);
    tab.addEventListener("click", select);
    tab.addEventListener("keydown", activateOnKeyboard);
  });
  return () => tabs.forEach((tab) => {
    tab.removeEventListener("click", select);
    tab.removeEventListener("keydown", activateOnKeyboard);
  });
}

async function updateProposalStatus(state: CockpitDomBindingState, event: Event, status: "approved" | "denied") {
  const proposalId = (event.currentTarget as HTMLElement).dataset.proposalId;
  if (!proposalId) {
    return;
  }
  const proposal = state.actionProposals.find((item) => item.id === proposalId);
  if (!proposal) {
    notifyLocalAction("Approval proposal is no longer available", "warning");
    return;
  }
  if (proposal.status !== "pending") {
    notifyLocalAction("Approval decision was already recorded", "warning");
    return;
  }
  if (isExpired(proposal.expiresAt)) {
    state.setActionProposals((current) => current.map((proposal) => (
      proposal.id === proposalId ? { ...proposal, status: "expired" } : proposal
    )));
    notifyLocalAction("Approval proposal is expired; request a fresh approval", "warning");
    return;
  }
  const decidedAtMs = Date.now();
  const decidedProposal = await decideApprovalOverBridge(proposalId, status, decidedAtMs) ?? { ...proposal, status };
  state.setActionProposals((current) => current.map((proposal) => (
    proposal.id === proposalId ? decidedProposal : proposal
  )));
  const activeThread = state.activeThread;
  if (decidedProposal.status === "expired") {
    notifyLocalAction("Approval proposal is expired; request a fresh approval", "warning");
    return;
  }
  if (activeThread) {
    state.setAgentRuns((current) => recordApprovalDecisionForRun(current, activeThread, decidedProposal, new Date(decidedAtMs).toISOString()));
    updateThreadAndRunStatus(state, activeThread, decidedProposal.status === "approved" ? "building" : "blocked");
  }
  notifyLocalAction(decidedProposal.status === "approved" ? "Approval granted for this run" : "Approval denied; run blocked", decidedProposal.status === "approved" ? "success" : "warning");
}

function isExpired(expiresAt: string) {
  const deadline = Date.parse(expiresAt);
  return Number.isFinite(deadline) && deadline <= Date.now();
}

function bindProposalButtons(buttons: HTMLElement[], updateStatus: (event: Event) => void, activateOnKeyboard: (event: Event) => void) {
  buttons.forEach((button) => {
    button.setAttribute("role", "button");
    button.setAttribute("tabindex", "0");
    button.setAttribute("aria-label", button.textContent?.trim() || "Update approval decision");
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

function bindAccessibility(commandButton: Element | null, projectButton: Element | null, threadButton: Element | null) {
  projectButton?.setAttribute("role", "button");
  projectButton?.setAttribute("tabindex", "0");
  projectButton?.setAttribute("aria-label", "Open workspace manager");
  threadButton?.setAttribute("role", "button");
  threadButton?.setAttribute("tabindex", "0");
  threadButton?.setAttribute("aria-label", "Open thread manager");
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
