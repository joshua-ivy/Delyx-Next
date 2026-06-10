import { describe, expect, it } from "vitest";

import { extractReviewableCode, qaqcVerdictMessage, type CliReviewResult } from "./cliReviewClient";

function result(verdict: CliReviewResult["verdict"], text: string): CliReviewResult {
  return { adapterId: "claude-code", text, verdict };
}

describe("qaqcVerdictMessage", () => {
  it("marks a pass and includes findings text", () => {
    const message = qaqcVerdictMessage("claude-code", result("pass", "looks correct"));
    expect(message).toContain("✓ QA/QC (claude-code): PASS");
    expect(message).toContain("looks correct");
  });

  it("flags a failing review", () => {
    const message = qaqcVerdictMessage("claude-code", result("fail", "off-by-one"));
    expect(message).toContain("⚠ QA/QC (claude-code): FAIL");
    expect(message).toContain("off-by-one");
  });

  it("shows just the header when the review text is empty", () => {
    const message = qaqcVerdictMessage("codex-cli", result("unclear", ""));
    expect(message).toBe("? QA/QC (codex-cli): UNCLEAR");
  });
});

describe("extractReviewableCode", () => {
  it("returns null when there is no fenced code", () => {
    expect(extractReviewableCode("Hello! I can help with that. Just prose here.")).toBeNull();
  });

  it("extracts a single fenced block without the prose", () => {
    const reply = "Here is a script:\n\n```python\nprint('hi')\n```\n\nHope that helps!";
    expect(extractReviewableCode(reply)).toBe("print('hi')");
  });

  it("joins multiple code blocks and drops empty fences", () => {
    const reply = "```js\nconst a = 1;\n```\nfiller\n```\n```\n```ts\nconst b = 2;\n```";
    expect(extractReviewableCode(reply)).toBe("const a = 1;\n\nconst b = 2;");
  });
});
