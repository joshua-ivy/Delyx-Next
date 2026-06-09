import type { Dispatch, SetStateAction } from "react";
import { sendCliChat } from "./cliChatClient";
import { qaqcVerdictMessage, sendCliReview } from "./cliReviewClient";
import { cliAdapterForSelection } from "./cliModels";
import { selectedCodingRoute, sendModelChat } from "../features/models/modelClient";
import type { ModelSettingsView, ThreadRoleMessage } from "../features/models/modelTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { recordModelCallFailure, recordModelCallResult, recordModelCallStarted } from "./appShellModelRunActions";
import { createRunForThread, threadWithRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { createThread, modeForThreadStatus } from "./appShellThreadActions";
import { notifyLocalAction } from "./ShellPreferenceController";
import {
  appendThreadMessageOverBridge,
  createThreadRunOverBridge,
  updateThreadStatusOverBridge,
} from "../features/threads/threadClient";

export interface ComposerBindingState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  modelSettings: ModelSettingsView;
  qaqcAdapterId?: string;
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
      void createThreadFromComposer(state, text);
      return;
    }
    continueThreadFromComposer(state, text);
  };
  form.addEventListener("submit", submit);
  return () => form.removeEventListener("submit", submit);
}

export function sendComposerInstruction(state: ComposerBindingState, text: string) {
  const body = text.trim();
  if (!body) {
    notifyLocalAction("Type a local instruction before sending", "warning");
    return;
  }
  if (!state.activeThread) {
    void createThreadFromComposer(state, body);
    return;
  }
  continueThreadFromComposer(state, body);
}

async function createThreadFromComposer(state: ComposerBindingState, goal: string) {
  const createdAt = new Date().toISOString();
  const record = await createThreadRunOverBridge(state.activeProject.id, goal, createdAt);
  const thread = record?.thread ?? createThread(goal, state.activeProject.id, state.threads.length + 1);
  const run = record?.run ?? (thread ? createRunForThread(thread, state.activeProject.id, state.threads.length + 1) : undefined);
  const runnableThread = thread && run ? (record?.thread ?? threadWithRun(thread, run)) : undefined;
  if (!runnableThread || !run) {
    state.setThreadState("error");
    notifyLocalAction("Thread goal was empty", "warning");
    return;
  }
  state.setAgentRuns((current) => [run, ...current]);
  state.setThreads((current) => [runnableThread, ...current]);
  state.setActiveThreadId(runnableThread.id);
  state.setThreadState("ready");
  void requestModelReply(state, runnableThread);
}

function continueThreadFromComposer(state: ComposerBindingState, body: string) {
  const thread = state.activeThread;
  if (!thread) {
    return;
  }
  const now = new Date().toISOString();
  const updatedThread = withUserMessage(ensureThreadRun(state, thread), body, now);
  void appendThreadMessageOverBridge(updatedThread.id, { role: "user", body }, now);
  state.setThreads((current) => current.map((item) => (item.id === updatedThread.id ? updatedThread : item)));
  void requestModelReply(state, updatedThread);
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

function requestModelReply(state: ComposerBindingState, thread: TaskThread) {
  const cliAdapter = cliAdapterForSelection(state.modelSettings);
  if (cliAdapter) {
    void requestCliReply(state, thread, cliAdapter);
    return;
  }
  void requestModelReplyInner(state, thread);
}

async function requestCliReply(state: ComposerBindingState, thread: TaskThread, adapterId: string) {
  markThread(state, thread.id, "exploring");
  state.setAgentRuns((current) => updateRunsForThreadStatus(current, thread, "exploring", new Date().toISOString()));
  try {
    state.setAgentRuns((current) => recordModelCallStarted(current, thread, adapterId, new Date().toISOString()));
    const result = await sendCliChat(adapterId, cliPrompt(thread), state.activeProject.path);
    const now = new Date().toISOString();
    appendMessage(state, thread.id, { role: "assistant", body: result.text }, "idle");
    state.setAgentRuns((current) => recordModelCallResult(current, thread, adapterId, adapterId, result.text, now));
    notifyLocalAction(`${adapterId} replied`, "success");
  } catch (error) {
    recordModelFailure(state, thread, adapterId, errorText(error, "CLI request failed."));
  }
}

function cliPrompt(thread: TaskThread): string {
  return thread.messages
    .map((message) => `${message.role === "user" ? "User" : message.role === "assistant" ? "Assistant" : "System"}: ${message.body}`)
    .join("\n\n");
}

async function requestModelReplyInner(state: ComposerBindingState, thread: TaskThread) {
  const route = selectedCodingRoute(state.modelSettings);
  markThread(state, thread.id, "exploring");
  state.setAgentRuns((current) => updateRunsForThreadStatus(current, thread, "exploring", new Date().toISOString()));
  if (!route) {
    recordModelFailure(state, thread, "no-model", "No ready model is selected. Import a Delyx Local model or select Ollama/CLI.");
    return;
  }
  try {
    state.setAgentRuns((current) => recordModelCallStarted(current, thread, route.modelId, new Date().toISOString()));
    const response = await sendModelChat(state.modelSettings, modelMessages(thread));
    const now = new Date().toISOString();
    appendMessage(state, thread.id, { role: "assistant", body: response.text }, "idle");
    state.setAgentRuns((current) => recordModelCallResult(current, thread, response.providerId, response.model, response.text, now));
    notifyLocalAction(`${providerLabel(response.providerId)} replied with ${response.model}`, "success");
    if (state.qaqcAdapterId) {
      void runQaqcReview(state, thread, response.text);
    }
  } catch (error) {
    recordModelFailure(state, thread, route.modelId, errorText(error, "Model request failed."));
  }
}

function errorText(error: unknown, fallback: string): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  return fallback;
}

function providerLabel(providerId: string): string {
  if (providerId === "delyx-local") {
    return "Delyx Local";
  }
  if (providerId === "ollama-local") {
    return "Ollama";
  }
  return providerId;
}

async function runQaqcReview(state: ComposerBindingState, thread: TaskThread, content: string) {
  const adapter = state.qaqcAdapterId;
  if (!adapter) {
    return;
  }
  try {
    const result = await sendCliReview(adapter, thread.goal, content, state.activeProject.path);
    appendMessage(
      state,
      thread.id,
      { role: "system", body: qaqcVerdictMessage(adapter, result) },
      result.verdict === "fail" ? "blocked" : "idle",
    );
    notifyLocalAction(`QA/QC ${result.verdict} via ${adapter}`, result.verdict === "fail" ? "warning" : "success");
  } catch (error) {
    appendMessage(
      state,
      thread.id,
      { role: "system", body: `QA/QC review via ${adapter} failed: ${error instanceof Error ? error.message : "review failed"}` },
      "idle",
    );
  }
}

function recordModelFailure(state: ComposerBindingState, thread: TaskThread, model: string, message: string) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "system", body: message }, "blocked");
  state.setAgentRuns((current) => recordModelCallFailure(current, thread, model, message, now));
  notifyLocalAction(message, "warning");
}

function withUserMessage(thread: TaskThread, body: string, updatedAt: string): TaskThread {
  return { ...thread, messages: [...thread.messages, { role: "user", body }], updatedAt };
}

function appendMessage(state: ComposerBindingState, threadId: string, message: TaskThread["messages"][number], status: ThreadStatus) {
  const now = new Date().toISOString();
  void appendThreadMessageOverBridge(threadId, message, now, status);
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, messages: [...thread.messages, message], mode: modeForThreadStatus(status), status, updatedAt: now }
      : thread
  )));
}

function markThread(state: ComposerBindingState, threadId: string, status: ThreadStatus) {
  const now = new Date().toISOString();
  void updateThreadStatusOverBridge(threadId, status, now);
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
