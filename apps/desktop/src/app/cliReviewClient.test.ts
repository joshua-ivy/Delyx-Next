import { describe, expect, it } from "vitest";

import { qaqcVerdictMessage, type CliReviewResult } from "./cliReviewClient";

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
