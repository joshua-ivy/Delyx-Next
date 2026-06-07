export type AgentRunStatus = "running" | "waiting_for_approval" | "completed" | "failed";

export interface AgentRunView {
  id: string;
  threadId: string;
  status: AgentRunStatus;
  events: AgentRunEventView[];
  eventCount: number;
  artifactCount: number;
  evidenceCount: number;
  outcome?: string;
}

export interface AgentRunEventView {
  id: string;
  kind: string;
  message: string;
}
