import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AttachmentBar } from "./AttachmentBar";
import {
  approveAttachment,
  createContextPack,
  denyAttachment,
  loadAttachmentSnapshot,
  parseAttachment,
  proposeAttachment,
} from "./attachmentClient";
import type { AttachmentProposal, AttachmentRecord } from "./attachmentTypes";

vi.mock("./attachmentClient", () => ({
  approveAttachment: vi.fn(),
  createContextPack: vi.fn(),
  denyAttachment: vi.fn(),
  loadAttachmentSnapshot: vi.fn(),
  parseAttachment: vi.fn(),
  proposeAttachment: vi.fn(),
}));

const loadSnapshot = vi.mocked(loadAttachmentSnapshot);
const propose = vi.mocked(proposeAttachment);
const approve = vi.mocked(approveAttachment);
const deny = vi.mocked(denyAttachment);
const parse = vi.mocked(parseAttachment);
const buildPack = vi.mocked(createContextPack);

afterEach(cleanup);
beforeEach(() => {
  vi.clearAllMocks();
  loadSnapshot.mockResolvedValue({ projectId: "p1", threadId: "t1", proposals: [], records: [] });
});

function proposal(over: Partial<AttachmentProposal>): AttachmentProposal {
  return {
    id: "a1", projectId: "p1", threadId: "t1", sourceKind: "local_file", detectedKind: "code",
    displayName: "util.ts", sourceLocator: "util.ts", proposedScope: { mode: "thread" },
    estimatedBytes: 10, estimatedFileCount: null, requiresApproval: false, risk: "low",
    status: "pending", createdAt: "", updatedAt: "", ...over,
  };
}

function record(over: Partial<AttachmentRecord>): AttachmentRecord {
  return {
    id: "a1", projectId: "p1", threadId: "t1", messageId: null, runId: null, sourceKind: "local_file",
    detectedKind: "code", displayName: "util.ts", originalLocator: "util.ts", localReferencePath: null,
    contentHash: null, bytes: 10, parseStatus: "not_started", indexStatus: "not_indexed",
    approvalId: "ui-a1", createdAt: "", updatedAt: "", ...over,
  };
}

describe("AttachmentBar", () => {
  it("renders nothing without a project id", () => {
    const { container } = render(<AttachmentBar threadId="t1" />);
    expect(container.firstChild).toBeNull();
  });

  it("proposes a dropped file and shows a truthful chip", async () => {
    propose.mockResolvedValue(proposal({ displayName: "spec.pdf", detectedKind: "pdf", status: "needs_approval" }));
    const { container } = render(<AttachmentBar projectId="p1" threadId="t1" />);
    await waitFor(() => expect(loadSnapshot).toHaveBeenCalled());

    fireEvent.drop(container.querySelector(".attach-bar")!, {
      dataTransfer: { files: [new File(["%PDF"], "spec.pdf", { type: "application/pdf" })] },
    });
    await waitFor(() => expect(propose).toHaveBeenCalled());
    expect(await screen.findByText("needs approval")).not.toBeNull();
  });

  it("approves a needs-approval file, parses it, and shows a parsed record", async () => {
    propose.mockResolvedValue(proposal({ status: "needs_approval", detectedKind: "code", sourceLocator: "util.ts", displayName: "util.ts" }));
    approve.mockResolvedValue(record({ parseStatus: "not_started" }));
    parse.mockResolvedValue({ attachmentId: "a1", parseStatus: "parsed", chunkCount: 2, partial: false });

    const { container } = render(<AttachmentBar projectId="p1" threadId="t1" />);
    await waitFor(() => expect(loadSnapshot).toHaveBeenCalled());

    fireEvent.drop(container.querySelector(".attach-bar")!, {
      dataTransfer: { files: [new File(["let x = 1;"], "util.ts", { type: "text/plain" })] },
    });
    fireEvent.click(await screen.findByRole("button", { name: "Approve util.ts" }));

    await waitFor(() => expect(approve).toHaveBeenCalledWith("a1", "ui-a1"));
    await waitFor(() => expect(parse).toHaveBeenCalledWith("a1", "let x = 1;"));
    expect(await screen.findByText("parsed")).not.toBeNull();
  });

  it("denies a needs-approval proposal", async () => {
    propose.mockResolvedValue(proposal({ status: "needs_approval" }));
    deny.mockResolvedValue(proposal({ status: "denied" }));
    const { container } = render(<AttachmentBar projectId="p1" threadId="t1" />);
    await waitFor(() => expect(loadSnapshot).toHaveBeenCalled());

    fireEvent.drop(container.querySelector(".attach-bar")!, {
      dataTransfer: { files: [new File(["x"], "util.ts")] },
    });
    fireEvent.click(await screen.findByRole("button", { name: "Deny util.ts" }));
    await waitFor(() => expect(deny).toHaveBeenCalledWith("a1"));
    expect(await screen.findByText("denied")).not.toBeNull();
  });

  it("shows a failed chip when proposing rejects", async () => {
    propose.mockRejectedValue(new Error("bridge down"));
    const { container } = render(<AttachmentBar projectId="p1" threadId="t1" />);
    await waitFor(() => expect(loadSnapshot).toHaveBeenCalled());
    fireEvent.drop(container.querySelector(".attach-bar")!, {
      dataTransfer: { files: [new File(["x"], "a.ts")] },
    });
    expect(await screen.findByText("failed")).not.toBeNull();
  });

  it("builds a context pack and summarizes inclusion", async () => {
    loadSnapshot.mockResolvedValue({
      projectId: "p1", threadId: "t1", proposals: [],
      records: [record({ parseStatus: "parsed" })],
    });
    buildPack.mockResolvedValue({
      id: "pack-1", projectId: "p1", threadId: "t1", runId: null, strategy: "direct_excerpt",
      budgetTokens: 4000, usedTokens: 120, status: "ready",
      items: [{ attachmentId: "a1", evidenceRecordId: null, locator: "util.ts#L1-L80", text: "x", tokenEstimate: 120, inclusionReason: "within budget" }],
      createdAt: "", excludedCount: 0,
    });
    render(<AttachmentBar projectId="p1" threadId="t1" />);
    await waitFor(() => expect(loadSnapshot).toHaveBeenCalled());

    fireEvent.click(await screen.findByRole("button", { name: "Build context" }));
    await waitFor(() => expect(buildPack).toHaveBeenCalledWith("p1", "t1"));
    expect(await screen.findByText(/120\/4000 tokens/)).not.toBeNull();
  });
});
