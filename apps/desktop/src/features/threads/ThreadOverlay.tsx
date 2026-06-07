import { useMemo, useState } from "react";

import type { TaskThread, ThreadStatus, ThreadUiState } from "./threadTypes";

interface ThreadOverlayProps {
  activeThread: TaskThread | undefined;
  open: boolean;
  state: ThreadUiState;
  threads: TaskThread[];
  onArchiveActive: () => void;
  onClose: () => void;
  onCreateThread: (goal: string) => void;
  onSelectThread: (threadId: string) => void;
  onSetStatus: (status: ThreadStatus) => void;
  onShowEmpty: () => void;
}

const statuses: ThreadStatus[] = ["idle", "active", "blocked", "failed", "done"];

export function ThreadOverlay({
  activeThread,
  onArchiveActive,
  onClose,
  onCreateThread,
  onSelectThread,
  onSetStatus,
  onShowEmpty,
  open,
  state,
  threads,
}: ThreadOverlayProps) {
  const [goal, setGoal] = useState("Create a fixture-backed thread manager state.");
  const visibleThreads = useMemo(() => threads.filter((thread) => !thread.archived), [threads]);

  if (!open) {
    return null;
  }

  return (
    <div aria-modal="true" className="thread-backdrop" role="dialog">
      <section className="thread-modal">
        <header>
          <div>
            <p>Thread manager</p>
            <h2>{activeThread?.title ?? "No active thread"}</h2>
          </div>
          <button onClick={onClose} type="button">Close</button>
        </header>

        <div className="thread-grid">
          <section className="thread-card">
            <h3>Create thread</h3>
            <textarea aria-label="Thread goal" onChange={(event) => setGoal(event.target.value)} value={goal} />
            <div className="thread-actions">
              <button onClick={() => onCreateThread(goal)} type="button">Create thread</button>
              <button onClick={onShowEmpty} type="button">Show empty state</button>
            </div>
            <StateNotice state={state} />
          </section>

          <section className="thread-card">
            <h3>Thread list</h3>
            <ul className="thread-list">
              {visibleThreads.map((thread) => (
                <li key={thread.id}>
                  <button
                    className={thread.id === activeThread?.id ? "active" : ""}
                    onClick={() => onSelectThread(thread.id)}
                    type="button"
                  >
                    <span>{thread.title}</span>
                    <StatusPill status={thread.status} />
                  </button>
                </li>
              ))}
              {visibleThreads.length === 0 && <li className="thread-empty">No active threads in this project.</li>}
            </ul>
          </section>

          <section className="thread-card">
            <h3>Conversation</h3>
            {activeThread ? (
              <div className="thread-conversation">
                <p>{activeThread.goal}</p>
                {activeThread.messages.map((message, index) => (
                  <div className="thread-message" key={`${message.role}-${index}`}>
                    <b>{message.role}</b>
                    <span>{message.body}</span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="thread-empty">Empty: select or create a thread to show conversation state.</p>
            )}
          </section>

          <section className="thread-card">
            <h3>Status controls</h3>
            <div className="thread-statuses">
              {statuses.map((status) => (
                <button key={status} onClick={() => onSetStatus(status)} type="button">
                  <StatusPill status={status} />
                </button>
              ))}
            </div>
            <div className="thread-actions">
              <button onClick={onArchiveActive} type="button">Archive active</button>
            </div>
            <p>Idle, active, blocked, failed, done, and empty states are visible before runtime work begins.</p>
          </section>
        </div>
      </section>
    </div>
  );
}

function StateNotice({ state }: { state: ThreadUiState }) {
  const messages: Record<ThreadUiState, string> = {
    empty: "Empty: no active threads are linked to this project.",
    error: "Error: thread goal is required before creation.",
    ready: "Ready: thread state is project-linked and local.",
  };

  return <p className={`thread-state thread-${state}`}>{messages[state]}</p>;
}

function StatusPill({ status }: { status: ThreadStatus }) {
  return <span className={`thread-pill ${status}`}>{status}</span>;
}
