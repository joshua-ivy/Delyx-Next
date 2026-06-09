import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ModelSettingsView } from "../features/models/modelTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { FocusShell } from "./FocusShell";

afterEach(cleanup);

describe("FocusShell", () => {
  it("sends trimmed home composer instructions and ignores blank sends", () => {
    const onSendInstruction = vi.fn();
    renderShell({ onSendInstruction });

    const input = screen.getByPlaceholderText("Message Delyx with a real local instruction...");
    fireEvent.change(input, { target: { value: "   build the agent   " } });
    fireEvent.click(screen.getByRole("button", { name: /Send/ }));
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.click(screen.getByRole("button", { name: /Send/ }));

    expect(onSendInstruction).toHaveBeenCalledTimes(1);
    expect(onSendInstruction).toHaveBeenCalledWith("build the agent");
  });

  it("lets the user switch modes from the chips on an active thread", () => {
    renderShell({ activeThread: thread() });

    const stage = () => document.querySelector('[data-screen-label="Active thread"]');
    // Thread status maps to the "explore" mode by default.
    expect(stage()?.getAttribute("data-mode")).toBe("explore");

    fireEvent.click(screen.getByRole("button", { name: "Plan" }));
    expect(stage()?.getAttribute("data-mode")).toBe("plan");
    fireEvent.click(screen.getByRole("button", { name: "Build" }));
    expect(stage()?.getAttribute("data-mode")).toBe("build");
  });

  it("opens settings from the keyboard and shows real desktop shell state", () => {
    renderShell();

    fireEvent.keyDown(window, { ctrlKey: true, key: "," });

    expect(screen.getByText("Settings")).not.toBeNull();
    expect(screen.getByText("Windows shell")).not.toBeNull();
    expect(screen.getByText("single instance; renderer commands; unsigned dev build")).not.toBeNull();
  });
});

function renderShell({ onSendInstruction = vi.fn(), activeThread = undefined as TaskThread | undefined } = {}) {
  return render(
    <FocusShell
      activePlan={undefined}
      activeProject={project()}
      activeRun={undefined}
      activeThread={activeThread}
      desktopShell={{
        mainWindowLabel: "main",
        nativeMenuPolicy: "renderer_command_ui",
        reopenBehavior: "single_instance_focus_main_window",
        signingPolicy: "unsigned_dev_build",
        startupBehavior: "focus_main_window",
      }}
      modelSettings={modelSettings()}
      onApplyPatch={vi.fn()}
      onArchiveActive={vi.fn()}
      onApprovePlan={vi.fn()}
      onDecideProposal={vi.fn()}
      onOpenWorkspace={vi.fn()}
      onRecordFinal={vi.fn()}
      onRefreshModels={vi.fn()}
      onRequestRepair={vi.fn()}
      onResumeRun={vi.fn()}
      onRunCommand={vi.fn()}
      onRunReview={vi.fn()}
      onRunTests={vi.fn()}
      onSelectModel={vi.fn()}
      onSelectQaqc={vi.fn()}
      onSelectThread={vi.fn()}
      onSendInstruction={onSendInstruction}
      patches={[]}
      proposals={[]}
      reviews={[]}
      schedulerDecision={undefined}
      tests={[]}
      threads={[]}
    />,
  );
}

function thread(): TaskThread {
  return {
    activeRunId: undefined,
    archived: false,
    createdAt: "2026-06-09T00:00:00.000Z",
    createdLabel: "now",
    goal: "Build a feature.",
    id: "thread-1",
    messages: [{ body: "hello", role: "user" }],
    mode: "explore",
    projectId: "project-1",
    runIds: [],
    status: "idle",
    title: "Build a feature",
    updatedAt: "2026-06-09T00:00:00.000Z",
  };
}

function project(): WorkspaceProject {
  return {
    approvalPolicy: "approval-gated",
    approvedRoots: ["C:/Users/geaux/Downloads/Delyx Next"],
    git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
    id: "project-1",
    indexedFiles: [],
    isolation: { detail: "No isolation active.", label: "none", mode: "none" },
    lastOpenedLabel: "now",
    name: "delyx-next",
    path: "C:/Users/geaux/Downloads/Delyx Next",
    pinned: true,
    rulesFiles: [],
  };
}

function modelSettings(): ModelSettingsView {
  return {
    providers: [{
      detail: "Ollama is ready.",
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
}
