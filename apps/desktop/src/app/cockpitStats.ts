import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import { pendingCount } from "./cockpitReview";
import { escapeHtml } from "./html";

export function emptyThreadStatsBlock() {
  return "";
}

export function threadStatsBlock(
  patches: PatchProposalView[],
  tests: TestArtifactView[],
  proposals: ActionProposalView[],
  run: AgentRunView | undefined,
) {
  if (!run) {
    return emptyThreadStatsBlock();
  }
  const pending = pendingCount(proposals);
  const latestEvent = run.events.at(-1)?.message ?? "Run created. Waiting for the next real action.";
  return `<section class="run-activity">
    <div class="run-activity-main">
      <span class="activity-dot ${activityTone(run.status, pending)}"></span>
      <div><strong>${escapeHtml(activityTitle(run.status, pending))}</strong><span>${escapeHtml(latestEvent)}</span></div>
    </div>
    <div class="run-activity-meta">
      ${activityMeta("Plan", run.mode.replaceAll("_", " "))}
      ${activityMeta("Approval", pending > 0 ? `${pending} waiting` : "clear")}
      ${activityMeta("Diff", patches.length > 0 ? `${changedFileCount(patches)} file(s)` : "none")}
      ${activityMeta("Tests", tests.length > 0 ? latestTestStatus(tests) : "not run")}
    </div>
  </section>`;
}

function activityMeta(label: string, value: string) {
  return `<span><b>${escapeHtml(label)}</b>${escapeHtml(value)}</span>`;
}

function changedFileCount(patches: PatchProposalView[]) {
  return patches.reduce((count, patch) => count + patch.files.length, 0);
}

function latestTestStatus(tests: TestArtifactView[]) {
  const latest = tests.at(-1);
  if (!latest) {
    return "not run";
  }
  return latest.status ?? (latest.exitCode === 0 ? "passed" : "failed");
}

function activityTitle(status: AgentRunView["status"], pending: number) {
  if (pending > 0) {
    return "Waiting for your approval";
  }
  const labels: Record<AgentRunView["status"], string> = {
    blocked: "Blocked",
    cancelled: "Cancelled",
    created: "Ready",
    failed: "Failed",
    repairing: "Repairing",
    running: "Working",
    succeeded: "Done",
    waiting_for_approval: "Waiting for your approval",
  };
  return labels[status];
}

function activityTone(status: AgentRunView["status"], pending: number) {
  if (pending > 0 || status === "waiting_for_approval") {
    return "wait";
  }
  if (status === "failed" || status === "blocked" || status === "cancelled") {
    return "bad";
  }
  return status === "succeeded" ? "done" : "live";
}
