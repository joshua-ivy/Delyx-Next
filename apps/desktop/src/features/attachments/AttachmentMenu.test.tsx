import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AttachmentMenu } from "./AttachmentMenu";

afterEach(cleanup);

describe("AttachmentMenu", () => {
  it("renders the documented attachment sources", () => {
    render(<AttachmentMenu onClose={vi.fn()} onDraft={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Add file" })).not.toBeNull();
    expect(screen.getByRole("button", { name: "Add folder" })).not.toBeNull();
    expect(screen.getByRole("button", { name: /screenshot/i })).not.toBeNull();
    expect(screen.getByRole("button", { name: /MCP provider/i })).not.toBeNull();
  });

  it("emits a local_file draft when a file is picked", () => {
    const onDraft = vi.fn();
    const onClose = vi.fn();
    render(<AttachmentMenu onClose={onClose} onDraft={onDraft} />);

    const input = screen.getByLabelText("Add file") as HTMLInputElement;
    const file = new File(["x".repeat(2048)], "util.ts", { type: "text/plain" });
    fireEvent.change(input, { target: { files: [file] } });

    expect(onDraft).toHaveBeenCalledTimes(1);
    expect(onDraft).toHaveBeenCalledWith(
      expect.objectContaining({ sourceKind: "local_file", displayName: "util.ts", detectedKind: "code", estimatedBytes: 2048 }),
      expect.any(File),
    );
    expect(onClose).toHaveBeenCalled();
  });

  it("emits a url draft from the inline input", () => {
    const onDraft = vi.fn();
    render(<AttachmentMenu onClose={vi.fn()} onDraft={onDraft} />);
    const input = screen.getByLabelText("URL or source");
    fireEvent.change(input, { target: { value: "https://example.com/spec" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onDraft).toHaveBeenCalledWith(
      expect.objectContaining({ sourceKind: "url", sourceLocator: "https://example.com/spec" }),
    );
  });
});
