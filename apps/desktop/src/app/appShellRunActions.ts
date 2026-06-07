import { modeForThreadStatus } from "./appShellThreadActions";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { AgentRunStatus, AgentRunView, RunMetrics } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus } from "../features/threads/threadTypes";

export function createRunForThread(thread: TaskThread, projectId: string, index: number): AgentRunView {
  const runId = `run-${thread.id}-${index}`;
  const now = thread.createdAt;
  return {
    artifacts: [],
    createdAt: now,
    events: [{
      createdAt: now,
      id: `${runId}-event-1`,
      kind: "thread.created",
      message: "Thread created from user goal.",
      payload: { threadId: thread.id },
      runId,
    }],
    evidence: [],
    goal: thread.goal,
    id: runId,
    metrics: metricsWithEvent(),
    mode: thread.mode,
    nodes: [],
    projectId,
    status: "created",
    threadId: thread.id,
    updatedAt: now,
  };
}

export function threadWithRun(thread: TaskThread, run: AgentRunView): TaskThread {
  return {
    ...thread,
    activeRunId: run.id,
    runIds: [...thread.runIds, run.id],
    updatedAt: run.updatedAt,
  };
}

export function updateRunsForThreadStatus(
  runs: AgentRunView[],
  thread: TaskThread,
  status: ThreadStatus,
  updatedAt: string,
) {
  if (!thread.activeRunId) {
    return runs;
  }
  return runs.map((run) => (run.id === thread.activeRunId ? runWithThreadStatus(run, status, updatedAt) : run));
}

export function recordApprovalProposalForRun(
  runs: AgentRunView[],
  thread: TaskThread,
  proposal: ActionProposalView,
  createdAt: string,
) {
  return appendApprovalEvent(runs, thread, proposal, "approval.proposed", `Approval requested for ${proposal.actionType}.`, createdAt);
}

export function recordApprovalDecisionForRun(
  runs: AgentRunView[],
  thread: TaskThread,
  proposal: ActionProposalView,
  createdAt: string,
) {
  return appendApprovalEvent(runs, thread, proposal, `approval.${proposal.status}`, `Approval ${proposal.status} for ${proposal.actionType}.`, createdAt);
}

export function recordPlanQuestionForRun(
  runs: AgentRunView[],
  thread: TaskThread,
  createdAt: string,
) {
  if (!thread.activeRunId) {
    return runs;
  }
  return runs.map((run) => (
    run.id === thread.activeRunId
      ? runWithEvent(run, "plan.question_requested", "Clarifying question requested locally; no model call ran.", { threadId: thread.id }, createdAt)
      : run
  ));
}

export function runStatusForThreadStatus(status: ThreadStatus): AgentRunStatus {
  const statuses: Record<ThreadStatus, AgentRunStatus> = {
    blocked: "blocked",
    building: "running",
    done: "succeeded",
    exploring: "running",
    failed: "failed",
    idle: "created",
    planning: "running",
    reviewing: "running",
    testing: "running",
    waiting_for_approval: "waiting_for_approval",
  };
  return statuses[status];
}

function appendApprovalEvent(
  runs: AgentRunView[],
  thread: TaskThread,
  proposal: ActionProposalView,
  kind: string,
  message: string,
  createdAt: string,
) {
  if (!thread.activeRunId) {
    return runs;
  }
  return runs.map((run) => (run.id === thread.activeRunId ? runWithApprovalEvent(run, proposal, kind, message, createdAt) : run));
}

function metricsWithEvent(): RunMetrics {
  return {
    approvalCount: 0,
    artifactCount: 0,
    commandCount: 0,
    eventCount: 1,
    evidenceCount: 0,
    nodeCount: 0,
  };
}

function runWithApprovalEvent(run: AgentRunView, proposal: ActionProposalView, kind: string, message: string, createdAt: string): AgentRunView {
  if (eventAlreadyRecorded(run, kind, proposal.id)) {
    return run;
  }
  const events = [...run.events, {
    createdAt,
    id: `${run.id}-event-${run.events.length + 1}`,
    kind,
    message,
    nodeId: proposal.nodeId,
    payload: { actionType: proposal.actionType, proposalId: proposal.id, riskLabel: proposal.riskLabel, status: proposal.status },
    runId: run.id,
  }];
  return {
    ...run,
    events,
    metrics: { ...run.metrics, approvalCount: approvalCountForEvents(events), eventCount: events.length },
    updatedAt: createdAt,
  };
}

function runWithThreadStatus(run: AgentRunView, status: ThreadStatus, updatedAt: string): AgentRunView {
  const nextMode = modeForThreadStatus(status);
  const nextStatus = runStatusForThreadStatus(status);
  if (run.mode === nextMode && run.status === nextStatus) {
    return run;
  }
  const updated = runWithEvent(run, "thread.status_changed", `Thread moved to ${status}.`, { status }, updatedAt);
  return {
    ...updated,
    mode: nextMode,
    status: nextStatus,
    updatedAt,
  };
}

function runWithEvent(run: AgentRunView, kind: string, message: string, payload: unknown, createdAt: string): AgentRunView {
  const events = [...run.events, {
    createdAt,
    id: `${run.id}-event-${run.events.length + 1}`,
    kind,
    message,
    payload,
    runId: run.id,
  }];
  return {
    ...run,
    events,
    metrics: { ...run.metrics, eventCount: events.length },
    updatedAt: createdAt,
  };
}

function eventAlreadyRecorded(run: AgentRunView, kind: string, proposalId: string) {
  return run.events.some((event) => event.kind === kind && payloadProposalId(event.payload) === proposalId);
}

function approvalCountForEvents(events: AgentRunView["events"]) {
  return new Set(events.map((event) => payloadProposalId(event.payload)).filter(Boolean)).size;
}

function payloadProposalId(payload: unknown) {
  if (!payload || typeof payload !== "object" || !("proposalId" in payload)) {
    return undefined;
  }
  const proposalId = (payload as { proposalId?: unknown }).proposalId;
  return typeof proposalId === "string" ? proposalId : undefined;
}
