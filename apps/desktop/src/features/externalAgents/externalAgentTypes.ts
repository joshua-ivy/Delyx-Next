export type ExternalAgentAdapterStatus = "available" | "missing" | "not_checked";
export type ExternalAgentContractStatus = "draft" | "approval_required" | "expired";
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
export type ExternalAgentPermissionMode = "read_only" | "workspace_write";

export interface ExternalAgentStateView {
  adapters: ExternalAgentAdapterView[];
  contracts: ExternalAgentCommandContractView[];
  artifacts: ExternalAgentRunArtifactView[];
}

export interface ExternalAgentAdapterView {
  id: string;
  label: string;
  status: ExternalAgentAdapterStatus;
  detail: string;
}

export interface ExternalAgentCommandContractView {
  id: string;
  runId: string;
  adapterId: string;
  status: ExternalAgentContractStatus;
  permissionMode: ExternalAgentPermissionMode;
  program: string;
  args: string[];
  workingDirectory: string;
  transcriptFormat: string;
  requiredDelyxTools: string[];
  safetySummary: string;
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
