import { loadPatchSnapshot } from "../features/patches/patchClient";
import { loadReviewSnapshot } from "../features/review/reviewClient";
import { driveRunOverBridge } from "../features/runs/agentDriveClient";
import type { AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import { loadTestSnapshot } from "../features/tests/testClient";
import { loadThreadRunSnapshot } from "../features/threads/threadClient";
import type { SchedulerDispatchState } from "./appShellSchedulerDispatch";
import { notifyLocalAction } from "./ShellPreferenceController";

export function canRunSchedulerDriver(decision: AgentScheduleDecisionView) {
  if (decision.kind === "run_tests") {
    return Boolean(decision.approvalIds[0]);
  }
  return decision.kind === "ready_for_final_support"
    || decision.kind === "run_patch_apply"
    || decision.kind === "run_review";
}

export async function runSchedulerDriver(state: SchedulerDispatchState) {
  if (!state.activeRun) {
    notifyLocalAction("Create a run before driving scheduler-selected work", "warning");
    return undefined;
  }
  const now = new Date();
  const outcome = await driveRunOverBridge({
    finalSummary: latestAssistantSummary(state),
    nowMs: now.getTime(),
    runId: state.activeRun.id,
    timeoutMs: 5 * 60 * 1000,
    updatedAt: now.toISOString(),
  });
  if (!outcome) {
    notifyLocalAction("Desktop bridge is required to drive scheduler-selected work", "warning");
    return undefined;
  }
  const [patches, tests, reviews, snapshot] = await Promise.all([
    loadPatchSnapshot(state.activeRun.id),
    loadTestSnapshot(state.activeRun.id),
    loadReviewSnapshot(state.activeRun.id),
    loadThreadRunSnapshot(state.activeProject.id),
  ]);
  if (patches) {
    state.setPatches(patches);
  }
  if (tests) {
    state.setTests(tests);
  }
  if (reviews) {
    state.setReviews((current) => [
      ...reviews,
      ...current.filter((report) => report.runId !== state.activeRun?.id),
    ]);
  }
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
  state.setThreadState("ready");
  notifyDriveStop(outcome.stoppedBecause.kind, outcome.stoppedBecause.message);
  return { outcome, patches, reviews, snapshot, tests };
}

export function driverReloadedState(
  state: SchedulerDispatchState,
  result: Awaited<ReturnType<typeof runSchedulerDriver>>,
) {
  return {
    ...state,
    activeRun: result?.snapshot?.runs.find((run) => run.id === state.activeRun?.id) ?? state.activeRun,
    patches: result?.patches ?? state.patches,
    reviews: result?.reviews ?? state.reviews,
    tests: result?.tests ?? state.tests,
  };
}

function latestAssistantSummary(state: SchedulerDispatchState) {
  return [...(state.activeThread?.messages ?? [])]
    .reverse()
    .find((message) => message.role === "assistant" && message.body.trim())
    ?.body.trim();
}

function notifyDriveStop(kind: string, message: string) {
  if (kind === "completed") {
    notifyLocalAction("Driver completed scheduler-selected work", "success");
  } else if (kind === "needs_final_summary") {
    notifyLocalAction("Final support needs an existing assistant answer; no prose is generated here", "warning");
  } else if (kind === "blocked" || kind.startsWith("needs_")) {
    notifyLocalAction(message, "warning");
  } else {
    notifyLocalAction(message, "success");
  }
}
