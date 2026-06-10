import { sendCliChat } from "./cliChatClient";
import { cliAdapterForSelection } from "./cliModels";
import { selectedCodingRoute, sendModelChat, sendModelChatTools } from "../features/models/modelClient";
import type { TaskThread, ThreadStatus } from "../features/threads/threadTypes";
import { recordModelCallFailure, recordModelCallResult, recordModelCallStarted } from "./appShellModelRunActions";
import { createRunForThread, threadWithRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { createThread, modeForThreadStatus } from "./appShellThreadActions";
import { notifyLocalAction } from "./ShellPreferenceController";
import {
  appendThreadMessageOverBridge,
  createThreadRunOverBridge,
  updateThreadStatusOverBridge,
} from "../features/threads/threadClient";
import {
  appendMessage,
  beginLocalDraft,
  cliPrompt,
  errorText,
  finalizeLocalDraft,
  lastUserMessage,
  modelMessages,
  providerLabel,
  updateLocalDraft,
  withUserMessage,
  type ComposerBindingState,
} from "./cockpitComposerThreadOps";
import { runQaqcFollowup } from "./cockpitComposerQaqc";

export { appendMessage };
export type { ComposerBindingState };

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

function recordModelFailure(state: ComposerBindingState, thread: TaskThread, model: string, message: string) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "system", body: message }, "blocked");
  state.setAgentRuns((current) => recordModelCallFailure(current, thread, model, message, now));
  notifyLocalAction(message, "warning");
}

export function markThread(state: ComposerBindingState, threadId: string, status: ThreadStatus) {
  const now = new Date().toISOString();
  void updateThreadStatusOverBridge(threadId, status, now);
  state.setThreads((current) => current.map((thread) => (
    thread.id === threadId ? { ...thread, mode: modeForThreadStatus(status), status, updatedAt: now } : thread
  )));
}
