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
});
