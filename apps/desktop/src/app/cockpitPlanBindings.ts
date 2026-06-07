import type { CockpitDomBindingState } from "./useCockpitDomBindings";
import { notifyLocalAction } from "./ShellPreferenceController";
import {
  createPlanApprovalProposal,
  upsertActionProposal,
} from "./appShellApprovalActions";
import {
  recordApprovalProposalForRun,
  recordPlanQuestionForRun,
} from "./appShellRunActions";
import { upsertPlan } from "./appShellThreadActions";
import { expireRunProposals, updateThreadAndRunStatus } from "./cockpitStateTransitions";
import { createPlanFromThread } from "../features/plans/planBuilder";
import type { PlanDecision } from "../features/plans/planTypes";

type KeyboardActivator = (event: Event) => void;

export function bindPlanControls(state: CockpitDomBindingState, activateOnKeyboard: KeyboardActivator) {
  const planCreate = document.querySelector(".plan-create");
  const planApprove = document.querySelector(".plan-approve");
  const planEdit = document.querySelector(".plan-edit");
  const planQuestion = document.querySelector(".plan-question");
  const planReviewMode = document.querySelector(".plan-review-mode");
  const planRevise = document.querySelector(".plan-revise");
  const planCancel = document.querySelector(".plan-cancel");
  const createPlan = () => {
    const activeThread = state.activeThread;
    if (!activeThread) {
      state.setThreadState("empty");
      notifyLocalAction("Create a thread before planning", "warning");
      return;
    }
    state.setPlans((current) => upsertPlan(current, createPlanFromThread(activeThread, state.activeProject)));
    updateThreadAndRunStatus(state, activeThread, "planning");
  };
  const approvePlan = () => updatePlanDecision(state, "approved");
  const revisePlan = () => requestPlanRevision(state);
  const cancelPlan = () => updatePlanDecision(state, "cancelled");
  const switchToReviewMode = () => {
    if (!state.activeThread) {
      state.setThreadState("empty");
      notifyLocalAction("Create a thread before switching to review", "warning");
      return;
    }
    updateThreadAndRunStatus(state, state.activeThread, "reviewing");
  };
  const askPlanQuestion = () => {
    const activeThread = state.activeThread;
    if (!activeThread) {
      state.setThreadState("empty");
      notifyLocalAction("Create a thread before asking a plan question", "warning");
      return;
    }
    const now = new Date().toISOString();
    state.setAgentRuns((current) => recordPlanQuestionForRun(current, activeThread, now));
    updateThreadAndRunStatus(state, activeThread, "planning");
    notifyLocalAction("Question request recorded locally; no model call ran.", "success");
  };
  const controls = [planCreate, planApprove, planEdit, planQuestion, planReviewMode, planRevise, planCancel];
  bindPlanAccessibility(controls);
  bindPlanButton(planCreate, createPlan, activateOnKeyboard);
  bindPlanButton(planApprove, approvePlan, activateOnKeyboard);
  bindPlanButton(planEdit, revisePlan, activateOnKeyboard);
  bindPlanButton(planQuestion, askPlanQuestion, activateOnKeyboard);
  bindPlanButton(planReviewMode, switchToReviewMode, activateOnKeyboard);
  bindPlanButton(planRevise, revisePlan, activateOnKeyboard);
  bindPlanButton(planCancel, cancelPlan, activateOnKeyboard);
  return () => {
    unbindPlanButton(planCreate, createPlan, activateOnKeyboard);
    unbindPlanButton(planApprove, approvePlan, activateOnKeyboard);
    unbindPlanButton(planEdit, revisePlan, activateOnKeyboard);
    unbindPlanButton(planQuestion, askPlanQuestion, activateOnKeyboard);
    unbindPlanButton(planReviewMode, switchToReviewMode, activateOnKeyboard);
    unbindPlanButton(planRevise, revisePlan, activateOnKeyboard);
    unbindPlanButton(planCancel, cancelPlan, activateOnKeyboard);
  };
}

export function requestPlanRevision(state: CockpitDomBindingState) {
  updatePlanDecision(state, "revision_requested");
}

function updatePlanDecision(state: CockpitDomBindingState, decision: PlanDecision) {
  if (!state.activePlan) {
    state.setThreadState(state.activeThread ? "ready" : "empty");
    notifyLocalAction("Create a plan before changing its decision", "warning");
    return;
  }
  state.setPlans((current) => current.map((plan) => (
    plan.threadId === state.activePlan?.threadId ? { ...plan, decision } : plan
  )));
  const activeThread = state.activeThread;
  if (decision === "approved" && activeThread) {
    const proposal = createPlanApprovalProposal(state.activePlan, activeThread, state.activeRun, state.activeProject);
    state.setActionProposals((current) => upsertActionProposal(current, proposal));
    state.setAgentRuns((current) => recordApprovalProposalForRun(current, activeThread, proposal, new Date().toISOString()));
    updateThreadAndRunStatus(state, activeThread, "waiting_for_approval");
  } else if (activeThread) {
    expireRunProposals(state, activeThread);
    updateThreadAndRunStatus(state, activeThread, decision === "cancelled" ? "blocked" : "planning");
  }
}

function bindPlanAccessibility(planButtons: (Element | null)[]) {
  const labels = ["Create plan", "Approve plan", "Edit plan", "Ask question", "Switch to read-only review", "Revise plan", "Cancel plan"];
  planButtons.forEach((button, index) => {
    button?.setAttribute("role", "button");
    button?.setAttribute("tabindex", "0");
    button?.setAttribute("aria-label", labels[index]);
  });
}

function bindPlanButton(button: Element | null, run: () => void, activateOnKeyboard: KeyboardActivator) {
  button?.addEventListener("click", run);
  button?.addEventListener("keydown", activateOnKeyboard);
}

function unbindPlanButton(button: Element | null, run: () => void, activateOnKeyboard: KeyboardActivator) {
  button?.removeEventListener("click", run);
  button?.removeEventListener("keydown", activateOnKeyboard);
}
