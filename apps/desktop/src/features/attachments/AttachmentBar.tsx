import { useEffect, useRef, useState } from "react";
import { AttachmentMenu } from "./AttachmentMenu";
import { AttachmentRecordTray } from "./AttachmentRecordTray";
import { PendingAttachmentTray } from "./PendingAttachmentTray";
import {
  approveAttachment,
  createContextPack,
  denyAttachment,
  loadAttachmentSnapshot,
  parseAttachment,
  proposeAttachment,
} from "./attachmentClient";
import { draftFromFile, isTextLike, readFileText } from "./attachmentFormat";
import type { AttachmentDraft, AttachmentProposal, AttachmentRecord, ContextPack } from "./attachmentTypes";

/**
 * Composer attachment surface and pipeline driver: `+` menu + drag/drop propose
 * through one path; needs-approval chips get Approve/Deny; approving creates a
 * record and (for text) parses its content into chunks; a Build-context action
 * shows what the model will actually see. Everything reflects real backend state.
 */
export function AttachmentBar({
  projectId,
  threadId,
}: {
  projectId?: string;
  threadId?: string;
}) {
  const [proposals, setProposals] = useState<AttachmentProposal[]>([]);
  const [records, setRecords] = useState<AttachmentRecord[]>([]);
  const [pack, setPack] = useState<ContextPack | undefined>(undefined);
  const [menuOpen, setMenuOpen] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [note, setNote] = useState<string | undefined>(undefined);
  const dragDepth = useRef(0);
  // Dropped/picked File objects, keyed by source locator, so approval can read
  // content to parse (browsers don't expose file paths).
  const files = useRef<Map<string, File>>(new Map());

  useEffect(() => {
    files.current.clear();
    setPack(undefined);
    setNote(undefined);
    setProposals([]);
    setRecords([]);
    if (!projectId) {
      return;
    }
    let active = true;
    void loadAttachmentSnapshot(projectId, threadId)
      .then((snapshot) => {
        if (!active) return;
        // Approved proposals already appear as records, so don't show them twice.
        setProposals(snapshot.proposals.filter((proposal) => proposal.status !== "approved"));
        setRecords(snapshot.records);
      })
      .catch(() => {
        // Desktop runtime unavailable (web preview) — leave the trays empty.
      });
    return () => {
      active = false;
    };
  }, [projectId, threadId]);

  function mergeProposal(proposal: AttachmentProposal) {
    setProposals((current) => [proposal, ...current.filter((item) => item.id !== proposal.id)]);
  }

  function mergeRecord(record: AttachmentRecord) {
    setRecords((current) => [record, ...current.filter((item) => item.id !== record.id)]);
  }

  async function propose(draft: AttachmentDraft, file?: File) {
    if (!projectId) {
      return;
    }
    if (file) {
      files.current.set(draft.sourceLocator, file);
    }
    try {
      mergeProposal(await proposeAttachment({ ...draft, projectId, threadId }));
    } catch (error) {
      mergeProposal(failedProposal(draft, projectId, threadId, error));
    }
  }

  async function approve(proposalId: string) {
    const proposal = proposals.find((item) => item.id === proposalId);
    try {
      let record = await approveAttachment(proposalId, `ui-${proposalId}`);
      const file = proposal ? files.current.get(proposal.sourceLocator) : undefined;
      if (proposal && isTextLike(proposal.detectedKind) && file) {
        const result = await parseAttachment(record.id, await readFileText(file));
        record = { ...record, parseStatus: result.parseStatus };
      }
      // The proposal is now a record; drop it from the pending tray.
      setProposals((current) => current.filter((item) => item.id !== proposalId));
      mergeRecord(record);
    } catch (error) {
      setNote(error instanceof Error ? error.message : "Approval failed.");
    }
  }

  async function deny(proposalId: string) {
    try {
      mergeProposal(await denyAttachment(proposalId));
    } catch (error) {
      setNote(error instanceof Error ? error.message : "Deny failed.");
    }
  }

  async function buildContext() {
    if (!projectId || !threadId) {
      return;
    }
    try {
      setPack(await createContextPack(projectId, threadId));
    } catch (error) {
      setNote(error instanceof Error ? error.message : "Could not build context.");
    }
  }

  function onDrop(event: React.DragEvent) {
    event.preventDefault();
    dragDepth.current = 0;
    setDragging(false);
    Array.from(event.dataTransfer?.files ?? []).forEach((file) =>
      void propose(draftFromFile(file), file),
    );
  }

  if (!projectId) {
    return null;
  }

  const hasParsed = records.some((record) => record.parseStatus === "parsed" || record.parseStatus === "partial");

  return (
    <div
      className={`attach-bar${dragging ? " dropping" : ""}`}
      onDragEnter={(event) => {
        event.preventDefault();
        dragDepth.current += 1;
        setDragging(true);
      }}
      onDragOver={(event) => event.preventDefault()}
      onDragLeave={() => {
        dragDepth.current -= 1;
        if (dragDepth.current <= 0) setDragging(false);
      }}
      onDrop={onDrop}
    >
      <div className="attach-row">
        <button aria-label="Add attachment" className="icon-btn" onClick={() => setMenuOpen((open) => !open)} type="button">+</button>
        {dragging && <span className="attach-hint">Drop files to attach</span>}
        <PendingAttachmentTray onApprove={(id) => void approve(id)} onDeny={(id) => void deny(id)} onRemove={removeLocal} proposals={proposals} />
        <AttachmentRecordTray records={records} />
        {hasParsed && <button className="attach-context-btn" onClick={() => void buildContext()} type="button">Build context</button>}
      </div>
      {pack && (
        <div className="attach-pack" role="status">
          Context pack: {pack.items.length} chunk(s), {pack.usedTokens}/{pack.budgetTokens} tokens
          {pack.excludedCount > 0 ? `, ${pack.excludedCount} excluded (budget)` : ""} · {pack.strategy}
        </div>
      )}
      {note && <div className="attach-note">{note}</div>}
      {menuOpen && <AttachmentMenu onClose={() => setMenuOpen(false)} onDraft={(draft, file) => void propose(draft, file)} />}
    </div>
  );

  function removeLocal(id: string) {
    setProposals((current) => current.filter((item) => item.id !== id));
  }
}

function failedProposal(
  draft: AttachmentDraft,
  projectId: string,
  threadId: string | undefined,
  error: unknown,
): AttachmentProposal {
  const message = error instanceof Error ? error.message : String(error);
  return {
    id: `failed-${draft.sourceLocator}`,
    projectId,
    threadId,
    sourceKind: draft.sourceKind,
    detectedKind: draft.detectedKind ?? "unknown",
    displayName: draft.displayName,
    sourceLocator: draft.sourceLocator,
    proposedScope: { mode: "thread" },
    estimatedBytes: draft.estimatedBytes ?? null,
    estimatedFileCount: draft.estimatedFileCount ?? null,
    requiresApproval: false,
    approvalReason: message,
    risk: "low",
    status: "failed",
    approvalId: null,
    createdAt: "",
    updatedAt: "",
  };
}
