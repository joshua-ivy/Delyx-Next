import type { AttachmentProposal } from "./attachmentTypes";
import { formatBytes, isUnsupportedKind, statusLabel, statusTone } from "./attachmentFormat";

/**
 * Renders pending attachment chips with their truthful status (ready / needs
 * approval / denied / expired / failed / unsupported). Clicking a chip's remove
 * control calls `onRemove`. No fake "indexed" state — only what the backend says.
 */
export function PendingAttachmentTray({
  proposals,
  onApprove,
  onDeny,
  onRemove,
}: {
  proposals: AttachmentProposal[];
  onApprove?: (id: string) => void;
  onDeny?: (id: string) => void;
  onRemove?: (id: string) => void;
}) {
  if (proposals.length === 0) {
    return null;
  }
  return (
    <div className="attach-tray" role="list">
      {proposals.map((proposal) => {
        const unsupported = isUnsupportedKind(proposal.detectedKind);
        const tone = unsupported ? "off" : statusTone(proposal.status);
        const label = unsupported ? "unsupported" : statusLabel(proposal.status);
        const detail = chipDetail(proposal);
        const needsApproval = proposal.status === "needs_approval";
        return (
          <span className="attach-chip" key={proposal.id} role="listitem" title={proposal.approvalReason ?? detail}>
            <span className="attach-chip-name">{proposal.displayName}</span>
            <span className={`tag ${tone}`}>{label}</span>
            {needsApproval && onApprove && (
              <button aria-label={`Approve ${proposal.displayName}`} className="attach-chip-act ok" onClick={() => onApprove(proposal.id)} type="button">Approve</button>
            )}
            {needsApproval && onDeny && (
              <button aria-label={`Deny ${proposal.displayName}`} className="attach-chip-act no" onClick={() => onDeny(proposal.id)} type="button">Deny</button>
            )}
            {onRemove && (
              <button
                aria-label={`Remove ${proposal.displayName}`}
                className="attach-chip-x"
                onClick={() => onRemove(proposal.id)}
                type="button"
              >
                ×
              </button>
            )}
          </span>
        );
      })}
    </div>
  );
}

function chipDetail(proposal: AttachmentProposal): string {
  const parts: string[] = [proposal.detectedKind];
  if (proposal.estimatedFileCount != null) {
    parts.push(`${proposal.estimatedFileCount} files`);
  } else if (proposal.estimatedBytes != null) {
    parts.push(formatBytes(proposal.estimatedBytes));
  }
  return parts.filter(Boolean).join(" · ");
}
