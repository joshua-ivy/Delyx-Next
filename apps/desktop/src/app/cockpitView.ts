import { cockpitMarkup } from "./cockpitMarkup";
import { automationBlock } from "./cockpitAutomations";
import { evidenceBlock } from "./cockpitEvidence";
import { externalAgentBlock } from "./cockpitExternalAgents";
import { memoryBlock } from "./cockpitMemory";
import { modePill } from "./cockpitModes";
import { mobileBlock } from "./cockpitMobile";
import { modelSettingsBlock } from "./cockpitModels";
import { releaseBlock } from "./cockpitRelease";
import { approvalBlock, diffBlock, emptyApprovalBlock, pendingCount, reviewBlock, testBlock } from "./cockpitReview";
import { runLabel, runStatusPill, runTimeline } from "./cockpitRuns";
import { threadStatsBlock } from "./cockpitStats";
import { skillBlock } from "./cockpitSkills";
import { escapeHtml } from "./html";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { AutomationStateView } from "../features/automations/automationTypes";
import type { ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import type { MemoryStateView } from "../features/memory/memoryTypes";
import type { MobileStateView } from "../features/mobile/mobileTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReleaseStateView } from "../features/release/releaseTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { ResearchAnswerView } from "../features/research/researchTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { SkillStateView } from "../features/skills/skillTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import type { TaskThread, ThreadStatus } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

const workflow = ["explore", "plan", "build", "test", "review"] as const;

export function buildCockpitMarkup(
  project: WorkspaceProject,
  activeThread: TaskThread | undefined,
  activeRun: AgentRunView | undefined,
  activePlan: PlanView | undefined,
  proposals: ActionProposalView[],
  patches: PatchProposalView[],
  tests: TestArtifactView[],
  reviews: ReviewReportView[],
  modelSettings: ModelSettingsView,
  externalAgents: ExternalAgentStateView,
  researchAnswers: ResearchAnswerView[],
  memoryState: MemoryStateView,
  skillState: SkillStateView,
  automationState: AutomationStateView,
  mobileState: MobileStateView,
  releaseState: ReleaseStateView,
  threads: TaskThread[],
) {
  const activeProposals = activeRun ? proposals.filter((proposal) => proposal.runId === activeRun.id) : [];
  const activePatches = activeRun ? patches.filter((patch) => patch.runId === activeRun.id) : [];
  const activeTests = activeRun ? tests.filter((artifact) => artifact.runId === activeRun.id) : [];
  const activeReview = activeRun ? reviews.find((report) => report.runId === activeRun.id) : undefined;
  const activeResearch = activeRun ? researchAnswers.find((answer) => answer.runId === activeRun.id) : undefined;

  return cockpitMarkup
    .replace("__SPINE_PIPE__", spinePipeline(activeThread?.status))
    .replace("__MODE_LABEL__", modeLabel(activeThread?.mode))
    .replace("__STATUS_PILL__", `${modePill(activeThread?.status)}${runStatusPill(activeRun)}`)
    .replace("__CONTEXT_CHIPS__", contextChips(project, modelSettings, threads.length))
    .replace("__THREAD_ID__", escapeHtml(activeThread?.id ?? "empty"))
    .replace("__RUN_LABEL__", runLabel(activeRun))
    .replace("__THREAD_TITLE__", escapeHtml(activeThread?.title ?? "No active thread"))
    .replace("__THREAD_DESC__", escapeHtml(activeThread?.goal ?? emptyThreadGoal()))
    .replace("__CONVERSATION__", conversationBlock(activeThread))
    .replace("__THREAD_STATS__", threadStatsBlock(activePatches, activeTests, activeProposals, activeRun))
    .replace("__PLAN_STATE__", escapeHtml(activePlan?.decision ?? "Empty"))
    .replace("__PLAN_GRID__", planGrid(activePlan, Boolean(activeThread)))
    .replace("__TIMELINE__", runTimeline(activeRun))
    .replace("__INSPECTOR_STATUS__", inspectorStatus(activeProposals, activeRun))
    .replace("__INSPECTOR__", inspectorBlock({
      activeProposals,
      activePatches,
      activeTests,
      activeReview,
      activeResearch,
      activeRun,
      automationState,
      externalAgents,
      memoryState,
      mobileState,
      modelSettings,
      releaseState,
      skillState,
    }));
}

function contextChips(project: WorkspaceProject, modelSettings: ModelSettingsView, activeThreads: number) {
  const provider = modelSettings.providers.find((item) => item.id === modelSettings.selectedProviderId);
  const git = project.git.isRepo ? `${project.git.branch} / ${gitChanges(project)}` : "not a Git repo";
  return [
    `<span class="deck-ctx-chip"><strong>${escapeHtml(project.name)}</strong> ${escapeHtml(git)}</span>`,
    `<span class="deck-ctx-chip"><strong>${activeThreads}</strong> active threads</span>`,
    `<span class="deck-ctx-chip"><strong>${escapeHtml(provider?.label ?? "No provider")}</strong> ${escapeHtml(provider?.status ?? "not_configured")}</span>`,
  ].join("");
}

function gitChanges(project: WorkspaceProject) {
  return project.git.uncommittedChanges === null ? "changes not loaded" : `${project.git.uncommittedChanges} changes`;
}

function spinePipeline(status: ThreadStatus | undefined) {
  const active = activeStepIndex(status);
  return workflow.map((step, index) => {
    const state = spineState(index, active, status);
    return `<span class="deck-spine-dot ${state}" title="${step}"></span>`;
  }).join("");
}

function spineState(index: number, active: number, status: ThreadStatus | undefined) {
  if (status === "done") {
    return "done";
  }
  if (active < 0) {
    return "";
  }
  if (index < active) {
    return "done";
  }
  return index === active ? "active" : index === active + 1 ? "ready" : "";
}

function activeStepIndex(status: ThreadStatus | undefined) {
  const active: Partial<Record<ThreadStatus, number>> = {
    building: 2,
    exploring: 0,
    planning: 1,
    reviewing: 4,
    testing: 3,
    waiting_for_approval: 1,
  };
  return status ? active[status] ?? -1 : -1;
}

function modeLabel(mode: TaskThread["mode"] | undefined) {
  return (mode ?? "local").toUpperCase();
}

function conversationBlock(thread: TaskThread | undefined) {
  if (!thread) {
    return `<div class="deck-msg system"><div class="deck-msg-bub">Create a thread to start a real local conversation. No model response has been generated.</div></div>`;
  }
  const messages = thread.messages.map((message) => {
    const role = message.role === "user" ? "you" : message.role === "assistant" ? "delyx" : "system";
    const avatar = role === "delyx" ? '<span class="deck-msg-av">D</span>' : "";
    return `<div class="deck-msg ${role}">${avatar}<div class="deck-msg-bub">${escapeHtml(message.body)}</div></div>`;
  }).join("");
  const hasAssistant = thread.messages.some((message) => message.role === "assistant");
  const assistantState = hasAssistant ? "" : `<div class="deck-msg system"><div class="deck-msg-bub">No assistant message has been generated for this thread yet.</div></div>`;
  return `${messages}${assistantState}`;
}

function emptyThreadGoal() {
  return "Create a thread in this project to start real local work. Runtime execution, approvals, diffs, tests, and evidence stay empty until their ledgers exist.";
}

function planGrid(plan: PlanView | undefined, hasThread: boolean) {
  if (!plan) {
    return `<div class="plan-grid">
      ${planBox("Files likely to change", ["No plan has been created."], "-")}
      ${planBox("Proposed steps", [hasThread ? "Create a plan from the active thread." : "Create a thread before planning."], "-")}
      ${planBox("Risks", ["No risky action has been proposed."], "!")}
      ${planBox("Verify and permissions", ["No test command has been proposed."], "-")}
    </div>`;
  }
  return `<div class="plan-grid">
    ${planBox("Goal understanding", [plan.goalUnderstanding], "-")}
    ${planBox("Files likely involved", plan.filesLikelyInvolved, "F")}
    ${planBox("Steps", plan.steps, "S")}
    ${planBox("Risks", plan.risks, "!")}
    ${planBox("Tests to run", plan.testsToRun, "$")}
    ${planBox("Rollback strategy", [plan.rollbackStrategy], "R")}
    <div class="pbox"><div class="bh">Permissions needed</div><div class="perm">${plan.permissionsNeeded.map((item) => `<span class="pill ghost micro">${escapeHtml(item)}</span>`).join("")}</div><div class="it"><span class="ix">D</span>${decisionLabel(plan.decision)}</div></div>
  </div>`;
}

function planBox(title: string, items: string[], marker: string) {
  const visibleItems = items.length > 0 ? items : ["None discovered yet."];
  return `<div class="pbox"><div class="bh">${title}</div>${visibleItems.map((item) => `<div class="it"><span class="ix">${marker}</span>${escapeHtml(item)}</div>`).join("")}</div>`;
}

function decisionLabel(decision: PlanView["decision"]) {
  const labels: Record<PlanView["decision"], string> = {
    approved: "Approved",
    cancelled: "Cancelled",
    pending: "Pending review",
    revision_requested: "Revision requested",
  };
  return labels[decision];
}

function inspectorStatus(proposals: ActionProposalView[], run: AgentRunView | undefined) {
  const pending = pendingCount(proposals);
  if (pending > 0) {
    return `${pending} pending`;
  }
  return run ? escapeHtml(run.status) : "idle";
}

interface InspectorState {
  activeProposals: ActionProposalView[];
  activePatches: PatchProposalView[];
  activeTests: TestArtifactView[];
  activeReview: ReviewReportView | undefined;
  activeResearch: ResearchAnswerView | undefined;
  activeRun: AgentRunView | undefined;
  automationState: AutomationStateView;
  externalAgents: ExternalAgentStateView;
  memoryState: MemoryStateView;
  mobileState: MobileStateView;
  modelSettings: ModelSettingsView;
  releaseState: ReleaseStateView;
  skillState: SkillStateView;
}

function inspectorBlock(state: InspectorState) {
  return [
    state.activeProposals.length > 0 ? approvalBlock(state.activeProposals) : emptyApprovalBlock(),
    diffBlock(state.activePatches),
    testBlock(state.activeTests),
    reviewBlock(state.activeReview),
    modelSettingsBlock(state.modelSettings),
    memoryBlock(state.memoryState, state.activeRun?.id),
    skillBlock(state.skillState),
    automationBlock(state.automationState),
    mobileBlock(state.mobileState),
    releaseBlock(state.releaseState),
    externalAgentBlock(state.externalAgents, state.activeRun?.id),
    evidenceBlock(state.activeResearch),
  ].join("");
}
