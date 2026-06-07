import { cockpitMarkup } from "./cockpitMarkup";
import { evidenceBlock } from "./cockpitEvidence";
import { externalAgentBlock } from "./cockpitExternalAgents";
import { approvalBlock, diffBlock, emptyApprovalBlock, pendingCount, reviewBlock, testBlock } from "./cockpitReview";
import { runLabel } from "./cockpitRuns";
import { threadStatsBlock } from "./cockpitStats";
import { buildProgressBlock, composerMode, terminalLabel, workDiffBlock } from "./cockpitWorkPane";
import { escapeHtml } from "./html";
import type { RuntimeBridgeState } from "./runtimeBridge";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import { researchAnswerFromRunEvidence } from "../features/research/runEvidence";
import type { ResearchAnswerView } from "../features/research/researchTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
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
  _memoryState: unknown,
  _skillState: unknown,
  _automationState: unknown,
  _mobileState: unknown,
  _releaseState: unknown,
  threads: TaskThread[],
  runtimeBridge: RuntimeBridgeState,
) {
  const activeProposals = activeRun ? proposals.filter((proposal) => proposal.runId === activeRun.id) : [];
  const activePatches = activeRun ? patches.filter((patch) => patch.runId === activeRun.id) : [];
  const activeTests = activeRun ? tests.filter((artifact) => artifact.runId === activeRun.id) : [];
  const activeReview = activeRun ? reviews.find((report) => report.runId === activeRun.id) : undefined;
  const activeEvidence = activeRun
    ? researchAnswers.find((answer) => answer.runId === activeRun.id) ?? researchAnswerFromRunEvidence(activeRun)
    : undefined;

  return cockpitMarkup
    .replace("__SPINE_PIPE__", spinePipeline(activeThread?.status))
    .replace("__MODE_LABEL__", modeLabel(activeThread?.mode))
    .replace("__STATUS_PILL__", barStatusPill(activeThread, activeRun, activeProposals))
    .replace("__CONTEXT_CHIPS__", contextChips(project, modelSettings, threads.length, runtimeBridge))
    .replace("__THREAD_ID__", escapeHtml(activeThread?.id ?? "empty"))
    .replace("__RUN_LABEL__", runLabel(activeRun))
    .replace("__THREAD_TITLE__", escapeHtml(activeThread?.title ?? "No active thread"))
    .replace("__THREAD_DESC__", escapeHtml(activeThread?.goal ?? emptyThreadGoal()))
    .replace("__CONVERSATION__", conversationBlock(activeThread))
    .replace("__THREAD_STATS__", threadStatsBlock(activePatches, activeTests, activeProposals, activeRun))
    .replace("__BUILD_PROGRESS__", buildProgressBlock(activePlan, activeThread))
    .replace("__WORK_DIFF__", workDiffBlock(activePatches))
    .replace("__TERMINAL_LABEL__", terminalLabel(activeRun))
    .replace("__EXTERNAL_AGENT_STREAM__", externalAgentBlock(externalAgents, activeRun?.id))
    .replace("__COMPOSER_MODE__", composerMode(activeThread?.mode))
    .replace("__INSPECTOR_STATUS__", inspectorStatus(activeProposals, activeRun))
    .replace("__INSPECTOR__", inspectorBlock({
      activeProposals,
      activePatches,
      activeEvidence,
      activeTests,
      activeReview,
      activeRun,
    }));
}

function barStatusPill(thread: TaskThread | undefined, run: AgentRunView | undefined, proposals: ActionProposalView[]) {
  const pending = pendingCount(proposals);
  if (pending > 0) {
    return statusPill("warning", "Waiting for approval");
  }
  if (!thread || !run) {
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
) {
  const provider = modelSettings.providers.find((item) => item.id === modelSettings.selectedProviderId);
  const route = modelSettings.routes.find((item) => item.providerId === provider?.id && item.role === "coding");
  const model = route?.modelId ?? provider?.models[0] ?? "no model";
  const git = project.git.isRepo ? project.git.branch : "not a Git repo";
  return [
    `<span class="deck-ctx-chip"><strong>${escapeHtml(project.name)}</strong> / ${escapeHtml(git)} / ${escapeHtml(gitChanges(project))}</span>`,
    `<span class="deck-ctx-chip"><strong>${activeThreads}</strong> threads / ${escapeHtml(provider?.label ?? "No provider")} / ${escapeHtml(model)}</span>`,
    `<span class="deck-ctx-chip"><strong>${escapeHtml(runtimeBridge.mode)}</strong> / ${escapeHtml(runtimeBridge.label)}</span>`,
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
  return (mode ?? "build").toUpperCase();
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
  activeEvidence: ResearchAnswerView | undefined;
  activeTests: TestArtifactView[];
  activeReview: ReviewReportView | undefined;
  activeRun: AgentRunView | undefined;
}

function inspectorBlock(state: InspectorState) {
  const pending = state.activeProposals.filter((proposal) => proposal.status === "pending");
  if (pending.length > 0) {
    return approvalBlock(pending);
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
  if (state.activeEvidence) {
    return evidenceBlock(state.activeEvidence);
  }
  if (state.activeRun) {
    return `<div class="appro">
        <div class="at"><span class="pill accent">Next action</span><span class="meta-id">${escapeHtml(state.activeRun.id)}</span></div>
        <h4>${escapeHtml(state.activeRun.status.replaceAll("_", " "))}</h4>
        <div class="kv"><span class="k">Run</span><span class="v">${escapeHtml(state.activeRun.goal)}</span></div>
        <div class="kv"><span class="k">Commands</span><span class="v">${state.activeRun.metrics.commandCount}</span></div>
        <div class="kv"><span class="k">Evidence</span><span class="v">${state.activeRun.metrics.evidenceCount}</span></div>
      </div>`;
  }
  return emptyApprovalBlock();
}
