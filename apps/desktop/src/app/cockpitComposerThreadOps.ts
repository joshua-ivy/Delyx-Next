import type { Dispatch, SetStateAction } from "react";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSettingsView, ThreadRoleMessage } from "../features/models/modelTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { appendThreadMessageOverBridge } from "../features/threads/threadClient";
import { modeForThreadStatus } from "./appShellThreadActions";

export interface ComposerBindingState {
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  modelSettings: ModelSettingsView;
  qaqcAdapterId?: string;
  qaqcModel?: string;
  /** When set, composer sends launch the approval-gated agentic CLI worker. */
  workerAdapterId?: string;
  /** Worker permission mode; write mode requires planned files in the task. */
  workerMode?: "read_only" | "workspace_write";
  setActionProposals?: Dispatch<SetStateAction<ActionProposalView[]>>;
  setActiveThreadId: Dispatch<SetStateAction<string | undefined>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
  threads: TaskThread[];
}

export function appendMessage(state: ComposerBindingState, threadId: string, message: TaskThread["messages"][number], status: ThreadStatus) {
  const now = new Date().toISOString();
  void appendThreadMessageOverBridge(threadId, message, now, status);
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, messages: [...thread.messages, message], mode: modeForThreadStatus(status), status, updatedAt: now }
      : thread
  )));
}

/** Add a local-only (not yet persisted) empty assistant draft to stream into. */
export function beginLocalDraft(state: ComposerBindingState, threadId: string) {
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, messages: [...thread.messages, { role: "assistant" as const, body: "" }] }
      : thread
  )));
}

/** Replace the trailing assistant draft's text locally (no bridge write). */
export function updateLocalDraft(state: ComposerBindingState, threadId: string, body: string) {
  state.setThreads((current) => current.map((thread) => {
    if (thread.id !== threadId) {
      return thread;
    }
    const messages = [...thread.messages];
    const last = messages.length - 1;
    if (last >= 0 && messages[last].role === "assistant") {
      messages[last] = { ...messages[last], body };
    }
    return { ...thread, messages };
  }));
}

/** Persist the finished draft exactly once and settle the thread to idle. */
export function finalizeLocalDraft(state: ComposerBindingState, threadId: string, body: string) {
  const now = new Date().toISOString();
  void appendThreadMessageOverBridge(threadId, { role: "assistant", body }, now, "idle");
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, mode: modeForThreadStatus("idle"), status: "idle", updatedAt: now }
      : thread
  )));
}

export function withUserMessage(thread: TaskThread, body: string, updatedAt: string): TaskThread {
  return { ...thread, messages: [...thread.messages, { role: "user", body }], updatedAt };
}

export function lastUserMessage(thread: TaskThread): string | undefined {
  return [...thread.messages].reverse().find((message) => message.role === "user")?.body;
}

export function cliPrompt(thread: TaskThread): string {
  return thread.messages
    .map((message) => `${message.role === "user" ? "User" : message.role === "assistant" ? "Assistant" : "System"}: ${message.body}`)
    .join("\n\n");
}

export function modelMessages(thread: TaskThread, projectContext = ""): ThreadRoleMessage[] {
  const base = "You are Delyx Next, a local-first AI workbench assistant. Be direct, honest, and do not claim tool execution unless an artifact exists.";
  const system = projectContext ? `${base}\n\n${projectContext}` : base;
  return [
    { role: "system", content: system },
    ...thread.messages.map((message) => ({ role: message.role, content: message.body })),
  ];
}

export function errorText(error: unknown, fallback: string): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  return fallback;
}

export function providerLabel(providerId: string): string {
  if (providerId === "delyx-local") {
    return "Delyx Local";
  }
  if (providerId === "ollama-local") {
    return "Ollama";
  }
  return providerId;
}
