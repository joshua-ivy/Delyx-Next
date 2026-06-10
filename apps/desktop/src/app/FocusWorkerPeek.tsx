import type { ActionProposalView } from "../features/approvals/approvalTypes";
import {
  workerAdapterFromCards,
  workerCards,
  workerLabel,
  workerModeFromCards,
  workerTaskFromCard,
} from "./appShellWorkerActions";

/**
 * Strong-worker launch state for the active run: shows the queued task, the
 * approval state of both cards, and a Launch action only when both are
 * approved. Denied/expired states stay visible instead of disappearing.
 */
export function FocusWorkerPeek({
  proposals,
  runId,
  onLaunch,
}: {
  proposals: ActionProposalView[];
  runId: string | undefined;
  onLaunch?: () => void;
}) {
  const cards = workerCards(runId, proposals);
  if (!cards || !onLaunch) {
    return null;
  }
  const label = workerLabel(workerAdapterFromCards(cards));
  const task = workerTaskFromCard(cards.external);
  const write = workerModeFromCards(cards) === "workspace_write";
  const plannedCount = write ? (cards.external.scope.paths?.length ?? 0) : 0;
  const ready = cards.external.status === "approved" && cards.terminal.status === "approved";
  const refused = ["denied", "expired"].includes(cards.external.status)
    || ["denied", "expired"].includes(cards.terminal.status);
  return (
    <div className="worker-peek" data-screen-label="Strong worker">
      <div className="worker-peek-meta">
        <b>Strong worker — {label}{write ? ` · write (${plannedCount} planned file${plannedCount === 1 ? "" : "s"})` : " · read-only"}</b>
        <span>{task}</span>
      </div>
      <div className="worker-peek-ctl">
        {refused && <span className="tag off">approval {cards.external.status === "approved" ? cards.terminal.status : cards.external.status}</span>}
        {!refused && !ready && <span className="tag warn">waiting for both approvals</span>}
        {ready && <button className="btn-send" onClick={onLaunch} type="button">Launch worker</button>}
      </div>
    </div>
  );
}
