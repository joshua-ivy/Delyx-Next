import type { AttachmentRecord } from "./attachmentTypes";
import { formatBytes, parseStatusLabel, parseStatusTone } from "./attachmentFormat";

/**
 * Accepted attachments with their real parse state (queued / reading / parsed /
 * partial / stored / failed). No invented "indexed" state — only what the
 * backend reports.
 */
export function AttachmentRecordTray({ records }: { records: AttachmentRecord[] }) {
  if (records.length === 0) {
    return null;
  }
  return (
    <div className="attach-tray" role="list">
      {records.map((record) => (
        <span className="attach-chip done" key={record.id} role="listitem" title={`${record.detectedKind} · ${formatBytes(record.bytes)}`}>
          <span className="attach-chip-name">{record.displayName}</span>
          <span className={`tag ${parseStatusTone(record.parseStatus)}`}>{parseStatusLabel(record.parseStatus)}</span>
        </span>
      ))}
    </div>
  );
}
