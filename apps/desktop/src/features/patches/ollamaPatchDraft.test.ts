import { describe, expect, it } from "vitest";

import { createPatchProposalRequestFromOllamaText } from "./ollamaPatchDraft";
import type { PlanView } from "../plans/planTypes";
import type { WorkspaceProject } from "../workspace/workspaceTypes";

describe("createPatchProposalRequestFromOllamaText", () => {
  it("builds a scoped patch request from fenced Ollama JSON", () => {
    const request = createPatchProposalRequestFromOllamaText({
      approvalId: "approval-1",
      clientId: "patch-1",
      plan,
      project,
      readFiles: [{ contents: "export const value = 1;\n", path: "src/main.ts", truncated: false }],
      runId: "run-1",
      text: "```json\n{\"files\":[{\"path\":\"src/main.ts\",\"after\":\"export const value = 2;\\n\"}]}\n```",
    });

    expect(request).toEqual({
      approvalId: "approval-1",
      approvedRoots: ["C:/repo"],
      clientId: "patch-1",
      files: [{ after: "export const value = 2;\n", path: "C:/repo/src/main.ts" }],
      runId: "run-1",
    });
  });

  it("rejects unapproved files and unchanged file contents", () => {
    const base = {
      approvalId: "approval-1",
      clientId: "patch-1",
      plan,
      project,
      readFiles: [{ contents: "before\n", path: "src/main.ts", truncated: false }],
      runId: "run-1",
    };

    expect(() => createPatchProposalRequestFromOllamaText({
      ...base,
      text: "{\"files\":[{\"path\":\"src/secret.ts\",\"after\":\"after\\n\"}]}",
    })).toThrow(/unapproved file/);
    expect(() => createPatchProposalRequestFromOllamaText({
      ...base,
      text: "{\"files\":[{\"path\":\"src/main.ts\",\"after\":\"before\\n\"}]}",
    })).toThrow(/unchanged/);
  });
});

const project: WorkspaceProject = {
  approvalPolicy: "manual",
  approvedRoots: ["C:/repo"],
  git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
  id: "project-1",
  indexedFiles: ["src/main.ts"],
  isolation: { detail: "none", label: "none", mode: "none" },
  lastOpenedLabel: "now",
  name: "Repo",
  path: "C:/repo",
  pinned: true,
  rulesFiles: [],
};

const plan: PlanView = {
  decision: "approved",
  explore: {
    architectureSummary: "TypeScript project.",
    projectCommands: ["npm test"],
    relevantFiles: ["src/main.ts"],
    relevantSymbols: [],
    risks: [],
    suggestedNextSteps: [],
    unknowns: [],
  },
  filesLikelyInvolved: ["src/main.ts"],
  goalUnderstanding: "Update value.",
  permissionsNeeded: ["edit_file"],
  risks: [],
  rollbackStrategy: "Restore the previous contents.",
  steps: ["Update value"],
  testsToRun: ["npm test"],
  threadId: "thread-1",
};
