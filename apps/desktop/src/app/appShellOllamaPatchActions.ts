import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import { selectedOllamaModel } from "../features/models/ollamaClient";
import { loadPatchSnapshot } from "../features/patches/patchClient";
import { executePatchDraftNodeOverBridge, scheduleNextRunActionOverBridge } from "../features/runs/agentExecutorClient";
import { appendThreadMessageOverBridge, loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { ThreadRunSnapshotView } from "../features/threads/threadClient";
import type { TaskThread, ThreadStatus } from "../features/threads/threadTypes";
import { recordModelCallFailure } from "./appShellModelRunActions";
import { updateRunsForThreadStatus } from "./appShellRunActions";
import { modeForThreadStatus } from "./appShellThreadActions";
import { dispatchSchedulerDecision } from "./appShellSchedulerDispatch";
import { notifyLocalAction } from "./ShellPreferenceController";
import { firstRunnableTestCommand } from "./testCommand";
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
    startPatchDraft(state, thread, model);
    const result = await executePatchDraftNodeOverBridge({
      approvalId: approval.id,
      approvedRoots: state.activeProject.approvedRoots,
      clientId: `patch-${run.id}-${approval.id}`,
      createdAtMs: Date.now(),
      filesLikelyInvolved: paths,
      goal: thread.goal,
      maxBytesPerFile: 20_000,
      model,
      planSteps: state.activePlan!.steps,
      projectPath: state.activeProject.path,
      runId: run.id,
      scopePaths: approval.scope.paths ?? [],
    });
    if (!result || result.status !== "completed") {
      throw new Error(result?.message ?? "Desktop bridge did not capture the patch proposal.");
    }
    const reloaded = await reloadPatchDraftState(state, run.id);
    completePatchDraft(state, thread, result.message);
    await dispatchNextAfterPatchDraft(state, reloaded);
    return true;
  } catch (error) {
    await reloadPatchDraftState(state, run.id);
    showPatchDraftFailure(state, thread, patchDraftErrorMessage(error));
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

async function dispatchNextAfterPatchDraft(
  state: OllamaPatchProposalState,
  reloaded: PatchDraftReload,
) {
  if (!state.activeRun || !reloaded.patches) {
    return;
  }
  const decision = await scheduleNextRunActionOverBridge({
    hasSupportedTestCommand: Boolean(firstRunnableTestCommand(state.activePlan?.testsToRun)),
    nowMs: Date.now(),
    runId: state.activeRun.id,
  });
  if (!decision) {
    return;
  }
  await dispatchSchedulerDecision({
    ...state,
    activeRun: reloaded.snapshot?.runs.find((run) => run.id === state.activeRun?.id) ?? state.activeRun,
    patches: reloaded.patches,
  }, decision);
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

type PatchProposalSnapshot = Awaited<ReturnType<typeof loadPatchSnapshot>>;

interface PatchDraftReload {
  patches: PatchProposalSnapshot;
  snapshot: ThreadRunSnapshotView | undefined;
}
