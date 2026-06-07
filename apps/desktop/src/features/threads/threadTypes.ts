export type ThreadStatus = "idle" | "active" | "blocked" | "failed" | "done";

export type ThreadUiState = "ready" | "empty" | "error";

export interface ThreadMessage {
  role: "user" | "assistant" | "system";
  body: string;
}

export interface TaskThread {
  id: string;
  projectId: string;
  title: string;
  goal: string;
  status: ThreadStatus;
  createdLabel: string;
  messages: ThreadMessage[];
  archived: boolean;
}
