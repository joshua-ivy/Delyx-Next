import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { FocusIcon } from "./focusAtoms";
import { activePatchApplyApproval } from "./patchApplyApproval";
import { activePatchRestoreApproval } from "./patchRestoreApproval";

interface FocusDiffPeekProps {
  onPatchAction: (patchId: string) => void;
  patches: PatchProposalView[];
  proposals: ActionProposalView[];
  run?: AgentRunView;
}

export function FocusDiffPeek({ onPatchAction, patches, proposals, run }: FocusDiffPeekProps) {
  const file = patches.flatMap((patch) => patch.files.map((item) => ({ item, patch })))[0];
  if (!file) {
    return null;
  }
  const action = patchAction(file.patch, proposals);
  return (
    <div className="peek">
      <div className="peek-head">
        <FocusIcon name="diff" /> {file.item.path}
        <span className="stat">{file.item.changeKind} / {patchStateLabel(file.patch)}</span>
      </div>
      {file.item.diff.slice(0, 8).map((line, index) => (
        <div className={`dl ${line.kind === "added" ? "add" : line.kind === "removed" ? "del" : "ctx"}`} key={index}>
          <span className="ln">{line.kind === "added" ? "+" : line.kind === "removed" ? "-" : index + 1}</span>
          <span className="tx">{line.text || " "}</span>
        </div>
      ))}
      <RollbackDetail patch={file.patch} run={run} />
      {action && (
        <div className="plan-actions">
          <button className={`select${action.tone === "danger" ? " danger" : ""}`} onClick={() => onPatchAction(file.patch.id)} type="button">
            {action.label}
          </button>
        </div>
      )}
    </div>
  );
}

function patchAction(patch: PatchProposalView, proposals: ActionProposalView[]) {
  if (patch.status === "proposed") {
    const approval = activePatchApplyApproval(proposals, patch);
    const canApply = !approval || approval.status === "approved" || approval.status === "expired";
    return canApply ? { label: approval?.status === "approved" ? "Apply patch" : "Request apply approval", tone: "default" } : undefined;
  }
  if (patch.status === "applied") {
    const approval = activePatchRestoreApproval(proposals, patch);
    const canRestore = !approval || approval.status === "approved" || approval.status === "expired";
    return canRestore ? { label: approval?.status === "approved" ? "Restore checkpoint" : "Request restore approval", tone: "danger" } : undefined;
  }
  return undefined;
}

function patchStateLabel(patch: PatchProposalView) {
  return patch.status === "applied" && patch.checkpointId ? `applied / ${patch.checkpointId}` : patch.status;
}

function RollbackDetail({ patch, run }: { patch: PatchProposalView; run?: AgentRunView }) {
  if (patch.status === "proposed") {
    return null;
  }
  const checkpointFiles = checkpointFileSummary(patch);
  const failure = latestRestoreFailure(run);
  return (
    <div className="patch-rollback">
      {checkpointFiles && <span>Checkpoint files: {checkpointFiles}</span>}
      {patch.restoreApprovalId && <span>Restore approval: {patch.restoreApprovalId}</span>}
      {failure && <span className="danger-text">Restore blocked: {failure}</span>}
      <span>{patch.status === "restored" ? "Review restored files before continuing." : "Restore is allowed only while files still match this applied patch."}</span>
    </div>
  );
}

function checkpointFileSummary(patch: PatchProposalView) {
  if (patch.checkpointFiles.length === 0) {
    return undefined;
  }
  const shown = patch.checkpointFiles.slice(0, 3).map((file) => file.path);
  const hidden = patch.checkpointFiles.length - shown.length;
  return hidden > 0 ? `${shown.join(", ")} +${hidden} more` : shown.join(", ");
}

function latestRestoreFailure(run?: AgentRunView) {
  return [...(run?.events ?? [])].reverse().find((event) => (
    event.kind === "agent_executor.failed" && /patch restore/i.test(event.message)
  ))?.message;
}
