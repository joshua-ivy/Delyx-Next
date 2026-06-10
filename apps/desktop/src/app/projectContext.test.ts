import { describe, expect, it } from "vitest";
import { buildProjectContextBlock } from "./projectContext";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

function project(over: Partial<WorkspaceProject> = {}): WorkspaceProject {
  return {
    approvalPolicy: "approval-gated",
    approvedRoots: ["C:/code/app"],
    git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
    id: "project-1",
    indexedFiles: ["src/main.rs", "src/lib.rs"],
    isolation: { detail: "none", label: "none", mode: "none" },
    lastOpenedLabel: "now",
    name: "app",
    path: "C:/code/app",
    pinned: true,
    rulesFiles: [],
    ...over,
  };
}

describe("buildProjectContextBlock", () => {
  it("includes project identity, branch, and the repo map", () => {
    const block = buildProjectContextBlock(project(), []);
    expect(block).toContain("Project: app at C:/code/app (git branch: main).");
    expect(block).toContain("Repository files:");
    expect(block).toContain("src/main.rs");
  });

  it("caps the repo map and says how many files were omitted", () => {
    const files = Array.from({ length: 120 }, (_, index) => `src/file-${index}.rs`);
    const block = buildProjectContextBlock(project({ indexedFiles: files }), []);
    expect(block).toContain("first 80 of 120");
    expect(block).toContain("src/file-79.rs");
    expect(block).not.toContain("src/file-80.rs\n");
  });

  it("includes rules file contents with truncation notes", () => {
    const block = buildProjectContextBlock(project(), [
      { path: "AGENTS.md", contents: "Always gate writes.\n", truncated: false },
      { path: "CLAUDE.md", contents: "Be honest.", truncated: true },
    ]);
    expect(block).toContain("Project rules from AGENTS.md:\nAlways gate writes.");
    expect(block).toContain("Project rules from CLAUDE.md (truncated):\nBe honest.");
  });

  it("omits the branch for non-repo folders", () => {
    const block = buildProjectContextBlock(
      project({ git: { branch: "branch not loaded", isRepo: false, uncommittedChanges: null } }),
      [],
    );
    expect(block).toContain("Project: app at C:/code/app.");
    expect(block).not.toContain("git branch");
  });
});
