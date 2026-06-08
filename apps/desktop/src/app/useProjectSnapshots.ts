import { useEffect, type Dispatch, type SetStateAction } from "react";

import { loadPlanSnapshot } from "../features/plans/planClient";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";

interface ProjectSnapshotState {
  projectId: string;
  setActiveThreadId: Dispatch<SetStateAction<string | undefined>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setPlans: Dispatch<SetStateAction<PlanView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export function useProjectSnapshots({
  projectId,
  setActiveThreadId,
  setAgentRuns,
  setPlans,
  setThreads,
  setThreadState,
}: ProjectSnapshotState) {
  useEffect(() => {
    let cancelled = false;
    void Promise.all([
      loadThreadRunSnapshot(projectId),
      loadPlanSnapshot(projectId),
    ]).then(([threadSnapshot, planSnapshot]) => {
      if (cancelled) {
        return;
      }
      if (threadSnapshot && threadSnapshot.threads.length > 0) {
        setThreads((current) => current.length > 0 ? current : threadSnapshot.threads);
        setAgentRuns((current) => current.length > 0 ? current : threadSnapshot.runs);
        setActiveThreadId((current) => current ?? threadSnapshot.threads[0]?.id);
        setThreadState("ready");
      }
      if (planSnapshot && planSnapshot.length > 0) {
        setPlans((current) => current.length > 0 ? current : planSnapshot);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [projectId, setActiveThreadId, setAgentRuns, setPlans, setThreadState, setThreads]);
}
