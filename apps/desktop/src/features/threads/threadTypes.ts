export type ThreadStatus =
  | "idle"
  | "exploring"
  | "planning"
  | "waiting_for_approval"
  | "building"
  | "testing"
  | "reviewing"
  | "blocked"
  | "failed"
  | "done";
export type ThreadMode = "explore" | "plan" | "build" | "review" | "test" | "research";

export const threadStatuses: ThreadStatus[] = [
  "idle",
  "exploring",
  "planning",
  "waiting_for_approval",
  "building",
  "testing",
  "reviewing",
  "blocked",
  "failed",
  "done",
];

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
  mode: ThreadMode;
  activeRunId?: string;
  runIds: string[];
  createdAt: string;
  updatedAt: string;
  createdLabel: string;
  messages: ThreadMessage[];
  archived: boolean;
}
