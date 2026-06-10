import { useRef, useState } from "react";
import type { AttachmentDraft } from "./attachmentTypes";
import { draftFromFile } from "./attachmentFormat";

/**
 * The composer `+` menu. It only builds attachment *drafts* (source + metadata);
 * the parent attaches the project/thread id and calls `attachment_propose`. The
 * same drafts flow as drag/drop, so both share one ingestion path.
 */
export function AttachmentMenu({
  onDraft,
  onClose,
}: {
  onDraft: (draft: AttachmentDraft, file?: File) => void;
  onClose: () => void;
}) {
  const fileInput = useRef<HTMLInputElement>(null);
  const folderInput = useRef<HTMLInputElement | null>(null);
  const imageInput = useRef<HTMLInputElement>(null);
  const [url, setUrl] = useState("");

  function emit(draft: AttachmentDraft, file?: File) {
    if (file) {
      onDraft(draft, file);
    } else {
      onDraft(draft);
    }
  }

  function onFilesPicked(files: FileList | null, sourceKind: AttachmentDraft["sourceKind"]) {
    if (!files || files.length === 0) {
      return;
    }
    Array.from(files).forEach((file) => emit({ ...draftFromFile(file), sourceKind }, file));
    onClose();
  }

  function onFolderPicked(files: FileList | null) {
    if (!files || files.length === 0) {
      return;
    }
    const list = Array.from(files);
    const totalBytes = list.reduce((sum, file) => sum + file.size, 0);
    const folderName = folderNameFrom(list[0]);
    emit({
      sourceKind: "local_folder",
      displayName: folderName,
      sourceLocator: folderName,
      detectedKind: "folder",
      estimatedBytes: totalBytes,
      estimatedFileCount: list.length,
    });
    onClose();
  }

  async function addClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (text.trim()) {
        emit({
          sourceKind: "clipboard",
          displayName: "Clipboard text",
          sourceLocator: "clipboard",
          detectedKind: "text",
          estimatedBytes: text.length,
        });
      }
    } catch {
      // Clipboard read denied/unavailable — nothing to attach.
    }
    onClose();
  }

  function addUrl() {
    const trimmed = url.trim();
    if (!trimmed) {
      return;
    }
    emit({
      sourceKind: "url",
      displayName: trimmed,
      sourceLocator: trimmed,
      detectedKind: "url",
    });
    setUrl("");
    onClose();
  }

  return (
    <div className="attach-menu" role="menu">
      <input aria-label="Add file" hidden multiple onChange={(e) => onFilesPicked(e.target.files, "local_file")} ref={fileInput} type="file" />
      <input
        aria-label="Add folder"
        hidden
        onChange={(e) => onFolderPicked(e.target.files)}
        ref={(el) => {
          folderInput.current = el;
          if (el) {
            // webkitdirectory isn't in the React types; set it on the DOM node.
            el.setAttribute("webkitdirectory", "");
            el.setAttribute("directory", "");
          }
        }}
        type="file"
      />
      <input accept="image/*" aria-label="Add image" hidden multiple onChange={(e) => onFilesPicked(e.target.files, "screenshot")} ref={imageInput} type="file" />

      <button className="attach-item" onClick={() => fileInput.current?.click()} type="button">Add file</button>
      <button className="attach-item" onClick={() => folderInput.current?.click()} type="button">Add folder</button>
      <button className="attach-item" onClick={() => imageInput.current?.click()} type="button">Add screenshot / image</button>
      <button className="attach-item" onClick={() => void addClipboard()} type="button">Add clipboard text</button>
      <div className="attach-url">
        <input
          aria-label="URL or source"
          className="set-input"
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); addUrl(); } }}
          placeholder="Add URL / source…"
          value={url}
        />
        <button className="attach-item" disabled={!url.trim()} onClick={addUrl} type="button">Add</button>
      </div>
      <button className="attach-item disabled" disabled title="Coming soon" type="button">Add from connector</button>
      <button className="attach-item disabled" disabled title="Coming soon" type="button">Add from MCP provider</button>
    </div>
  );
}

function folderNameFrom(file: File): string {
  // Browsers expose the folder via webkitRelativePath ("folder/sub/file.ext").
  const relative = (file as File & { webkitRelativePath?: string }).webkitRelativePath ?? "";
  const top = relative.split("/")[0];
  return top || "Folder";
}
