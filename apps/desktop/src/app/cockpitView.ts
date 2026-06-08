import { automationBlock, hasAutomations } from "./cockpitAutomations";
import { cockpitMarkup } from "./cockpitMarkup";
import { evidenceBlock } from "./cockpitEvidence";
import { externalAgentBlock } from "./cockpitExternalAgents";
import { hasMemoryForRun, memoryBlock } from "./cockpitMemory";
import { hasMobileActivity, mobileBlock } from "./cockpitMobile";
import { hasReleaseReadiness, releaseBlock } from "./cockpitRelease";
import { approvalBlock, diffBlock, emptyApprovalBlock, pendingCount, reviewBlock, testBlock } from "./cockpitReview";
import { runLabel } from "./cockpitRuns";
import { hasSkills, skillBlock } from "./cockpitSkills";
import { threadStatsBlock } from "./cockpitStats";
import { buildProgressBlock, composerMode, hintbarBlock, quickActionsBlock, terminalBlock, workDiffBlock } from "./cockpitWorkPane";
import { escapeHtml } from "./html";
import { conversationBlock, threadGoalBlock } from "./cockpitMessageFormat";
import type { RuntimeBridgeState } from "./runtimeBridge";
import { riskTaxonomy, type ActionProposalView, type RiskTaxonomySnapshotView } from "../features/approvals/approvalTypes";
import type { AutomationStateView } from "../features/automations/automationTypes";
import type { ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import type { MemoryStateView } from "../features/memory/memoryTypes";
import type { MobileStateView } from "../features/mobile/mobileTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReleaseStateView } from "../features/release/releaseTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import { researchAnswerFromRunEvidence } from "../features/research/runEvidence";
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
  runtimeBridge: RuntimeBridgeState,
  riskPolicy: RiskTaxonomySnapshotView = riskTaxonomy,
) {
  const activeProposals = activeRun ? proposals.filter((proposal) => proposal.runId === activeRun.id) : [];
  const activePatches = activeRun ? patches.filter((patch) => patch.runId === activeRun.id) : [];
  const activeTests = activeRun ? tests.filter((artifact) => artifact.runId === activeRun.id) : [];
  const activeReview = activeRun ? reviews.find((report) => report.runId === activeRun.id) : undefined;
  const activeEvidence = activeRun
    ? researchAnswers.find((answer) => answer.runId === activeRun.id) ?? researchAnswerFromRunEvidence(activeRun)
    : undefined;
  const activeMemory = hasMemoryForRun(memoryState, activeRun?.id)
    ? memoryBlock(memoryState, activeRun?.id)
    : undefined;
  const activeSkills = hasSkills(skillState) ? skillBlock(skillState) : undefined;
  const activeAutomations = hasAutomations(automationState)
    ? automationBlock(automationState)
    : undefined;
  const activeMobile = hasMobileActivity(mobileState) ? mobileBlock(mobileState) : undefined;
  const activeRelease = hasReleaseReadiness(releaseState)
    ? releaseBlock(releaseState)
    : undefined;
  const externalAgentStream = externalAgentBlock(externalAgents, activeRun?.id);

  return cockpitMarkup
    .replace("__EMPTY_CLASS__", activeThread ? "" : "deck-empty")
    .replace("__SPINE_PIPE__", spinePipeline(activeThread?.status))
    .replace("__MODE_LABEL__", modeLabel(activeThread?.mode))
    .replace("__STATUS_PILL__", barStatusPill(activeThread, activeRun, activeProposals))
    .replace("__CONTEXT_CHIPS__", contextChips(project, modelSettings, threads.length, runtimeBridge, Boolean(activeThread)))
    .replace("__THREAD_ID__", escapeHtml(activeThread?.id ?? "new"))
    .replace("__RUN_LABEL__", runLabel(activeRun))
    .replace("__THREAD_TITLE__", escapeHtml(activeThread?.title ?? "Start with an instruction"))
    .replace("__THREAD_DESC__", threadGoalBlock(activeThread))
    .replace("__CONVERSATION__", conversationBlock(activeThread))
    .replace("__THREAD_STATS__", threadStatsBlock(activePatches, activeTests, activeProposals, activeRun))
    .replace("__BUILD_PROGRESS__", buildProgressBlock(activePlan, activeThread))
    .replace("__WORK_DIFF__", workDiffBlock(activePatches))
    .replace("__TERMINAL_BLOCK__", terminalBlock(activeRun, externalAgentStream))
    .replace("__QUICK_ACTIONS__", quickActionsBlock(activeThread))
    .replace("__COMPOSER_MODE__", composerMode(activeThread?.mode))
    .replace("__INSPECTOR_LABEL__", inspectorLabel(activeProposals, activeRun))
    .replace("__INSPECTOR_STATUS__", inspectorStatus(activeProposals, activeRun))
    .replace("__INSPECTOR__", activeThread ? inspectorBlock({
      activeProposals,
      activePatches,
      activeEvidence,
      activeMemory,
      activeSkills,
      activeAutomations,
      activeMobile,
      activeRelease,
      activeTests,
      activeReview,
      activeRun,
      riskPolicy,
    }) : "")
    .replace("__HINTBAR__", hintbarBlock(activeRun));
}

function barStatusPill(thread: TaskThread | undefined, run: AgentRunView | undefined, proposals: ActionProposalView[]) {
  const pending = pendingCount(proposals);
  if (pending > 0) {
    return statusPill("warning", "Waiting for approval");
  }
  if (!thread) {
    return "";
  }
  if (!run) {
    return statusPill("accent", "Idle");
  }
  const status: Record<AgentRunView["status"], { label: string; tone: string }> = {
    blocked: { label: "Blocked", tone: "danger" },
    cancelled: { label: "Cancelled", tone: "danger" },
    created: { label: "Ready", tone: "accent" },
    failed: { label: "Failed", tone: "danger" },
    repairing: { label: "Repairing", tone: "warning" },
    running: { label: "Running", tone: "accent" },
    succeeded: { label: "Done", tone: "success" },
    waiting_for_approval: { label: "Waiting for approval", tone: "warning" },
  };
  const item = status[run.status];
  return statusPill(item.tone, item.label);
}

function statusPill(tone: string, label: string) {
  return `<span class="pill deck-bar-status ${tone}"><span class="dot ${tone}"></span>${escapeHtml(label)}</span>`;
}

function contextChips(
  project: WorkspaceProject,
  modelSettings: ModelSettingsView,
  activeThreads: number,
  runtimeBridge: RuntimeBridgeState,
  showRuntime: boolean,
) {
  const provider = modelSettings.providers.find((item) => item.id === modelSettings.selectedProviderId);
  const route = modelSettings.routes.find((item) => item.providerId === provider?.id && item.role === "coding");
  const model = route?.modelId ?? provider?.models[0] ?? "no model";
  const git = project.git.isRepo ? project.git.branch : "not a Git repo";
  const chips = [
    `<span class="deck-ctx-chip"><strong>${escapeHtml(project.name)}</strong> / ${escapeHtml(git)} / ${escapeHtml(gitChanges(project))}</span>`,
    `<span class="deck-ctx-chip"><strong>${activeThreads}</strong> threads / ${escapeHtml(provider?.label ?? "No provider")} / ${escapeHtml(model)}</span>`,
  ];
  if (showRuntime) {
    chips.push(`<span class="deck-ctx-chip"><strong>${escapeHtml(runtimeBridge.mode)}</strong> / ${escapeHtml(runtimeBridge.label)}</span>`);
  }
  return chips.join("");
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
  return mode ? mode.toUpperCase() : "NEW";
}

function inspectorStatus(proposals: ActionProposalView[], run: AgentRunView | undefined) {
  const pending = pendingCount(proposals);
  if (pending > 0) {
    return `${pending} pending`;
  }
  return run ? escapeHtml(run.status) : "idle";
}

function inspectorLabel(proposals: ActionProposalView[], run: AgentRunView | undefined) {
  if (pendingCount(proposals) > 0) {
    return "Needs you now";
  }
  return run ? "Run state" : "Inspector";
}

interface InspectorState {
  activeProposals: ActionProposalView[];
  activePatches: PatchProposalView[];
  activeEvidence: ResearchAnswerView | undefined;
  activeMemory: string | undefined;
  activeSkills: string | undefined;
  activeAutomations: string | undefined;
  activeMobile: string | undefined;
  activeRelease: string | undefined;
  activeTests: TestArtifactView[];
  activeReview: ReviewReportView | undefined;
  activeRun: AgentRunView | undefined;
  riskPolicy: RiskTaxonomySnapshotView;
}

function inspectorBlock(state: InspectorState) {
  const pending = state.activeProposals.filter((proposal) => proposal.status === "pending");
  if (pending.length > 0) {
    return approvalBlock(pending, state.riskPolicy);
  }
  if (state.activeReview) {
    return reviewBlock(state.activeReview);
  }
  if (state.activeTests.length > 0) {
    return testBlock(state.activeTests);
  }
  if (state.activePatches.length > 0) {
    return diffBlock(state.activePatches);
  }
  if (
    state.activeEvidence
    || state.activeMemory
    || state.activeSkills
    || state.activeAutomations
    || state.activeMobile
    || state.activeRelease
  ) {
    return [
      state.activeEvidence ? evidenceBlock(state.activeEvidence) : "",
      state.activeMemory ?? "",
      state.activeSkills ?? "",
      state.activeAutomations ?? "",
      state.activeMobile ?? "",
      state.activeRelease ?? "",
    ].join("");
  }
  if (state.activeRun) {
    const latest = state.activeRun.events.at(-1)?.message ?? "Run created. Waiting for the next real action.";
    return `<div class="appro">
        <div class="at"><span class="pill accent">Run activity</span><span class="meta-id">${escapeHtml(state.activeRun.id)}</span></div>
        <h4>${escapeHtml(state.activeRun.status.replaceAll("_", " "))}</h4>
        <div class="kv"><span class="k">Now</span><span class="v">${escapeHtml(latest)}</span></div>
        <div class="kv"><span class="k">Next</span><span class="v">${escapeHtml(nextRunHint(state.activeRun))}</span></div>
      </div>`;
  }
  return emptyApprovalBlock(state.riskPolicy);
}

function nextRunHint(run: AgentRunView) {
  if (run.status === "running" && run.events.at(-1)?.kind === "model_call.started") {
    return "Waiting for the local model response.";
  }
  if (run.status === "created") {
    return "Create a plan or send another instruction.";
  }
  if (run.status === "waiting_for_approval") {
    return "Review the approval request.";
  }
  if (run.status === "succeeded") {
    return "Review receipts before making final claims.";
  }
  return "Check the active work pane for the next available action.";
}
