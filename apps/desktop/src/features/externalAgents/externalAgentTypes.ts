export type ExternalAgentAdapterStatus = "available" | "missing" | "not_checked";
export type ExternalAgentEventKind =
  | "command"
  | "completed"
  | "started"
  | "stderr"
  | "stdout"
  | "file_changed"
  | "diff_captured"
  | "failed"
  | "test_result"
  | "review_decision";
export type ExternalAgentRunStatus = "accepted" | "completed" | "blocked" | "failed" | "reverted";

export interface ExternalAgentStateView {
  adapters: ExternalAgentAdapterView[];
  artifacts: ExternalAgentRunArtifactView[];
}

export interface ExternalAgentAdapterView {
  id: string;
  label: string;
  status: ExternalAgentAdapterStatus;
  detail: string;
}

export interface ExternalAgentRunArtifactView {
  id: string;
  runId: string;
  adapterId: string;
  status: ExternalAgentRunStatus;
  scope: string;
  transcript: ExternalAgentEventView[];
  terminalOutput: string;
  diffSummary?: string;
  testArtifactIds: string[];
  reviewRequired: boolean;
}

export interface ExternalAgentEventView {
  kind: ExternalAgentEventKind;
  message: string;
  timestamp: string;
}
