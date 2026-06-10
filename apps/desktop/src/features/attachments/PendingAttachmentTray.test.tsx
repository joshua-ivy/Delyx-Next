import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PendingAttachmentTray } from "./PendingAttachmentTray";
import type { AttachmentProposal } from "./attachmentTypes";

afterEach(cleanup);

function proposal(over: Partial<AttachmentProposal>): AttachmentProposal {
  return {
    id: "a1",
    projectId: "p1",
    threadId: "t1",
    sourceKind: "local_file",
    detectedKind: "code",
    displayName: "main.rs",
    sourceLocator: "main.rs",
    proposedScope: { mode: "thread" },
    estimatedBytes: 1200,
    estimatedFileCount: null,
    requiresApproval: false,
    risk: "low",
    status: "pending",
    createdAt: "",
    updatedAt: "",
    ...over,
  };
}

describe("PendingAttachmentTray", () => {
  it("renders nothing when there are no proposals", () => {
    const { container } = render(<PendingAttachmentTray proposals={[]} />);
    expect(container.querySelector(".attach-tray")).toBeNull();
  });

  it("shows truthful status labels per proposal", () => {
    render(
      <PendingAttachmentTray
        proposals={[
          proposal({ id: "a", displayName: "f.ts", status: "pending" }),
          proposal({ id: "b", displayName: "spec.pdf", detectedKind: "pdf", status: "needs_approval" }),
        ]}
      />,
    );
    expect(screen.getByText("ready")).not.toBeNull();
    expect(screen.getByText("needs approval")).not.toBeNull();
  });

  it("renders an unsupported chip for binary kinds regardless of status", () => {
    render(<PendingAttachmentTray proposals={[proposal({ detectedKind: "binary", status: "pending" })]} />);
    expect(screen.getByText("unsupported")).not.toBeNull();
  });

  it("calls onRemove when the chip's remove control is clicked", () => {
    const onRemove = vi.fn();
    render(<PendingAttachmentTray onRemove={onRemove} proposals={[proposal({ id: "x", displayName: "f.ts" })]} />);
    fireEvent.click(screen.getByRole("button", { name: "Remove f.ts" }));
    expect(onRemove).toHaveBeenCalledWith("x");
  });
});
