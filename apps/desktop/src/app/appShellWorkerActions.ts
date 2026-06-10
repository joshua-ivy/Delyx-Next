import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import { runExternalAgent } from "../features/externalAgents/externalAgentClient";
import type { ExternalAgentRunArtifactView } from "../features/externalAgents/externalAgentTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import {
  adapterFromScope,
  parsePlannedFiles,
  TASK_PREFIX,
  terminalCommandLabel,
  unplannedEdits,
  workerCards,
  workerLabel,
  workerModeFromCards,
  workerNodeIds,
  workerResultText,
  workerTaskFromCard,
  type WorkerMode,
} from "./appShellWorkerCards";
import { appendMessage, markThread, type ComposerBindingState } from "./cockpitComposerBindings";
import { notifyLocalAction } from "./ShellPreferenceController";

export {
  parsePlannedFiles,
  unplannedEdits,
  workerAdapterFromCards,
  workerCards,
  workerLabel,
  workerModeFromCards,
  workerNodeIds,
  workerResultText,
  workerTaskFromCard,
} from "./appShellWorkerCards";
export type { WorkerMode } from "./appShellWorkerCards";

/**
 * Strong-worker mode: route an instruction to an agentic CLI run (Claude Code /
 * Codex) through the existing approval-gated external-agent bridge. Queueing
 * creates two visible approval cards; launching runs the CLI read-only in the
 * workspace and posts the transcript + result back into the thread as receipts.
 */

const WORKER_TIMEOUT_MS = 600_000; // bounded agentic run: 10 minutes
const APPROVAL_TTL_MS = 15 * 60 * 1000;

/**
 * Queue a worker run: create the external-agent and terminal approval cards
 * (visible in the thread) and tell the user what happens next. Does not launch.
 */
export async function queueWorkerRun(
  state: ComposerBindingState,
  thread: TaskThread,
  adapterId: string,
  rawTask: string,
  mode: WorkerMode = "read_only",
): Promise<void> {
  const runId = thread.activeRunId;
  if (!runId) {
    appendMessage(state, thread.id, { role: "system", body: "Worker run needs an active run for this thread." }, "blocked");
    return;
  }
  const label = workerLabel(adapterId);
  const { task, files } = parsePlannedFiles(rawTask);
  if (mode === "workspace_write" && files.length === 0) {
    appendMessage(
      state,
      thread.id,
      { role: "system", body: "Write-mode worker needs planned files so they can be checkpointed first. Add a tag like `[files: src/parser.rs, src/parser_tests.rs]` to your task and resend." },
      "blocked",
    );
    return;
  }
  const nodes = workerNodeIds(runId);
  const expiresAt = new Date(Date.now() + APPROVAL_TTL_MS).toISOString();
  const root = state.activeProject.path;
  const write = mode === "workspace_write";
  // Planned files are checkpointed/diffed by the backend, so resolve them to
  // absolute paths under the project root.
  const plannedPaths = files.map((file) => `${root}/${file}`.replaceAll("\\", "/"));

  const externalCard: ActionProposalView = {
    id: nodes.external,
    runId,
    nodeId: nodes.external,
    actionType: "external_agent",
    riskLabel: "high",
    requiredPermission: write
      ? `Run ${label} with write access to ${files.length} planned file(s)`
      : `Run ${label} read-only inside the project root`,
    rationale: `${TASK_PREFIX}${task}`,
    expectedResult: write
      ? `${label} edits the planned files; each is checkpointed before the run and the diff is captured for review.`
      : `${label} explores the workspace and reports back; no files are written.`,
    rollbackPlan: write
      ? "Planned files are checkpointed before the run; restore from checkpoint receipts."
      : "Read-only run: nothing to roll back.",
    scope: {
      kind: "external_agent",
      summary: write
        ? `${label} agentic run, write-capable for: ${files.join(", ")}.`
        : `${label} agentic run, read-only, rooted at ${root}.`,
      projectId: state.activeProject.id,
      root,
      paths: write ? plannedPaths : [root],
    },
    expiresAt,
    status: "pending",
  };
  const terminalCard: ActionProposalView = {
    id: nodes.terminal,
    runId,
    nodeId: nodes.terminal,
    actionType: "run_terminal",
    riskLabel: "medium",
    requiredPermission: `Execute the ${label} CLI process`,
    rationale: `The worker runs as a local CLI process with captured output.`,
    expectedResult: "One bounded CLI process; full terminal output is captured as a receipt.",
    scope: {
      kind: "terminal",
      summary: `${label} CLI, working directory ${root}.`,
      projectId: state.activeProject.id,
      root,
      commands: [terminalCommandLabel(adapterId, write)],
    },
    expiresAt,
    status: "pending",
  };

  const [external, terminal] = await Promise.all([
    proposeApprovalOverBridge(externalCard),
    proposeApprovalOverBridge(terminalCard),
  ]);
  if (!external || !terminal) {
    appendMessage(state, thread.id, { role: "system", body: "Worker approvals could not be created (desktop bridge unavailable)." }, "blocked");
    return;
  }
  state.setActionProposals?.((current) => [
    ...current.filter((item) => item.id !== external.id && item.id !== terminal.id),
    external,
    terminal,
  ]);
  appendMessage(
    state,
    thread.id,
    {
      role: "system",
      body: write
        ? `⚙ Strong worker queued — ${label} may edit ${files.length} planned file(s) in ${root} (checkpointed first). Approve both cards, then press Launch worker.`
        : `⚙ Strong worker queued — ${label} will run read-only in ${root}. Approve both cards, then press Launch worker.`,
    },
    "idle",
  );
  notifyLocalAction(`Worker queued: approve to launch ${label}`, "info");
}

/**
 * Launch a queued worker once both cards are approved. Posts the result text as
 * an assistant message and a run-receipt system message.
 */
export async function launchQueuedWorker(
  state: ComposerBindingState,
  thread: TaskThread,
  proposals: ActionProposalView[],
): Promise<void> {
  const runId = thread.activeRunId;
  const cards = workerCards(runId, proposals);
  if (!runId || !cards) {
    notifyLocalAction("No queued worker run for this thread", "warning");
    return;
  }
  if (cards.external.status !== "approved" || cards.terminal.status !== "approved") {
    notifyLocalAction("Approve both worker cards before launching", "warning");
    return;
  }
  const adapterId = adapterFromScope(cards.terminal);
  const label = workerLabel(adapterId);
  const task = workerTaskFromCard(cards.external);
  const mode = workerModeFromCards(cards);
  const write = mode === "workspace_write";
  const plannedFiles = write ? (cards.external.scope.paths ?? []) : [];
  markThread(state, thread.id, "exploring");
  notifyLocalAction(`${label} worker running…`, "info");
  try {
    const artifact = await runExternalAgent(adapterId, {
      runId,
      externalApprovalId: cards.external.id,
      terminalApprovalId: cards.terminal.id,
      task,
      workingDirectory: state.activeProject.path,
      approvedRoots: state.activeProject.approvedRoots,
      allowedPaths: [],
      permissionMode: mode,
      timeoutMs: WORKER_TIMEOUT_MS,
      createdAtMs: Date.now(),
      captureDiff: write,
      changedFiles: plannedFiles,
      testArtifactIds: [],
    });
    postWorkerResult(state, thread, label, artifact, plannedFiles);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    appendMessage(state, thread.id, { role: "system", body: `Worker launch failed: ${message}` }, "blocked");
    notifyLocalAction(`Worker launch failed`, "warning");
  }
}

function postWorkerResult(
  state: ComposerBindingState,
  thread: TaskThread,
  label: string,
  artifact: ExternalAgentRunArtifactView,
  plannedFiles: string[],
): void {
  const failed = artifact.status === "failed";
  const result = workerResultText(artifact);
  appendMessage(
    state,
    thread.id,
    { role: "assistant", body: `**${label} worker result**\n\n${result}` },
    failed ? "blocked" : "idle",
  );
  const unplanned = unplannedEdits(artifact, plannedFiles);
  const receipt = [
    `Worker run ${artifact.status}: ${artifact.transcript.length} transcript event(s).`,
    `Scope: ${artifact.scope}`,
    artifact.diffSummary ? `Diff: ${artifact.diffSummary}` : undefined,
    unplanned.length > 0
      ? `⚠ Edited outside the planned scope (NOT checkpointed — review manually): ${unplanned.join(", ")}`
      : undefined,
    artifact.reviewRequired ? "Review required before these changes count as accepted." : undefined,
  ]
    .filter(Boolean)
    .join("\n");
  appendMessage(
    state,
    thread.id,
    { role: "system", body: receipt },
    failed || unplanned.length > 0 ? "blocked" : "idle",
  );
  notifyLocalAction(`${label} worker ${artifact.status}`, failed ? "warning" : "success");
}
