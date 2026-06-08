import type { Dispatch, SetStateAction } from "react";

import { sendOllamaChat, selectedOllamaModel } from "../features/models/ollamaClient";
import type { ModelSettingsView } from "../features/models/modelTypes";
import { createOllamaPlanMessages, createPlanFromOllamaText } from "../features/plans/ollamaPlan";
import { savePlanOverBridge } from "../features/plans/planClient";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { appendThreadMessageOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { recordModelCallFailure, recordModelCallResult, recordModelCallStarted } from "./appShellModelRunActions";
import { createRunForThread, threadWithRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { modeForThreadStatus, upsertPlan } from "./appShellThreadActions";
import { notifyLocalAction } from "./ShellPreferenceController";

export interface OllamaPlanState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  modelSettings: ModelSettingsView;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setPlans: Dispatch<SetStateAction<PlanView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  threads: TaskThread[];
}

export async function createPlanWithOllama(state: OllamaPlanState) {
  const thread = state.activeThread;
  if (!thread) {
    state.setThreadState("empty");
    notifyLocalAction("Create a thread before planning", "warning");
    return;
  }

  const runnableThread = ensureRun(state, thread);
  const model = selectedOllamaModel(state.modelSettings);
  if (!model) {
    recordOllamaPlanFailure(state, runnableThread, "ollama-local", "Ollama is not ready. Start Ollama and pull a model before planning.");
    return;
  }

  startOllamaPlan(state, runnableThread, model);
  try {
    const response = await sendOllamaChat(state.modelSettings, createOllamaPlanMessages(runnableThread, state.activeProject));
    const plan = createPlanFromOllamaText(runnableThread, state.activeProject, response.text);
    const savedPlan = await savePlanOverBridge(state.activeProject.id, plan) ?? plan;
    const now = new Date().toISOString();
    state.setPlans((current) => upsertPlan(current, savedPlan));
    appendMessage(state, runnableThread.id, {
      role: "assistant",
      body: `Ollama drafted a read-only plan with ${savedPlan.steps.length} step(s). Review it before any approval.`,
    }, "planning");
    state.setAgentRuns((current) => recordModelCallResult(current, runnableThread, response.providerId, response.model, response.text, now, "running"));
    notifyLocalAction(`Ollama plan drafted with ${response.model}`, "success");
  } catch (error) {
    recordOllamaPlanFailure(state, runnableThread, model, planErrorMessage(error));
  }
}

function ensureRun(state: OllamaPlanState, thread: TaskThread) {
  if (thread.activeRunId) {
    return thread;
  }
  const run = createRunForThread(thread, state.activeProject.id, state.threads.length + 1);
  const runnableThread = threadWithRun(thread, run);
  state.setAgentRuns((current) => [run, ...current]);
  state.setThreads((current) => current.map((item) => (item.id === thread.id ? runnableThread : item)));
  return runnableThread;
}

function startOllamaPlan(state: OllamaPlanState, thread: TaskThread, model: string) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "system", body: `Ollama PlanAgent is drafting with ${model}.` }, "planning");
  state.setThreadState("ready");
  state.setAgentRuns((current) => recordModelCallStarted(updateRunsForThreadStatus(current, thread, "planning", now), thread, model, now));
}

function recordOllamaPlanFailure(state: OllamaPlanState, thread: TaskThread, model: string, message: string) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "system", body: message }, "blocked");
  state.setThreadState("ready");
  state.setAgentRuns((current) => recordModelCallFailure(current, thread, model, message, now, "blocked"));
  notifyLocalAction(message, "warning");
}

function appendMessage(
  state: OllamaPlanState,
  threadId: string,
  message: TaskThread["messages"][number],
  status: ThreadStatus,
) {
  const now = new Date().toISOString();
  void appendThreadMessageOverBridge(threadId, message, now, status);
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, messages: [...thread.messages, message], mode: modeForThreadStatus(status), status, updatedAt: now }
      : thread
  )));
}

function planErrorMessage(error: unknown) {
  const detail = error instanceof Error ? error.message : "Ollama plan request failed.";
  return `Ollama plan was not usable: ${detail}`;
}
