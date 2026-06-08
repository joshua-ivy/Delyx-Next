import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ModelSettingsView } from "../models/modelTypes";
import { WorkspaceOverlay } from "./WorkspaceOverlay";
import type { WorkspaceProject, WorkspaceUiState } from "./workspaceTypes";

afterEach(cleanup);

describe("WorkspaceOverlay", () => {
  it("adds and removes local projects from the visible workspace menu", () => {
    const onAddProject = vi.fn();
    const onRemoveProject = vi.fn();
    renderWorkspace({ onAddProject, onRemoveProject });

    fireEvent.click(screen.getByRole("button", { name: /Repo/ }));
    fireEvent.change(screen.getByLabelText("Project path"), { target: { value: "C:/repo-next" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    fireEvent.click(screen.getByRole("button", { name: "Remove" }));

    expect(onAddProject).toHaveBeenCalledWith("C:/repo-next");
    expect(onRemoveProject).toHaveBeenCalledWith("project-1");
    expect(screen.getByText("Pinned projects: Repo")).not.toBeNull();
  });

  it("renders empty, loading, denied, and error project states truthfully", () => {
    renderWorkspace({ state: "empty", project: project({ indexedFiles: [], name: "Missing project" }) });
    expect(screen.getByText("No project linked")).not.toBeNull();
    cleanup();

    renderWorkspace({ state: "loading", project: project({ indexedFiles: [] }) });
    expect(screen.getByText("Reading workspace index")).not.toBeNull();
    expect(screen.getByText("Loading")).not.toBeNull();
    cleanup();

    renderWorkspace({ state: "denied", project: project({ indexedFiles: [] }) });
    expect(screen.getByText("Denied")).not.toBeNull();
    expect(screen.getByText(/approved root only/)).not.toBeNull();
    cleanup();

    renderWorkspace({ state: "error", project: project({ indexedFiles: [] }) });
    expect(screen.getByText("Error")).not.toBeNull();
  });
});

function renderWorkspace({
  onAddProject = vi.fn(),
  onRemoveProject = vi.fn(),
  project: activeProject = project(),
  state = "ready",
}: {
  onAddProject?: (path: string) => void;
  onRemoveProject?: (projectId: string) => void;
  project?: WorkspaceProject;
  state?: WorkspaceUiState;
} = {}) {
  return render(
    <WorkspaceOverlay
      activeThreadCount={2}
      lastRun={{ status: "blocked" } as never}
      modelSettings={modelSettings}
      onAddProject={onAddProject}
      onClose={vi.fn()}
      onRemoveProject={onRemoveProject}
      open
      project={activeProject}
      projects={[activeProject]}
      state={state}
    />,
  );
}

function project(overrides: Partial<WorkspaceProject> = {}): WorkspaceProject {
  return {
    approvalPolicy: "approval-gated",
    approvedRoots: ["C:/repo"],
    git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
    id: "project-1",
    indexedFiles: ["package.json", "src/app.ts"],
    isolation: { detail: "No isolation active.", label: "none", mode: "none" },
    lastOpenedLabel: "now",
    name: "Repo",
    path: "C:/repo",
    pinned: true,
    rulesFiles: [],
    ...overrides,
  };
}

const modelSettings: ModelSettingsView = {
  providers: [{
    detail: "Ollama ready.",
    id: "ollama-local",
    kind: "ollama",
    label: "Ollama",
    models: ["qwen3-coder:30b"],
    requiresSecret: false,
    status: "ready",
  }],
  routes: [{ modelId: "qwen3-coder:30b", providerId: "ollama-local", role: "coding", saved: true }],
  selectedProviderId: "ollama-local",
};
