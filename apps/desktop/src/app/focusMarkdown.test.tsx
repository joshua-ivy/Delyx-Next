import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { MarkdownMessage } from "./focusMarkdown";

afterEach(cleanup);

describe("MarkdownMessage", () => {
  it("renders markdown blocks and inline formatting as elements", () => {
    const { container } = render(
      <MarkdownMessage text={[
        "## Options",
        "- **React** renders UI",
        "- Use `Tauri` for desktop",
        "```ts",
        "const safe = true;",
        "```",
      ].join("\n")} />,
    );

    expect(screen.getByRole("heading", { name: "Options" })).not.toBeNull();
    expect(screen.getByText("React").tagName).toBe("STRONG");
    expect(screen.getByText("Tauri").tagName).toBe("CODE");
    expect(container.querySelectorAll("li")).toHaveLength(2);
    expect(container.querySelector("pre code")?.textContent).toBe("const safe = true;");
  });

  it("renders a QA/QC marker as a styled verdict badge, not raw text", () => {
    const { container } = render(
      <MarkdownMessage text={"[[qaqc:pass:Claude Code]]\n\nLooks good."} />,
    );

    const badge = container.querySelector(".qaqc-badge.qaqc-pass");
    expect(badge).not.toBeNull();
    expect(screen.getByText("QA/QC passed")).not.toBeNull();
    expect(screen.getByText("Claude Code")).not.toBeNull();
    // The raw marker text must not leak into the rendered output.
    expect(container.textContent).not.toContain("[[qaqc");
    expect(screen.getByText("Looks good.")).not.toBeNull();
  });

  it("maps each verdict to its badge variant", () => {
    for (const [verdict, klass] of [["fixed", "qaqc-fixed"], ["verified", "qaqc-verified"], ["fail", "qaqc-fail"], ["unclear", "qaqc-unclear"]] as const) {
      const { container } = render(<MarkdownMessage text={`[[qaqc:${verdict}:Codex]]`} />);
      expect(container.querySelector(`.qaqc-badge.${klass}`)).not.toBeNull();
      cleanup();
    }
  });
});
