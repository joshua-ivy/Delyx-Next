import type { Dispatch, SetStateAction } from "react";
import { sendOllamaChat, selectedOllamaModel } from "../features/models/ollamaClient";
import type { ModelSettingsView, ThreadRoleMessage } from "../features/models/modelTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { recordModelCallFailure, recordModelCallResult } from "./appShellModelRunActions";
import { createRunForThread, threadWithRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { createThread, modeForThreadStatus } from "./appShellThreadActions";
import { notifyLocalAction } from "./ShellPreferenceController";

export interface ComposerBindingState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  modelSettings: ModelSettingsView;
  setActiveThreadId: Dispatch<SetStateAction<string | undefined>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  threads: TaskThread[];
}

export function bindComposerForm(state: ComposerBindingState, form: Element | null) {
  if (!(form instanceof HTMLFormElement)) {
    return () => undefined;
  }
  const input = form.querySelector(".deck-comp-input");
  if (!(input instanceof HTMLTextAreaElement)) {
    return () => undefined;
  }
  const submit = (event: Event) => {
    event.preventDefault();
    const text = input.value.trim();
    if (!text) {
      notifyLocalAction("Type a local instruction before sending", "warning");
      return;
    }
    input.value = "";
    if (!state.activeThread) {
      createThreadFromComposer(state, text);
      return;
    }
    continueThreadFromComposer(state, text);
  };
  form.addEventListener("submit", submit);
  return () => form.removeEventListener("submit", submit);
}

function createThreadFromComposer(state: ComposerBindingState, goal: string) {
  const thread = createThread(goal, state.activeProject.id, state.threads.length + 1);
  if (!thread) {
    state.setThreadState("error");
    notifyLocalAction("Thread goal was empty", "warning");
    return;
  }
  const run = createRunForThread(thread, state.activeProject.id, state.threads.length + 1);
  const runnableThread = threadWithRun(thread, run);
  state.setAgentRuns((current) => [run, ...current]);
  state.setThreads((current) => [runnableThread, ...current]);
  state.setActiveThreadId(runnableThread.id);
  state.setThreadState("ready");
  void requestOllamaReply(state, runnableThread);
}

function continueThreadFromComposer(state: ComposerBindingState, body: string) {
  const thread = state.activeThread;
  if (!thread) {
    return;
  }
  const updatedThread = withUserMessage(ensureThreadRun(state, thread), body);
  state.setThreads((current) => current.map((item) => (item.id === updatedThread.id ? updatedThread : item)));
  void requestOllamaReply(state, updatedThread);
}

function ensureThreadRun(state: ComposerBindingState, thread: TaskThread) {
  if (thread.activeRunId) {
    return thread;
  }
  const run = createRunForThread(thread, state.activeProject.id, state.threads.length + 1);
  const runnableThread = threadWithRun(thread, run);
  state.setAgentRuns((current) => [run, ...current]);
  return runnableThread;
}

async function requestOllamaReply(state: ComposerBindingState, thread: TaskThread) {
  const model = selectedOllamaModel(state.modelSettings);
  markThread(state, thread.id, "exploring");
  state.setAgentRuns((current) => updateRunsForThreadStatus(current, thread, "exploring", new Date().toISOString()));
  if (!model) {
    recordOllamaFailure(state, thread, "ollama-local", "Ollama is not ready. Start Ollama and pull a model, then send again.");
    return;
  }
  try {
    const response = await sendOllamaChat(state.modelSettings, modelMessages(thread));
    const now = new Date().toISOString();
    appendMessage(state, thread.id, { role: "assistant", body: response.text }, "idle");
    state.setAgentRuns((current) => recordModelCallResult(current, thread, response.providerId, response.model, response.text, now));
    notifyLocalAction(`Ollama replied with ${response.model}`, "success");
  } catch (error) {
    recordOllamaFailure(state, thread, model, error instanceof Error ? error.message : "Ollama request failed.");
  }
}

function recordOllamaFailure(state: ComposerBindingState, thread: TaskThread, model: string, message: string) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "system", body: message }, "blocked");
  state.setAgentRuns((current) => recordModelCallFailure(current, thread, model, message, now));
  notifyLocalAction(message, "warning");
}

function withUserMessage(thread: TaskThread, body: string): TaskThread {
  return { ...thread, messages: [...thread.messages, { role: "user", body }], updatedAt: new Date().toISOString() };
}

function appendMessage(state: ComposerBindingState, threadId: string, message: TaskThread["messages"][number], status: ThreadStatus) {
  const now = new Date().toISOString();
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, messages: [...thread.messages, message], mode: modeForThreadStatus(status), status, updatedAt: now }
      : thread
  )));
}

function markThread(state: ComposerBindingState, threadId: string, status: ThreadStatus) {
  const now = new Date().toISOString();
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId ? { ...thread, mode: modeForThreadStatus(status), status, updatedAt: now } : thread
  )));
}

function modelMessages(thread: TaskThread): ThreadRoleMessage[] {
  return [
    { role: "system", content: "You are Delyx Next, a local-first AI workbench assistant. Be direct, honest, and do not claim tool execution unless an artifact exists." },
    ...thread.messages.map((message) => ({ role: message.role, content: message.body })),
  ];
}
