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
  const events = run.events.slice(-5);
  const meta = [
    activityMeta("Mode", run.mode.replaceAll("_", " ")),
    pending > 0 ? activityMeta("Approval", `${pending} waiting`) : "",
    patches.length > 0 ? activityMeta("Diff", `${changedFileCount(patches)} file(s)`) : "",
    tests.length > 0 ? activityMeta("Tests", latestTestStatus(tests)) : "",
    run.metrics.commandCount > 0 ? activityMeta("Commands", `${run.metrics.commandCount}`) : "",
    run.evidence.length > 0 ? activityMeta("Evidence", `${run.evidence.length}`) : "",
  ].filter(Boolean).join("");
  return `<section class="run-activity">
    <div class="run-activity-main">
      <span class="activity-dot ${activityTone(run.status, pending)}"></span>
      <div><strong>${escapeHtml(activityTitle(run.status, pending))}</strong><span>${escapeHtml(latestEvent)}</span></div>
    </div>
    ${events.length > 1 ? `<div class="run-event-stream">${events.map(eventLine).join("")}</div>` : ""}
    ${meta ? `<div class="run-activity-meta">${meta}</div>` : ""}
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

function eventLine(event: AgentRunView["events"][number]) {
  return `<div class="run-event"><span>${escapeHtml(shortTime(event.createdAt))}</span><b>${escapeHtml(event.kind.replaceAll("_", " "))}</b><em>${escapeHtml(event.message)}</em></div>`;
}

function shortTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
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
