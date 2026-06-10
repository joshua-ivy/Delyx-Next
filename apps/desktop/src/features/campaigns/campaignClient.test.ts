import { describe, expect, it } from "vitest";
import { parseQaqcReply } from "./campaignClient";

describe("parseQaqcReply", () => {
  it("recognizes a clean verdict", () => {
    expect(parseQaqcReply("VERDICT: clean")).toEqual({ status: "clean" });
    expect(parseQaqcReply("Some preamble...\nverdict: CLEAN").status).toBe("clean");
  });

  it("captures issue notes after an issues verdict", () => {
    const reply = "VERDICT: issues\n- The M1 Garand was not issued until 1936.\n- Mills died on turn 2.";
    const parsed = parseQaqcReply(reply);
    expect(parsed.status).toBe("corrected");
    expect(parsed.notes).toContain("M1 Garand");
    expect(parsed.notes).toContain("Mills died");
  });

  it("falls back to skipped when the reviewer rambles", () => {
    const parsed = parseQaqcReply("I am not sure what you want from me.");
    expect(parsed.status).toBe("skipped");
    expect(parsed.notes).toContain("not sure");
  });

  it("handles an issues verdict with no detail", () => {
    const parsed = parseQaqcReply("VERDICT: issues");
    expect(parsed.status).toBe("corrected");
    expect(parsed.notes).toBe("Issues flagged without detail.");
  });
});
