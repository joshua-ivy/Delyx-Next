import type { ActionProposalView } from "../features/approvals/approvalTypes";
import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import { runExternalAgent } from "../features/externalAgents/externalAgentClient";
import type { ExternalAgentRunArtifactView } from "../features/externalAgents/externalAgentTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import { appendMessage, markThread, type ComposerBindingState } from "./cockpitComposerBindings";
import { notifyLocalAction } from "./ShellPreferenceController";

/**
 * Strong-worker mode: route an instruction to an agentic CLI run (Claude Code /
 * Codex) through the existing approval-gated external-agent bridge. Queueing
 * creates two visible approval cards; launching runs the CLI read-only in the
 * workspace and posts the transcript + result back into the thread as receipts.
 */

const WORKER_TIMEOUT_MS = 600_000; // bounded agentic run: 10 minutes
const APPROVAL_TTL_MS = 15 * 60 * 1000;
const TASK_PREFIX = "Task: ";

export type WorkerMode = "read_only" | "workspace_write";

/**
 * Write-mode tasks declare the files the worker may touch with a `[files: a, b]`
 * tag anywhere in the message. Those exact paths are checkpointed before the run
 * (rollback receipts) and shown on the approval card the user grants.
 */
export function parsePlannedFiles(task: string): { task: string; files: string[] } {
  const match = task.match(/\[files:\s*([^\]]+)\]/i);
  if (!match) {
    return { task: task.trim(), files: [] };
  }
  const files = match[1]
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
  return { task: task.replace(match[0], "").replace(/\s{2,}/g, " ").trim(), files };
}

export function workerLabel(adapterId: string): string {
  if (adapterId === "claude-code") {
    return "Claude Code";
  }
  if (adapterId === "codex-cli") {
    return "Codex";
  }
  return adapterId;
}

export function workerNodeIds(runId: string) {
  return {
    external: `${runId}-worker-external`,
    terminal: `${runId}-worker-terminal`,
  };
}

/** The two worker cards for a run, when both exist. */
export function workerCards(runId: string | undefined, proposals: ActionProposalView[]) {
  if (!runId) {
    return undefined;
  }
  const nodes = workerNodeIds(runId);
  const external = proposals.find((item) => item.nodeId === nodes.external);
  const terminal = proposals.find((item) => item.nodeId === nodes.terminal);
  return external && terminal ? { external, terminal } : undefined;
}

/** Extract the queued task from the external card (it travels in the rationale). */
export function workerTaskFromCard(card: ActionProposalView): string {
  return card.rationale.startsWith(TASK_PREFIX)
    ? card.rationale.slice(TASK_PREFIX.length)
    : card.rationale;
}

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

function terminalCommandLabel(adapterId: string, write: boolean): string {
  if (adapterId === "claude-code") {
    return write ? "claude -p … (acceptEdits)" : "claude -p … (read-only)";
  }
  return write ? "codex exec --sandbox workspace-write …" : "codex exec --sandbox read-only …";
}

/** Which mode a queued worker pair was approved for. */
export function workerModeFromCards(cards: {
  external: ActionProposalView;
  terminal: ActionProposalView;
}): WorkerMode {
  return cards.external.requiredPermission.includes("write access")
    ? "workspace_write"
    : "read_only";
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

/**
 * Cross-check the worker's parser-derived file edits against the planned set.
 * Anything edited outside plan was not checkpointed and must be flagged, never
 * silently absorbed.
 */
export function unplannedEdits(
  artifact: ExternalAgentRunArtifactView,
  plannedFiles: string[],
): string[] {
  if (plannedFiles.length === 0) {
    return [];
  }
  const planned = plannedFiles.map(normalizePath);
  return artifact.transcript
    .filter((event) => event.kind === "file_changed")
    .map((event) => event.message.trim())
    .filter((path) => {
      const changed = normalizePath(path);
      return !planned.some((plan) => plan.endsWith(changed) || changed.endsWith(plan));
    });
}

function normalizePath(path: string): string {
  return path.replaceAll("\\", "/").toLowerCase();
}

/** Prefer the parsed final result, then assistant text, then raw terminal tail. */
export function workerResultText(artifact: ExternalAgentRunArtifactView): string {
  const resultEvent = [...artifact.transcript]
    .reverse()
    .find((event) => event.kind === "stdout" && event.message.startsWith("result: "));
  if (resultEvent) {
    return resultEvent.message.slice("result: ".length).trim();
  }
  const stdout = artifact.transcript
    .filter((event) => event.kind === "stdout")
    .map((event) => event.message)
    .join("\n")
    .trim();
  if (stdout) {
    return tail(stdout);
  }
  const terminal = artifact.terminalOutput.trim();
  return terminal ? tail(terminal) : `Worker finished with status ${artifact.status}.`;
}

function adapterFromScope(terminalCard: ActionProposalView): string {
  const command = terminalCard.scope.commands?.[0] ?? "";
  return command.startsWith("claude") ? "claude-code" : "codex-cli";
}

/** Which adapter a queued worker pair belongs to (derived from the terminal card). */
export function workerAdapterFromCards(cards: {
  external: ActionProposalView;
  terminal: ActionProposalView;
}): string {
  return adapterFromScope(cards.terminal);
}

function tail(text: string, max = 4_000): string {
  return text.length <= max ? text : `…${text.slice(text.length - max)}`;
}
