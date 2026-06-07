import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import { pendingCount, testStat } from "./cockpitReview";

export function emptyThreadStatsBlock() {
  return `<div class="stat-row">
          <div class="stat"><div class="sv">0</div><div class="sk">Files touched</div></div>
          <div class="stat"><div class="sv">None</div><div class="sk">Diff</div></div>
          <div class="stat"><div class="sv">Not run</div><div class="sk">Tests</div></div>
          <div class="stat"><div class="sv">0</div><div class="sk">Commands run</div></div>
          <div class="stat"><div class="sv">0</div><div class="sk">Approvals needed</div></div>
          <div class="stat"><div class="sv">None</div><div class="sk">Final answer</div></div>
          <div class="stat"><div class="sv">0</div><div class="sk">Evidence receipts</div></div>
        </div>`;
}

export function threadStatsBlock(
  patches: PatchProposalView[],
  tests: TestArtifactView[],
  proposals: ActionProposalView[],
  run: AgentRunView | undefined,
) {
  return `<div class="stat-row">
          ${stat(filesTouchedStat(patches), "Files touched")}
          ${stat(diffStat(patches), "Diff")}
          ${stat(testStat(tests), "Tests")}
          ${stat(commandsRunStat(tests, run), "Commands run")}
          ${stat(pendingCount(proposals), "Approvals needed")}
          ${stat(finalAnswerStat(run), "Final answer")}
          ${stat(run?.metrics.evidenceCount ?? run?.evidence.length ?? 0, "Evidence receipts")}
        </div>`;
}

function stat(value: number | string, label: string) {
  return `<div class="stat"><div class="sv">${value}</div><div class="sk">${label}</div></div>`;
}

function filesTouchedStat(patches: PatchProposalView[]) {
  return patches.reduce((count, patch) => count + patch.files.length, 0);
}

function diffStat(patches: PatchProposalView[]) {
  return patches.some((patch) => patch.files.length > 0) ? "Ready" : "None";
}

function commandsRunStat(tests: TestArtifactView[], run: AgentRunView | undefined) {
  const commandEvents = run?.events.filter((event) => event.kind.toLowerCase().includes("command")).length ?? 0;
  return Math.max(tests.length, commandEvents, run?.metrics.commandCount ?? 0);
}

function finalAnswerStat(run: AgentRunView | undefined) {
  if (!run) {
    return "None";
  }
  if (run.status === "succeeded" && run.outcome?.summary) {
    return "Ready";
  }
  if (run.status === "failed" || run.status === "cancelled" || run.status === "blocked") {
    return "Failed";
  }
  return "Pending";
}
