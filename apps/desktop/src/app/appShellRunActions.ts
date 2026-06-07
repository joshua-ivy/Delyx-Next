import type { AgentRunView, RunMetrics } from "../features/runs/agentRunTypes";
import type { TaskThread } from "../features/threads/threadTypes";

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
