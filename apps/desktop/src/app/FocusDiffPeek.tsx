import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import { FocusIcon } from "./focusAtoms";
import { activePatchApplyApproval } from "./patchApplyApproval";
import { activePatchRestoreApproval } from "./patchRestoreApproval";

interface FocusDiffPeekProps {
  onPatchAction: (patchId: string) => void;
  patches: PatchProposalView[];
  proposals: ActionProposalView[];
}

export function FocusDiffPeek({ onPatchAction, patches, proposals }: FocusDiffPeekProps) {
  const file = patches.flatMap((patch) => patch.files.map((item) => ({ item, patch })))[0];
  if (!file) {
    return null;
  }
  const action = patchAction(file.patch, proposals);
  return (
    <div className="peek">
      <div className="peek-head">
        <FocusIcon name="diff" /> {file.item.path}
        <span className="stat">{patchStateLabel(file.patch)}</span>
      </div>
      {file.item.diff.slice(0, 8).map((line, index) => (
        <div className={`dl ${line.kind === "added" ? "add" : line.kind === "removed" ? "del" : "ctx"}`} key={index}>
          <span className="ln">{line.kind === "added" ? "+" : line.kind === "removed" ? "-" : index + 1}</span>
          <span className="tx">{line.text || " "}</span>
        </div>
      ))}
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
