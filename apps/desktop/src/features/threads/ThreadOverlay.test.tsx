import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ThreadOverlay } from "./ThreadOverlay";
import type { TaskThread, ThreadStatus, ThreadUiState } from "./threadTypes";

afterEach(cleanup);

describe("ThreadOverlay", () => {
  it("creates a real thread request only after a non-empty goal", () => {
    const onCreateThread = vi.fn();
    renderThreadOverlay({ onCreateThread, threads: [] });

    expect(screen.getByRole<HTMLButtonElement>("button", { name: "Create thread" }).disabled).toBe(true);
    fireEvent.change(screen.getByLabelText("Thread goal"), { target: { value: "Build the local agent" } });
    fireEvent.click(screen.getByRole("button", { name: "Create thread" }));

    expect(onCreateThread).toHaveBeenCalledWith("Build the local agent");
    expect(screen.getByLabelText<HTMLTextAreaElement>("Thread goal").value).toBe("");
    expect(screen.getByText("No active threads in this project.")).not.toBeNull();
  });

  it("selects, archives, and changes status from visible thread state controls", () => {
    const onArchiveActive = vi.fn();
    const onSelectThread = vi.fn();
    const onSetStatus = vi.fn();
    renderThreadOverlay({ onArchiveActive, onSelectThread, onSetStatus });

    fireEvent.click(screen.getByRole("button", { name: /Repair parser/ }));
    fireEvent.click(screen.getByRole("button", { name: "blocked" }));
    fireEvent.click(screen.getByRole("button", { name: "Archive active" }));

    expect(onSelectThread).toHaveBeenCalledWith("thread-1");
    expect(onSetStatus).toHaveBeenCalledWith("blocked");
    expect(onArchiveActive).toHaveBeenCalledTimes(1);
    expect(screen.queryByText("Archived thread")).toBeNull();
  });

  it("renders empty and error thread states without hiding the reason", () => {
    renderThreadOverlay({ activeThread: undefined, state: "empty", threads: [] });
    expect(screen.getByText("No active thread")).not.toBeNull();
    expect(screen.getByText("Empty: no active threads are linked to this project.")).not.toBeNull();
    cleanup();

    renderThreadOverlay({ activeThread: undefined, state: "error", threads: [] });
    expect(screen.getByText("Error: thread goal is required before creation.")).not.toBeNull();
  });
});

function renderThreadOverlay(options: {
  activeThread?: TaskThread;
  onArchiveActive?: () => void;
  onCreateThread?: (goal: string) => void;
  onSelectThread?: (threadId: string) => void;
  onSetStatus?: (status: ThreadStatus) => void;
  state?: ThreadUiState;
  threads?: TaskThread[];
} = {}) {
  const activeThread = Object.hasOwn(options, "activeThread") ? options.activeThread : thread();
  const onArchiveActive = options.onArchiveActive ?? vi.fn();
  const onCreateThread = options.onCreateThread ?? vi.fn();
  const onSelectThread = options.onSelectThread ?? vi.fn();
  const onSetStatus = options.onSetStatus ?? vi.fn();
  const state = options.state ?? "ready";
  const threads = options.threads ?? [thread(), { ...thread(), archived: true, id: "archived", title: "Archived thread" }];
  return render(
    <ThreadOverlay
      activeThread={activeThread}
      onArchiveActive={onArchiveActive}
      onClose={vi.fn()}
      onCreateThread={onCreateThread}
      onSelectThread={onSelectThread}
      onSetStatus={onSetStatus}
      open
      state={state}
      threads={threads}
    />,
  );
}

function thread(): TaskThread {
  return {
    activeRunId: "run-1",
    archived: false,
    createdAt: "2026-06-08T00:00:00.000Z",
    createdLabel: "now",
    goal: "Repair parser",
    id: "thread-1",
    messages: [{ body: "Repair parser", role: "user" }],
    mode: "build",
    projectId: "project-1",
    runIds: ["run-1"],
    status: "building",
    title: "Repair parser",
    updatedAt: "2026-06-08T00:00:00.000Z",
  };
}
