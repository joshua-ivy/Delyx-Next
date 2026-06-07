import { modeForThreadStatus } from "./appShellThreadActions";
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

function runWithThreadStatus(run: AgentRunView, status: ThreadStatus, updatedAt: string): AgentRunView {
  const events = [...run.events, {
    createdAt: updatedAt,
    id: `${run.id}-event-${run.events.length + 1}`,
    kind: "thread.status_changed",
    message: `Thread moved to ${status}.`,
    payload: { status },
    runId: run.id,
  }];
  return {
    ...run,
    events,
    metrics: { ...run.metrics, eventCount: events.length },
    mode: modeForThreadStatus(status),
    status: runStatusForThreadStatus(status),
    updatedAt,
  };
}
