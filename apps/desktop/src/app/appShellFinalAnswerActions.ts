import type { Dispatch, SetStateAction } from "react";

import type { AgentRunView } from "../features/runs/agentRunTypes";
import { finalizeThreadRunOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import { notifyLocalAction } from "./ShellPreferenceController";

interface FinalAnswerState {
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function recordFinalSupportForActiveThread(state: FinalAnswerState) {
  if (!state.activeRun || !state.activeThread) {
    notifyLocalAction("Create a thread with a run before recording final support", "warning");
    return;
  }
  if (state.activeRun.outcome) {
    notifyLocalAction("Final support is already recorded for this run", "warning");
    return;
  }
  const summary = latestAssistantSummary(state.activeThread);
  if (!summary) {
    notifyLocalAction("Final support needs an existing assistant answer; no prose is generated here", "warning");
    return;
  }
  const record = await finalizeThreadRunOverBridge(state.activeThread.id, summary, new Date().toISOString());
  if (!record) {
    notifyLocalAction("Desktop bridge is required to record final support", "warning");
    return;
  }
  state.setThreads((current) => current.map((thread) => (
    thread.id === record.thread.id ? record.thread : thread
  )));
  state.setAgentRuns((current) => current.map((run) => (
    run.id === record.run.id ? record.run : run
  )));
  state.setThreadState("ready");
  notifyLocalAction("Final support linked existing evidence and passed tests", "success");
}

function latestAssistantSummary(thread: TaskThread) {
  return [...thread.messages].reverse().find((message) => (
    message.role === "assistant" && message.body.trim()
  ))?.body.trim();
}
