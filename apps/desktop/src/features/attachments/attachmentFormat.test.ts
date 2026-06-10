import { describe, expect, it } from "vitest";
import {
  draftFromFile,
  inferKind,
  isUnsupportedKind,
  statusLabel,
  statusTone,
} from "./attachmentFormat";

describe("attachmentFormat", () => {
  it("maps statuses to truthful labels and tones", () => {
    expect(statusLabel("needs_approval")).toBe("needs approval");
    expect(statusTone("needs_approval")).toBe("warn");
    expect(statusTone("denied")).toBe("off");
    expect(statusTone("pending")).toBe("live");
  });

  it("guesses kind from extension and flags binaries as unsupported", () => {
    expect(inferKind("main.rs")).toBe("code");
    expect(inferKind("spec.PDF")).toBe("pdf");
    expect(inferKind("blob")).toBe("unknown");
    expect(isUnsupportedKind(inferKind("tool.exe"))).toBe(true);
    expect(isUnsupportedKind("code")).toBe(false);
  });

  it("builds a local_file draft from a File", () => {
    const file = new File(["hello world"], "notes.md", { type: "text/markdown" });
    const draft = draftFromFile(file);
    expect(draft.sourceKind).toBe("local_file");
    expect(draft.displayName).toBe("notes.md");
    expect(draft.detectedKind).toBe("markdown");
    expect(draft.estimatedBytes).toBe(11);
  });
});
