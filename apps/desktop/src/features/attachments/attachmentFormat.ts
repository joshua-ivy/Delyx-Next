import type {
  AttachmentDraft,
  AttachmentKind,
  AttachmentParseStatus,
  AttachmentProposalStatus,
} from "./attachmentTypes";

/** Human label for a proposal status. */
export function statusLabel(status: AttachmentProposalStatus): string {
  switch (status) {
    case "pending":
      return "ready";
    case "needs_approval":
      return "needs approval";
    case "approved":
      return "approved";
    case "denied":
      return "denied";
    case "expired":
      return "expired";
    case "failed":
      return "failed";
    default:
      return status;
  }
}

/** Tag tone (maps to existing .tag CSS variants) for a status. */
export function statusTone(status: AttachmentProposalStatus): "live" | "warn" | "off" {
  switch (status) {
    case "pending":
    case "approved":
      return "live";
    case "needs_approval":
    case "expired":
      return "warn";
    case "denied":
    case "failed":
      return "off";
    default:
      return "off";
  }
}

/** Label for an accepted record's parse state (what the user sees on the chip). */
export function parseStatusLabel(status: AttachmentParseStatus): string {
  switch (status) {
    case "not_started":
      return "queued";
    case "reading":
      return "reading";
    case "parsed":
      return "parsed";
    case "partial":
      return "partial";
    case "unsupported":
      return "stored";
    case "failed":
      return "failed";
    default:
      return status;
  }
}

export function parseStatusTone(status: AttachmentParseStatus): "live" | "warn" | "off" {
  switch (status) {
    case "parsed":
      return "live";
    case "partial":
    case "reading":
    case "not_started":
      return "warn";
    case "failed":
      return "off";
    case "unsupported":
      return "off";
    default:
      return "off";
  }
}

/** Read a File's text (browsers don't expose paths, so the UI reads content). */
export function readFileText(file: File): Promise<string> {
  return file.text();
}

const KIND_BY_EXT: Record<string, AttachmentKind> = {
  md: "markdown", markdown: "markdown",
  txt: "text", log: "text", csv: "text", json: "text", yaml: "text", yml: "text", toml: "text",
  rs: "code", ts: "code", tsx: "code", js: "code", jsx: "code", py: "code", go: "code",
  java: "code", c: "code", h: "code", cpp: "code", hpp: "code", cs: "code", rb: "code",
  php: "code", swift: "code", kt: "code", sql: "code", sh: "code",
  pdf: "pdf",
  png: "image", jpg: "image", jpeg: "image", gif: "image", webp: "image", bmp: "image", svg: "image",
  zip: "archive", tar: "archive", gz: "archive", tgz: "archive", rar: "archive", "7z": "archive",
  exe: "binary", dll: "binary", bin: "binary", so: "binary", dylib: "binary", o: "binary",
};

/** Client-side kind guess from a filename (the backend re-infers authoritatively). */
export function inferKind(name: string): AttachmentKind {
  const ext = name.toLowerCase().split(".").pop() ?? "";
  return KIND_BY_EXT[ext] ?? "unknown";
}

/** Whether a guessed kind is something Delyx can't ingest, for an early chip. */
export function isUnsupportedKind(kind: AttachmentKind): boolean {
  return kind === "binary";
}

/** Text-like kinds can be parsed into chunks (matches the Rust `is_text_like`). */
export function isTextLike(kind: AttachmentKind): boolean {
  return kind === "text" || kind === "code" || kind === "markdown";
}

/** Build a proposal draft from a dropped/picked File (no path access in browsers). */
export function draftFromFile(file: File): AttachmentDraft {
  return {
    sourceKind: "local_file",
    displayName: file.name,
    sourceLocator: file.name,
    detectedKind: inferKind(file.name),
    estimatedBytes: file.size,
  };
}

export function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null) {
    return "";
  }
  if (bytes >= 1_000_000) {
    return `${(bytes / 1_000_000).toFixed(1)} MB`;
  }
  if (bytes >= 1_000) {
    return `${Math.round(bytes / 1_000)} KB`;
  }
  return `${bytes} B`;
}
