import type { Dispatch, SetStateAction } from "react";
import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import {
  previewExternalAgentContract,
  runCodexExternalAgent,
  type ExternalAgentContractKind,
} from "../features/externalAgents/externalAgentClient";
import type { ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { upsertActionProposal } from "./appShellApprovalActions";
import {
  approvedCodexProposal,
  codexApprovalExpired,
  codexNodeId,
  codexSandbox,
  currentCodexProposal,
  type CodexApprovalAction,
  type CodexPermissionMode,
} from "./appShellExternalAgentApprovals";
import { recordApprovalProposalForRun } from "./appShellRunActions";
import { notifyLocalAction } from "./ShellPreferenceController";

export interface ExternalAgentPreviewState {
  actionProposals: ActionProposalView[];
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setExternalAgentState: Dispatch<SetStateAction<ExternalAgentStateView>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function previewExternalAgentContractForRun(
  state: ExternalAgentPreviewState,
  kind: ExternalAgentContractKind,
) {
  if (!state.activeThread || !state.activeRun) {
    state.setThreadState(state.activeThread ? "ready" : "empty");
    notifyLocalAction("Create a thread and AgentRun before previewing an external-agent contract", "warning");
    return;
  }
  try {
    const contract = await previewExternalAgentContract({
      kind,
      permissionMode: "read_only",
      runId: state.activeRun.id,
      task: contractTask(state.activeThread),
      workingDirectory: state.activeProject.path,
    });
    state.setExternalAgentState((current) => ({
      ...current,
      contracts: [contract, ...current.contracts.filter((item) => item.id !== contract.id)],
    }));
    notifyLocalAction(`External-agent contract ready: ${contract.adapterId}`, "success");
  } catch (error) {
    notifyLocalAction(contractPreviewError(error), "warning");
  }
}

export async function runCodexExternalAgentForRun(state: ExternalAgentPreviewState) {
  if (!state.activeThread || !state.activeRun) {
    state.setThreadState(state.activeThread ? "ready" : "empty");
    notifyLocalAction("Create a thread and AgentRun before running Codex", "warning");
    return;
  }
  const approvals = approvedCodexApprovals(state);
  if (!approvals) {
    const result = await queueCodexApprovalProposals(state);
    notifyLocalAction(codexQueueMessage(result), "warning");
    return;
  }
  const permissionMode = permissionModeForState(state);
  const changedFiles = permissionMode === "workspace_write" ? codexChangedFilesFromPlan(state) : [];
  if (permissionMode === "workspace_write" && changedFiles.length === 0) {
    notifyLocalAction("Codex write mode needs planned files so Delyx can checkpoint before launch", "warning");
    return;
  }
  try {
    const artifact = await runCodexExternalAgent({
      allowedPaths: state.activeProject.approvedRoots,
      approvedRoots: state.activeProject.approvedRoots,
      captureDiff: permissionMode === "workspace_write",
      changedFiles,
      checkpointId: undefined,
      createdAtMs: Date.now(),
      externalApprovalId: approvals.external.id,
      permissionMode,
      runId: state.activeRun.id,
      task: contractTask(state.activeThread),
      terminalApprovalId: approvals.terminal.id,
      testArtifactIds: [],
      timeoutMs: 5 * 60 * 1000,
      workingDirectory: state.activeProject.path,
      worktreeId: undefined,
    });
    state.setExternalAgentState((current) => ({
      ...current,
      artifacts: [artifact, ...current.artifacts.filter((item) => item.id !== artifact.id)],
    }));
    notifyLocalAction(`Codex ${permissionMode === "read_only" ? "read-only" : "write-capable"} run captured`, "success");
  } catch (error) {
    notifyLocalAction(externalAgentRunError(error), "warning");
  }
}

function contractTask(thread: TaskThread) {
  const userMessage = [...thread.messages].reverse().find((message) => message.role === "user");
  return userMessage?.body.trim() || thread.goal;
}

async function queueCodexApprovalProposals(state: ExternalAgentPreviewState) {
  if (!state.activeThread || !state.activeRun) {
    return "waiting";
  }
  let denied = false;
  let queued = false;
  const proposals = [
    codexProposalToQueue(state, "external_agent"),
    codexProposalToQueue(state, "run_terminal"),
  ];
  for (const proposal of proposals) {
    if (proposal === "denied") {
      denied = true;
      continue;
    }
    if (!proposal) {
      continue;
    }
    const recorded = await proposeApprovalOverBridge(proposal) ?? proposal;
    state.setActionProposals((current) => upsertActionProposal(current, recorded));
    state.setAgentRuns((current) => recordApprovalProposalForRun(current, state.activeThread!, recorded, new Date().toISOString()));
    queued = true;
  }
  return denied ? "denied" : queued ? "queued" : "waiting";
}

function approvedCodexApprovals(state: ExternalAgentPreviewState) {
  const permissionMode = permissionModeForState(state);
  const external = approvedCodexProposal(state.actionProposals, state.activeRun?.id, "external_agent", permissionMode);
  const terminal = approvedCodexProposal(state.actionProposals, state.activeRun?.id, "run_terminal", permissionMode);
  return external && terminal ? { external, terminal } : undefined;
}

function codexProposalToQueue(
  state: ExternalAgentPreviewState,
  actionType: CodexApprovalAction,
): ActionProposalView | "denied" | undefined {
  const existing = currentCodexProposal(
    state.actionProposals,
    state.activeRun?.id,
    actionType,
    permissionModeForState(state),
  );
  if (existing?.status === "denied") {
    return "denied";
  }
  if (existing && !codexApprovalExpired(existing)) {
    return undefined;
  }
  const fallback = actionType === "external_agent" ? codexExternalProposal(state) : codexTerminalProposal(state);
  return existing ? { ...fallback, id: `${fallback.id}-${Date.now()}` } : fallback;
}

function codexExternalProposal(state: ExternalAgentPreviewState): ActionProposalView {
  const runId = state.activeRun!.id;
  const permissionMode = permissionModeForState(state);
  return {
    actionType: "external_agent",
    expectedResult: "Codex CLI runs only inside the approved project root and returns a captured transcript artifact.",
    expiresAt: expiresAt(),
    id: `approval-${runId}-codex-external-${permissionMode}`,
    nodeId: codexNodeId(runId, "external_agent", permissionMode),
    rationale: `Launch Codex CLI for: ${contractTask(state.activeThread!)}`,
    requiredPermission: "external_agent",
    riskLabel: "high",
    rollbackPlan: "Read-only Codex launches have no file mutation; write-capable launches create a checkpoint before execution.",
    runId,
    scope: {
      kind: "external_agent",
      paths: state.activeProject.approvedRoots,
      projectId: state.activeProject.id,
      root: state.activeProject.path,
      summary: "Launch Codex CLI inside the approved Delyx project root only.",
    },
    status: "pending",
  };
}

function codexTerminalProposal(state: ExternalAgentPreviewState): ActionProposalView {
  const runId = state.activeRun!.id;
  const permissionMode = permissionModeForState(state);
  return {
    actionType: "run_terminal",
    expectedResult: "Run the Codex CLI command with stdout, stderr, exit status, duration, and transcript captured.",
    expiresAt: expiresAt(),
    id: `approval-${runId}-codex-terminal-${permissionMode}`,
    nodeId: codexNodeId(runId, "run_terminal", permissionMode),
    rationale: "Codex CLI execution is a terminal command and must be captured as an AgentRun artifact.",
    requiredPermission: "terminal_command",
    riskLabel: "high",
    rollbackPlan: "Discard the captured command artifact; write-capable Codex runs create checkpoint receipts before execution.",
    runId,
    scope: {
      commands: [`codex exec --json --sandbox ${codexSandbox(permissionMode)} <task>`],
      kind: "terminal",
      projectId: state.activeProject.id,
      root: state.activeProject.path,
      summary: "Run one Codex CLI command from the approved project root.",
    },
    status: "pending",
  };
}

function permissionModeForState(state: ExternalAgentPreviewState): CodexPermissionMode {
  return state.activePlan?.decision === "approved" ? "workspace_write" : "read_only";
}

function codexChangedFilesFromPlan(state: ExternalAgentPreviewState) {
  const root = state.activeProject.path.replace(/[\\/]$/, "");
  const separator = root.includes("\\") && !root.includes("/") ? "\\" : "/";
  const files = new Set<string>();
  for (const file of state.activePlan?.filesLikelyInvolved ?? []) {
    const clean = file.trim();
    if (!clean || clean.includes("*")) {
      continue;
    }
    const path = isAbsolutePath(clean)
      ? clean
      : `${root}${separator}${clean.replace(/^[/\\]+/, "").replace(/[\\/]/g, separator)}`;
    files.add(path);
  }
  return [...files];
}

function expiresAt() {
  return new Date(Date.now() + 30 * 60 * 1000).toISOString();
}

function codexQueueMessage(result: "denied" | "queued" | "waiting") {
  if (result === "denied") {
    return "Codex approval was denied; Delyx will not launch Codex for this run";
  }
  return "Approve the Codex external-agent and terminal-command proposals, then run Codex again";
}

function isAbsolutePath(path: string) {
  return /^[a-zA-Z]:[\\/]/.test(path) || path.startsWith("/") || path.startsWith("\\\\");
}

function contractPreviewError(error: unknown) {
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  return "Desktop bridge is required to preview external-agent contracts.";
}

function externalAgentRunError(error: unknown) {
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  return "Desktop bridge is required to run Codex.";
}
