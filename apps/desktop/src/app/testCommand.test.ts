import { describe, expect, it } from "vitest";

import { firstRunnableTestCommand, parseRunnableTestCommand } from "./testCommand";

describe("test command parsing", () => {
  it("accepts direct test commands and bundled npm paths", () => {
    expect(parseRunnableTestCommand("cargo test --workspace")).toEqual({
      args: ["test", "--workspace"],
      label: "cargo test --workspace",
      program: "cargo",
    });
    expect(parseRunnableTestCommand(".\\.tools\\npm.cmd test")).toEqual({
      args: ["test"],
      label: ".\\.tools\\npm.cmd test",
      program: ".\\.tools\\npm.cmd",
    });
  });

  it("rejects shell chains and non-test commands", () => {
    expect(parseRunnableTestCommand("npm install && npm test")).toBeUndefined();
    expect(parseRunnableTestCommand("powershell -c npm test")).toBeUndefined();
    expect(parseRunnableTestCommand("npm install")).toBeUndefined();
  });

  it("selects the first supported command", () => {
    expect(firstRunnableTestCommand(["No project test command discovered yet.", "npm run test"])?.args).toEqual(["run", "test"]);
  });
});
