import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ExternalAgentRunArtifactView } from "../features/externalAgents/externalAgentTypes";

export const TASK_PREFIX = "Task: ";

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

export function terminalCommandLabel(adapterId: string, write: boolean): string {
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

export function adapterFromScope(terminalCard: ActionProposalView): string {
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
