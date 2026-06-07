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
    await queueCodexApprovalProposals(state);
    notifyLocalAction("Approve the Codex external-agent and terminal-command proposals, then run Codex again", "warning");
    return;
  }
  const permissionMode = permissionModeForState(state);
  const isolation = isolationFromRun(state.activeRun);
  if (permissionMode === "workspace_write" && !isolation) {
    notifyLocalAction("Codex write mode needs a real checkpoint or isolated worktree before launch", "warning");
    return;
  }
  try {
    const artifact = await runCodexExternalAgent({
      allowedPaths: state.activeProject.approvedRoots,
      approvedRoots: state.activeProject.approvedRoots,
      captureDiff: permissionMode === "workspace_write",
      changedFiles: [],
      checkpointId: isolation?.checkpointId,
      createdAtMs: Date.now(),
      externalApprovalId: approvals.external.id,
      permissionMode,
      runId: state.activeRun.id,
      task: contractTask(state.activeThread),
      terminalApprovalId: approvals.terminal.id,
      testArtifactIds: [],
      timeoutMs: 5 * 60 * 1000,
      workingDirectory: state.activeProject.path,
      worktreeId: isolation?.worktreeId,
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
    return;
  }
  const proposals = [
    existingCodexProposal(state, "external_agent") ?? codexExternalProposal(state),
    existingCodexProposal(state, "run_terminal") ?? codexTerminalProposal(state),
  ];
  for (const proposal of proposals) {
    const recorded = await proposeApprovalOverBridge(proposal) ?? proposal;
    state.setActionProposals((current) => upsertActionProposal(current, recorded));
    state.setAgentRuns((current) => recordApprovalProposalForRun(current, state.activeThread!, recorded, new Date().toISOString()));
  }
}

function approvedCodexApprovals(state: ExternalAgentPreviewState) {
  const external = existingCodexProposal(state, "external_agent", "approved");
  const terminal = existingCodexProposal(state, "run_terminal", "approved");
  return external && terminal ? { external, terminal } : undefined;
}

function existingCodexProposal(
  state: ExternalAgentPreviewState,
  actionType: ActionProposalView["actionType"],
  status?: ActionProposalView["status"],
) {
  const runId = state.activeRun?.id;
  if (!runId) {
    return undefined;
  }
  return state.actionProposals.find((proposal) => (
    proposal.runId === runId
    && proposal.actionType === actionType
    && proposal.nodeId === codexNodeId(runId, actionType, permissionModeForState(state))
    && (status ? proposal.status === status : proposal.status === "pending" || proposal.status === "approved")
  ));
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
    rollbackPlan: "Read-only Codex launches have no file mutation; write-capable launches require checkpoint or worktree isolation before execution.",
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
    rollbackPlan: "Discard the captured command artifact; write-capable Codex runs remain blocked until real isolation exists.",
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

function isolationFromRun(run: AgentRunView) {
  const artifact = [...run.artifacts].reverse().find((item) => item.kind === "checkpoint" || item.kind === "worktree");
  const checkpointId = artifact?.kind === "checkpoint" ? artifact.id : metadataString(artifact?.metadata?.checkpointId);
  const worktreeId = artifact?.kind === "worktree" ? artifact.id : metadataString(artifact?.metadata?.worktreeId);
  return checkpointId || worktreeId ? { checkpointId, worktreeId } : undefined;
}

function permissionModeForState(state: ExternalAgentPreviewState) {
  return state.activePlan?.decision === "approved" ? "workspace_write" : "read_only";
}

function codexNodeId(
  runId: string,
  actionType: ActionProposalView["actionType"],
  permissionMode: "read_only" | "workspace_write",
) {
  return `${runId}-codex-${actionType}-${permissionMode}`;
}

function codexSandbox(permissionMode: "read_only" | "workspace_write") {
  return permissionMode === "read_only" ? "read-only" : "workspace-write";
}

function expiresAt() {
  return new Date(Date.now() + 30 * 60 * 1000).toISOString();
}

function metadataString(value: unknown) {
  return typeof value === "string" && value.trim() ? value : undefined;
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
