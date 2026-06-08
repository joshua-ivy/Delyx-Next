import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import { selectedOllamaModel, sendOllamaChat } from "../features/models/ollamaClient";
import {
  createOllamaPatchDraftMessages,
  createPatchProposalRequestFromOllamaText,
} from "../features/patches/ollamaPatchDraft";
import { loadPatchSnapshot } from "../features/patches/patchClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { executePatchProposalNodeOverBridge } from "../features/runs/agentExecutorClient";
import { appendThreadMessageOverBridge, loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import { loadWorkspaceFiles } from "./workspaceBridge";
import { recordModelCallFailure, recordModelCallResult, recordModelCallStarted } from "./appShellModelRunActions";
import { updateRunsForThreadStatus } from "./appShellRunActions";
import { modeForThreadStatus } from "./appShellThreadActions";
import { notifyLocalAction } from "./ShellPreferenceController";
import type { SchedulerDispatchState } from "./appShellSchedulerDispatch";

export interface OllamaPatchProposalState extends SchedulerDispatchState {
  modelSettings: ModelSettingsView;
}

export async function proposeApprovedPlanPatchWithOllama(
  state: OllamaPatchProposalState,
  approval: ActionProposalView,
) {
  if (!shouldDraftPatchAfterPlanApproval(state, approval)) {
    return false;
  }
  const thread = state.activeThread!;
  const run = state.activeRun!;
  const model = selectedOllamaModel(state.modelSettings);
  if (!model) {
    recordPatchDraftFailure(state, thread, "ollama-local", "Ollama is not ready to draft a patch.");
    return false;
  }

  try {
    const paths = approvedPlanFiles(state, approval);
    const readFiles = await loadWorkspaceFiles(state.activeProject, paths);
    if (!readFiles || readFiles.length === 0) {
      throw new Error("Desktop bridge could not read the approved plan files.");
    }
    startPatchDraft(state, thread, model);
    const response = await sendOllamaChat(
      state.modelSettings,
      createOllamaPatchDraftMessages(thread, state.activePlan!, state.activeProject, readFiles),
    );
    const request = createPatchProposalRequestFromOllamaText({
      approvalId: approval.id,
      clientId: `patch-${run.id}-${approval.id}`,
      plan: state.activePlan!,
      project: state.activeProject,
      readFiles,
      runId: run.id,
      text: response.text,
    });
    const result = await executePatchProposalNodeOverBridge({ ...request, createdAtMs: Date.now() });
    if (!result || result.status !== "completed") {
      throw new Error(result?.message ?? "Desktop bridge did not capture the patch proposal.");
    }
    await reloadPatchDraftState(state, run.id);
    completePatchDraft(state, thread, response.providerId, response.model, response.text, result.message);
    return true;
  } catch (error) {
    recordPatchDraftFailure(state, thread, model, patchDraftErrorMessage(error));
    return false;
  }
}

function shouldDraftPatchAfterPlanApproval(
  state: OllamaPatchProposalState,
  approval: ActionProposalView,
) {
  return Boolean(
    approval.status === "approved"
      && (approval.actionType === "edit_file" || approval.actionType === "write_file")
      && state.activePlan?.decision === "approved"
      && state.activeRun
      && state.activeThread
      && state.patches.every((patch) => patch.runId !== state.activeRun?.id)
      && approvedPlanFiles(state, approval).length > 0,
  );
}

function approvedPlanFiles(state: OllamaPatchProposalState, approval: ActionProposalView) {
  const indexed = new Set(state.activeProject.indexedFiles.map(normalizePath));
  const scoped = new Set((approval.scope.paths ?? []).map(normalizePath));
  return (state.activePlan?.filesLikelyInvolved ?? [])
    .filter((path) => indexed.has(normalizePath(path)))
    .filter((path) => scoped.size === 0 || scoped.has(normalizePath(path)))
    .slice(0, 4);
}

function startPatchDraft(state: OllamaPatchProposalState, thread: TaskThread, model: string) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "system", body: `Ollama PatchDraftAgent is drafting with ${model}.` }, "building");
  state.setThreadState("ready");
  state.setAgentRuns((current) => recordModelCallStarted(
    updateRunsForThreadStatus(current, thread, "building", now),
    thread,
    model,
    now,
  ));
}

function completePatchDraft(
  state: OllamaPatchProposalState,
  thread: TaskThread,
  providerId: string,
  model: string,
  responseText: string,
  message: string,
) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "assistant", body: message }, "building");
  state.setThreadState("ready");
  state.setAgentRuns((current) => recordModelCallResult(current, thread, providerId, model, responseText, now, "running"));
  notifyLocalAction(message, "success");
}

function recordPatchDraftFailure(
  state: OllamaPatchProposalState,
  thread: TaskThread,
  model: string,
  message: string,
) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "system", body: message }, "blocked");
  state.setThreadState("ready");
  state.setAgentRuns((current) => recordModelCallFailure(current, thread, model, message, now, "blocked"));
  notifyLocalAction(message, "warning");
}

async function reloadPatchDraftState(state: OllamaPatchProposalState, runId: string) {
  const [patches, snapshot] = await Promise.all([
    loadPatchSnapshot(runId),
    loadThreadRunSnapshot(state.activeProject.id),
  ]);
  if (patches) {
    state.setPatches(patches);
  }
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
}

function appendMessage(
  state: OllamaPatchProposalState,
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

function patchDraftErrorMessage(error: unknown) {
  const detail = error instanceof Error ? error.message : "Ollama patch draft request failed.";
  return `Ollama patch draft was not usable: ${detail}`;
}

function normalizePath(path: string) {
  return path.replace(/\\/g, "/").replace(/^\.\//, "").toLowerCase();
}
