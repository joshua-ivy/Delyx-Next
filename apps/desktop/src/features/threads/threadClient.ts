import { invoke } from "@tauri-apps/api/core";
import type { AgentRunView } from "../runs/agentRunTypes";
import type { TaskThread, ThreadStatus } from "./threadTypes";

export interface ThreadRunRecordView {
  thread: TaskThread;
  run: AgentRunView;
}

export interface ThreadRunSnapshotView {
  threads: TaskThread[];
  runs: AgentRunView[];
}

export async function createThreadRunOverBridge(
  projectId: string,
  goal: string,
  createdAt: string,
): Promise<ThreadRunRecordView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<ThreadRunRecordView>("thread_run_create", {
      request: { createdAt, goal, projectId },
    });
  } catch {
    return undefined;
  }
}

export async function loadThreadRunSnapshot(projectId: string): Promise<ThreadRunSnapshotView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<ThreadRunSnapshotView>("thread_run_snapshot", { projectId });
  } catch {
    return undefined;
  }
}

export async function updateThreadStatusOverBridge(
  threadId: string,
  status: ThreadStatus,
  updatedAt: string,
): Promise<TaskThread | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<TaskThread>("thread_status_update", {
      request: { status, threadId, updatedAt },
    });
  } catch {
    return undefined;
  }
}

export async function archiveThreadOverBridge(
  threadId: string,
  updatedAt: string,
): Promise<TaskThread | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<TaskThread>("thread_archive", {
      request: { threadId, updatedAt },
    });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
