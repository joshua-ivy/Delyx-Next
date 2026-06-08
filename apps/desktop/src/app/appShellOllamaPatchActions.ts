import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import { selectedOllamaModel } from "../features/models/ollamaClient";
import { loadPatchSnapshot } from "../features/patches/patchClient";
import type { PatchProposalView } from "../features/patches/patchTypes";
import { dispatchPatchDraftFromContextOverBridge } from "../features/runs/agentExecutorClient";
import { appendThreadMessageOverBridge, loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { ThreadRunSnapshotView } from "../features/threads/threadClient";
import type { TaskThread, ThreadStatus } from "../features/threads/threadTypes";
import { recordModelCallFailure } from "./appShellModelRunActions";
import {
  patchDraftApprovalId,
} from "./appShellPatchDraftDecision";
import { updateRunsForThreadStatus } from "./appShellRunActions";
import { modeForThreadStatus } from "./appShellThreadActions";
import { notifyLocalAction } from "./ShellPreferenceController";
import type { SchedulerDispatchState } from "./appShellSchedulerDispatch";
import { activeTestApprovalId } from "./appShellTestApprovalDecision";
import { firstRunnableTestCommand } from "./testCommand";

export interface OllamaPatchProposalState extends SchedulerDispatchState {
  modelSettings: ModelSettingsView;
}

export async function proposeApprovedPlanPatchWithOllama(
  state: OllamaPatchProposalState,
  approval: ActionProposalView,
): Promise<PatchDraftDispatchResult> {
  if (patchDraftApprovalId(state) !== approval.id) {
    return { created: false };
  }
  if (!state.activeThread || !state.activeRun) {
    return { created: false };
  }
  const thread = state.activeThread!;
  const run = state.activeRun!;
  const model = selectedOllamaModel(state.modelSettings);
  if (!model) {
    recordPatchDraftFailure(state, thread, "ollama-local", "Ollama is not ready to draft a patch.");
    return { created: false };
  }

  try {
    startPatchDraft(state, thread, model);
    const createdAtMs = Date.now();
    const result = await dispatchPatchDraftFromContextOverBridge({
      approvalId: approval.id,
      hasSupportedTestCommand: Boolean(firstRunnableTestCommand(state.activePlan?.testsToRun)),
      maxBytesPerFile: 20_000,
      model,
      nowMs: createdAtMs,
      projectId: state.activeProject.id,
      runId: run.id,
      testApprovalId: activeTestApprovalId(state),
    });
    if (!result || result.status !== "completed") {
      throw new Error(result?.message ?? "Desktop bridge did not capture the patch proposal.");
    }
    const reloaded = await reloadPatchDraftState(state, run.id);
    completePatchDraft(state, thread, result.message);
    return { created: true, ...reloaded };
  } catch (error) {
    const reloaded = await reloadPatchDraftState(state, run.id);
    showPatchDraftFailure(state, thread, patchDraftErrorMessage(error));
    return { created: false, ...reloaded };
  }
}

function startPatchDraft(state: OllamaPatchProposalState, thread: TaskThread, model: string) {
  const now = new Date().toISOString();
  appendMessage(state, thread.id, { role: "system", body: `Ollama PatchDraftAgent is drafting with ${model}.` }, "building");
  state.setThreadState("ready");
  state.setAgentRuns((current) => updateRunsForThreadStatus(current, thread, "building", now));
}

function completePatchDraft(
  state: OllamaPatchProposalState,
  thread: TaskThread,
  message: string,
) {
  appendMessage(state, thread.id, { role: "assistant", body: message }, "building");
  state.setThreadState("ready");
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

function showPatchDraftFailure(
  state: OllamaPatchProposalState,
  thread: TaskThread,
  message: string,
) {
  appendMessage(state, thread.id, { role: "system", body: message }, "blocked");
  state.setThreadState("ready");
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
  return { patches, snapshot };
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

type PatchProposalSnapshot = Awaited<ReturnType<typeof loadPatchSnapshot>>;

export interface PatchDraftDispatchResult {
  created: boolean;
  patches?: PatchProposalView[];
  snapshot?: ThreadRunSnapshotView;
}

interface PatchDraftReload {
  patches: PatchProposalSnapshot;
  snapshot: ThreadRunSnapshotView | undefined;
}
