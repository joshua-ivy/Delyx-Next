import { cockpitMarkup } from "./cockpitMarkup";
import { automationBlock, emptyAutomationBlock } from "./cockpitAutomations";
import { emptyEvidenceBlock, evidenceBlock } from "./cockpitEvidence";
import { emptyExternalAgentBlock, externalAgentBlock } from "./cockpitExternalAgents";
import { emptyMemoryBlock, memoryBlock } from "./cockpitMemory";
import { emptyPipelineBlock, modePill, pipelineBlock } from "./cockpitModes";
import { emptyMobileBlock, mobileBlock } from "./cockpitMobile";
import { emptyModelSettingsBlock, modelSettingsBlock, modelStatusChip } from "./cockpitModels";
import { emptyReleaseBlock, releaseBlock } from "./cockpitRelease";
import { approvalBlock, diffBlock, emptyApprovalBlock, emptyDiffBlock, emptyReviewBlock, emptyTestBlock, pendingCount, reviewBlock, testBlock, testStat } from "./cockpitReview";
import { emptySkillBlock, skillBlock } from "./cockpitSkills";
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
    .replace("/ no-thread", `/ ${activeThread?.id ?? "no-thread"}`)
    .replace('<span class="chip"><span class="k">runtime</span><b>not connected</b> local only</span>', modelStatusChip(modelSettings))
    .replace('<span class="pill build"><span class="dot"></span>BUILD MODE</span>', modePill(activeThread?.status))
    .replace(emptyPipelineBlock(), pipelineBlock(activeThread?.status))
    .replace('<span class="pill ghost">No active run</span>', runStatusPill(activeRun))
    .replace(emptyThreadCardBlock(), threadCards(threads, activeThread?.id))
    .replace("<span class=\"chip\"><span class=\"k\">git</span><b>0</b> uncommitted</span>", gitChip(project))
    .replace("<span class=\"chip\"><span class=\"k\">isolation</span><b>none</b> no checkpoint/worktree</span>", isolationChip(project))
    .replace("C:/Users/geaux/Downloads/Delyx Next", project.approvedRoots[0])
    .replace("THREAD &middot; empty", `THREAD &middot; ${escapeHtml(activeThread?.id ?? "empty")}${runLabel(activeRun)}`)
    .replace("No active thread", escapeHtml(activeThread?.title ?? "No active thread"))
    .replace(emptyThreadGoal(), escapeHtml(activeThread?.goal ?? "Empty: create a thread inside this project to begin."))
    .replace(emptyFilesTouchedStat(), filesTouchedStat(activePatches))
    .replace(emptyDiffStat(), diffStat(activePatches))
    .replace(emptyTestsStat(), testsStat(activeTests))
    .replace(emptyPlanGrid(), planGrid(activePlan, Boolean(activeThread)))
    .replace("0 pending", `${pendingCount(activeProposals)} pending`)
    .replace('<span class="rtab">Approvals<span class="c">0</span></span>', `<span class="rtab">Approvals<span class="c">${pendingCount(activeProposals)}</span></span>`)
    .replace(emptyApprovalBlock(), approvalBlock(activeProposals))
    .replace(emptyDiffBlock(), diffBlock(activePatches))
    .replace(emptyTestBlock(), testBlock(activeTests))
    .replace(emptyReviewBlock(), reviewBlock(activeReview))
    .replace(emptyModelSettingsBlock(), modelSettingsBlock(modelSettings))
    .replace(emptyMemoryBlock(), memoryBlock(memoryState, activeRun?.id))
    .replace(emptySkillBlock(), skillBlock(skillState))
    .replace(emptyAutomationBlock(), automationBlock(automationState))
    .replace(emptyMobileBlock(), mobileBlock(mobileState))
    .replace(emptyReleaseBlock(), releaseBlock(releaseState))
    .replace(emptyExternalAgentBlock(), externalAgentBlock(externalAgents, activeRun?.id))
    .replace(emptyTimelineBlock(), runTimeline(activeRun))
    .replace('<div class="sv">0</div><div class="sk">Evidence</div>', `<div class="sv">${activeRun?.evidenceCount ?? 0}</div><div class="sk">Evidence</div>`)
    .replace(emptyEvidenceBlock(), evidenceBlock(activeResearch))
    .replace("Local only</span><span class=\"pill ghost\">AGENTS.md", `Local only</span><span class="pill ghost">${rulesLabel(project)}`);
}

function emptyFilesTouchedStat() {
  return '<div class="stat"><div class="sv">0</div><div class="sk">Files touched</div></div>';
}

function filesTouchedStat(patches: PatchProposalView[]) {
  const touched = patches.reduce((count, patch) => count + patch.files.length, 0);
  return `<div class="stat"><div class="sv">${touched}</div><div class="sk">Files touched</div></div>`;
}

function emptyDiffStat() {
  return '<div class="stat"><div class="sv">None</div><div class="sk">Diff</div></div>';
}

function diffStat(patches: PatchProposalView[]) {
  const hasDiff = patches.some((patch) => patch.files.length > 0);
  return `<div class="stat"><div class="sv">${hasDiff ? "Ready" : "None"}</div><div class="sk">Diff</div></div>`;
}

function emptyTestsStat() {
  return '<div class="stat"><div class="sv">Not run</div><div class="sk">Tests</div></div>';
}

function testsStat(artifacts: TestArtifactView[]) {
  return `<div class="stat"><div class="sv">${testStat(artifacts)}</div><div class="sk">Tests</div></div>`;
}

function gitChip(project: WorkspaceProject) {
  if (!project.git.isRepo) {
    return "<span class=\"chip\"><span class=\"k\">git</span><b>not repo</b></span>";
  }

  const changes = project.git.uncommittedChanges === null
    ? "changes not loaded"
    : `${project.git.uncommittedChanges} uncommitted`;
  return `<span class="chip"><span class="k">git</span><b>${escapeHtml(project.git.branch)}</b> ${escapeHtml(changes)}</span>`;
}

function isolationChip(project: WorkspaceProject) {
  return `<span class="chip"><span class="k">isolation</span><b>${escapeHtml(project.isolation.mode)}</b> ${escapeHtml(project.isolation.label)}</span>`;
}

function emptyThreadCardBlock() {
  return `      <div class="tcard">
        <div class="tt"><span class="md"></span>Empty: no active threads</div>
        <div class="tm"><span class="dt">Now</span><span class="pill ghost">Idle</span></div>
      </div>`;
}

function emptyThreadGoal() {
  return "Create a thread in this project to start real local work. Runtime execution, approvals, diffs, tests, and evidence stay empty until their ledgers exist.";
}

function emptyPlanGrid() {
  return `<div class="plan-grid">
          <div class="pbox">
            <div class="bh">Files likely to change</div>
            <div class="it"><span class="ix">-</span>No plan has been created.</div>
          </div>
          <div class="pbox">
            <div class="bh">Proposed steps</div>
            <div class="it"><span class="ix">-</span>Create a thread before planning.</div>
          </div>
          <div class="pbox risk">
            <div class="bh">Risks</div>
            <div class="it"><span class="ix">!</span>No risky action has been proposed.</div>
          </div>
          <div class="pbox">
            <div class="bh">Verify and permissions</div>
            <div class="it"><span class="ix">-</span>No test command has been proposed.</div>
            <div class="perm"><span class="pill ghost micro">no approvals pending</span></div>
          </div>
        </div>`;
}

function planGrid(plan: PlanView | undefined, hasThread: boolean) {
  if (!plan) {
    return emptyPlanGrid().replace("Create a thread before planning.", hasThread ? "Create a plan from the active thread." : "Create a thread before planning.");
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

function emptyTimelineBlock() {
  return '<div class="tnode pending"><div class="tr"><span class="kd">empty</span><span class="ms">No AgentRun events have been recorded for this thread.</span><span class="ts">-</span></div></div>';
}

function runTimeline(run: AgentRunView | undefined) {
  if (!run || run.events.length === 0) {
    return emptyTimelineBlock();
  }

  return run.events.map((event) => (
    `<div class="tnode done"><div class="tr"><span class="kd">${escapeHtml(event.kind)}</span><span class="ms">${escapeHtml(event.message)}</span><span class="ts">${escapeHtml(event.id)}</span></div></div>`
  )).join("");
}

function threadCards(threads: TaskThread[], activeThreadId: string | undefined) {
  if (threads.length === 0) {
    return emptyThreadCardBlock();
  }

  return threads.map((thread) => `
      <div class="tcard ${thread.id === activeThreadId ? "on" : ""}" data-thread-id="${escapeHtml(thread.id)}">
        <div class="tt"><span class="md ${statusMarkerClass(thread.status)}"></span>${escapeHtml(thread.title)}</div>
        <div class="tm"><span class="dt">${escapeHtml(thread.createdLabel)}</span>${statusPill(thread.status)}</div>
      </div>`).join("");
}

function runStatusPill(run: AgentRunView | undefined) {
  if (!run) {
    return '<span class="pill ghost">No active run</span>';
  }

  return `<span class="pill wait"><span class="dot"></span>${escapeHtml(run.status)}</span>`;
}

function runLabel(run: AgentRunView | undefined) {
  return run ? ` <span class="pill build micro">${escapeHtml(run.id)}</span>` : "";
}

function statusPill(status: ThreadStatus) {
  const labels: Record<ThreadStatus, string> = {
    blocked: "Blocked",
    building: "Building",
    done: "Done",
    exploring: "Exploring",
    failed: "Failed",
    idle: "Idle",
    planning: "Planning",
    reviewing: "Reviewing",
    testing: "Testing",
    waiting_for_approval: "Waiting",
  };
  const classes: Record<ThreadStatus, string> = {
    blocked: "blocked",
    building: "wait",
    done: "done",
    exploring: "wait",
    failed: "failed",
    idle: "ghost",
    planning: "wait",
    reviewing: "wait",
    testing: "wait",
    waiting_for_approval: "blocked",
  };

  return `<span class="pill ${classes[status]}"><span class="dot"></span>${labels[status]}</span>`;
}

function statusMarkerClass(status: ThreadStatus) {
  const classes: Record<ThreadStatus, string> = {
    blocked: "status-blocked",
    building: "status-active",
    done: "status-done",
    exploring: "status-active",
    failed: "status-failed",
    idle: "status-idle",
    planning: "status-active",
    reviewing: "status-active",
    testing: "status-active",
    waiting_for_approval: "status-blocked",
  };
  return classes[status];
}

function rulesLabel(project: WorkspaceProject) {
  return escapeHtml(project.rulesFiles.map((file) => file.path).join(", ") || "no rules file");
}
