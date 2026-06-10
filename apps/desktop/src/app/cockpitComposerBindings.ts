import type { Dispatch, SetStateAction } from "react";
import { sendCliChat } from "./cliChatClient";
import { extractReviewableCode, qaqcVerdictMessage, sendCliReview } from "./cliReviewClient";
import { cliAdapterForSelection } from "./cliModels";
import { selectedCodingRoute, sendModelChat, sendModelChatTools } from "../features/models/modelClient";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
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

// How many times to re-review (and re-fix) the corrected code before showing it.
// Bounded so a failing reply costs at most 1 + this many cheap CLI calls.
const QAQC_VERIFY_ROUNDS = 2;

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

export function sendComposerInstruction(state: ComposerBindingState, text: string, newThread = false) {
  const body = text.trim();
  if (!body) {
    notifyLocalAction("Type a local instruction before sending", "warning");
    return;
  }
  // Sending from the home / new-chat view starts a fresh thread; sending from
  // inside a thread continues it. (activeThread is never undefined once any
  // thread exists, so we can't infer intent from it alone.)
  if (newThread || !state.activeThread) {
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
  // Strong-worker mode: the instruction becomes an approval-gated agentic CLI
  // run instead of a chat reply. Queueing creates the visible approval cards.
  if (state.workerAdapterId) {
    const task = lastUserMessage(thread);
    if (task) {
      void import("./appShellWorkerActions").then(({ queueWorkerRun }) =>
        queueWorkerRun(state, thread, state.workerAdapterId!, task, state.workerMode ?? "read_only"),
      );
      return;
    }
  }
  const cliAdapter = cliAdapterForSelection(state.modelSettings);
  if (cliAdapter) {
    void requestCliReply(state, thread, cliAdapter);
    return;
  }
  void requestModelReplyInner(state, thread);
}

function lastUserMessage(thread: TaskThread): string | undefined {
  return [...thread.messages].reverse().find((message) => message.role === "user")?.body;
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
    // Project context (repo map + rules files) gives the local model real
    // codebase awareness; cached per project so the cost is one read.
    const projectContext = await import("./projectContext")
      .then(({ projectContextBlock }) => projectContextBlock(state.activeProject))
      .catch(() => "");
    // Delyx Local runs the agentic tool loop: read-only tool calls narrate into
    // the draft bubble, then the final answer streams over them. Other providers
    // resolve in one shot (the client falls back internally).
    let draftStarted = false;
    const toolSummaries: string[] = [];
    const ensureDraft = () => {
      if (!draftStarted) {
        draftStarted = true;
        beginLocalDraft(state, thread.id);
      }
    };
    const response = await sendModelChatTools(
      state.modelSettings,
      modelMessages(thread, projectContext),
      state.activeProject.path,
      {
        onToken: (accumulated) => {
          ensureDraft();
          updateLocalDraft(state, thread.id, accumulated);
        },
        onTool: (summary) => {
          toolSummaries.push(summary);
          ensureDraft();
          updateLocalDraft(state, thread.id, toolSummaries.map((item) => `🔧 ${item}…`).join("\n"));
        },
      },
    );
    const now = new Date().toISOString();
    state.setAgentRuns((current) => recordModelCallResult(current, thread, response.providerId, response.model, response.text, now));
    if (draftStarted && response.text) {
      // The draft already shows the text locally; persist the final once.
      updateLocalDraft(state, thread.id, response.text);
      finalizeLocalDraft(state, thread.id, response.text);
    } else if (response.text) {
      appendMessage(state, thread.id, { role: "assistant", body: response.text }, "idle");
    }
    if (toolSummaries.length > 0) {
      appendMessage(
        state,
        thread.id,
        { role: "system", body: `🔧 Tool loop: ${toolSummaries.length} read-only call(s) — ${toolSummaries.join("; ")}` },
        "idle",
      );
    }
    if (response.cancelled) {
      appendMessage(state, thread.id, { role: "system", body: "Generation stopped — the partial reply above was kept." }, "idle");
      notifyLocalAction("Generation stopped", "info");
    } else {
      notifyLocalAction(`${providerLabel(response.providerId)} replied with ${response.model}`, "success");
    }
    // QA/QC runs in the background and appends its verdict / corrected code as a
    // follow-up; skipped for cancelled partials (reviewing half an answer wastes
    // a paid call).
    if (state.qaqcAdapterId && !response.cancelled && response.text) {
      void runQaqcFollowup(state, thread, response.text);
    }
  } catch (error) {
    recordModelFailure(state, thread, route.modelId, errorText(error, "Model request failed."));
  }
}

/** Add a local-only (not yet persisted) empty assistant draft to stream into. */
function beginLocalDraft(state: ComposerBindingState, threadId: string) {
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, messages: [...thread.messages, { role: "assistant" as const, body: "" }] }
      : thread
  )));
}

/** Replace the trailing assistant draft's text locally (no bridge write). */
function updateLocalDraft(state: ComposerBindingState, threadId: string, body: string) {
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
function finalizeLocalDraft(state: ComposerBindingState, threadId: string, body: string) {
  const now = new Date().toISOString();
  void appendThreadMessageOverBridge(threadId, { role: "assistant", body }, now, "idle");
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, mode: modeForThreadStatus("idle"), status: "idle", updatedAt: now }
      : thread
  )));
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

function reviewerLabel(adapterId: string): string {
  if (adapterId === "claude-code") {
    return "Claude Code";
  }
  if (adapterId === "codex-cli") {
    return "Codex";
  }
  return adapterId;
}

/** Drop the leading "VERDICT: …" line so findings can be reframed without it. */
function reviewFindings(text: string): string {
  return text.replace(/^\s*VERDICT:\s*(PASS|FAIL|UNCLEAR)\b.*\n?/i, "").trim();
}

/**
 * Background QA/QC of a model reply. The answer is already shown; this only
 * appends a follow-up — a PASS note, or the corrected code when a fix is found —
 * so it never blocks or hides the user's answer. No reviewer / no code → no-op.
 */
async function runQaqcFollowup(state: ComposerBindingState, thread: TaskThread, replyText: string): Promise<void> {
  const adapter = state.qaqcAdapterId;
  if (!adapter) {
    return;
  }
  // Only spend a paid CLI review when there is code to review, and send just the
  // code blocks rather than the model's prose — both cut tokens on every reply.
  const code = extractReviewableCode(replyText);
  if (!code) {
    return;
  }
  const label = reviewerLabel(adapter);
  notifyLocalAction(`QA/QC reviewing via ${label}…`, "info");
  try {
    const result = await sendCliReview(adapter, thread.goal, code, state.activeProject.path, state.qaqcModel);
    const fix = result.fix?.trim();
    if (result.verdict === "fail" && fix) {
      const originalIssues = reviewFindings(result.text);
      // Re-check the corrected code (bounded) so we only claim "verified" once a
      // fresh review of the corrected code actually passes.
      let bestFix = fix;
      let verified = false;
      let remainingFindings = "";
      for (let round = 0; round < QAQC_VERIFY_ROUNDS; round += 1) {
        const recheck = await sendCliReview(adapter, thread.goal, bestFix, state.activeProject.path, state.qaqcModel);
        if (recheck.verdict === "pass") {
          verified = true;
          break;
        }
        remainingFindings = reviewFindings(recheck.text);
        const nextFix = recheck.fix?.trim();
        if (!nextFix) {
          break;
        }
        bestFix = nextFix;
      }
      const verdictTag = verified ? "verified" : "fixed";
      appendMessage(state, thread.id, { role: "assistant", body: `[[qaqc:${verdictTag}:${label}]]\n\n\`\`\`\n${bestFix}\n\`\`\`` }, "idle");
      const note = verified
        ? [`✓ QA/QC (${label}): FIXED & VERIFIED — re-checked and now passes.`, originalIssues && `\nIssues found in the original and corrected:\n\n${originalIssues}`].filter(Boolean).join("\n")
        : `✦ QA/QC (${label}): FIXED (not fully verified)\n\nCorrected the main issues, but a re-check still flagged:\n\n${remainingFindings || originalIssues}`;
      appendMessage(state, thread.id, { role: "system", body: note }, "idle");
      notifyLocalAction(verified ? `QA/QC fixed & verified via ${adapter}` : `QA/QC fixed (unverified) via ${adapter}`, verified ? "success" : "warning");
      return;
    }
    if (result.verdict === "pass") {
      appendMessage(state, thread.id, { role: "system", body: `✓ QA/QC (${label}): PASS` }, "idle");
      notifyLocalAction(`QA/QC pass via ${adapter}`, "success");
      return;
    }
    // Failed with no usable fix (or unclear): surface why (the answer stays put).
    appendMessage(state, thread.id, { role: "system", body: qaqcVerdictMessage(adapter, result) }, "idle");
    notifyLocalAction(`QA/QC ${result.verdict} via ${adapter}`, "warning");
  } catch (error) {
    appendMessage(state, thread.id, { role: "system", body: `QA/QC review via ${adapter} failed: ${error instanceof Error ? error.message : "review failed"}` }, "idle");
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

export function appendMessage(state: ComposerBindingState, threadId: string, message: TaskThread["messages"][number], status: ThreadStatus) {
  const now = new Date().toISOString();
  void appendThreadMessageOverBridge(threadId, message, now, status);
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId
      ? { ...thread, messages: [...thread.messages, message], mode: modeForThreadStatus(status), status, updatedAt: now }
      : thread
  )));
}

export function markThread(state: ComposerBindingState, threadId: string, status: ThreadStatus) {
  const now = new Date().toISOString();
  void updateThreadStatusOverBridge(threadId, status, now);
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId ? { ...thread, mode: modeForThreadStatus(status), status, updatedAt: now } : thread
  )));
}

function modelMessages(thread: TaskThread, projectContext = ""): ThreadRoleMessage[] {
  const base = "You are Delyx Next, a local-first AI workbench assistant. Be direct, honest, and do not claim tool execution unless an artifact exists.";
  const system = projectContext ? `${base}\n\n${projectContext}` : base;
  return [
    { role: "system", content: system },
    ...thread.messages.map((message) => ({ role: message.role, content: message.body })),
  ];
}
