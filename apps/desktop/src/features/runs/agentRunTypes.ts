import type { ThreadMode } from "../threads/threadTypes";

export type AgentRunMode = ThreadMode | "automation";
export type AgentRunStatus =
  | "created"
  | "running"
  | "waiting_for_approval"
  | "blocked"
  | "repairing"
  | "succeeded"
  | "failed"
  | "cancelled";
export type AgentNodeKind =
  | "classify"
  | "explore"
  | "plan"
  | "model_call"
  | "tool_proposal"
  | "wait_for_approval"
  | "tool_execution"
  | "patch_proposal"
  | "diff_review"
  | "test_execution"
  | "verify"
  | "repair"
  | "answer"
  | "memory_candidate"
  | "external_agent"
  | "done"
  | "blocked";
export type AgentNodeStatus = "pending" | "running" | "waiting" | "succeeded" | "failed" | "skipped";
export type EvidenceSourceKind =
  | "local_file"
  | "repo_symbol"
  | "terminal"
  | "test"
  | "diff"
  | "web"
  | "memory"
  | "external_agent"
  | "model_call";

export interface AgentRun {
  id: string;
  projectId?: string;
  threadId?: string;
  parentRunId?: string;
  goal: string;
  mode: AgentRunMode;
  status: AgentRunStatus;
  nodes: AgentNode[];
  events: AgentEvent[];
  artifacts: Artifact[];
  evidence: EvidenceRecord[];
  metrics: RunMetrics;
  outcome?: AgentOutcome;
  createdAt: string;
  updatedAt: string;
}

export interface AgentNode {
  id: string;
  runId: string;
  kind: AgentNodeKind;
  status: AgentNodeStatus;
  dependsOn: string[];
  input: unknown;
  output?: unknown;
  error?: string;
  startedAt?: string;
  completedAt?: string;
}

export interface AgentEvent {
  id: string;
  runId: string;
  nodeId?: string;
  kind: string;
  message: string;
  createdAt: string;
  payload?: unknown;
}

export interface Artifact {
  id: string;
  runId: string;
  kind: string;
  title: string;
  uri?: string;
  createdAt: string;
  metadata?: Record<string, unknown>;
}

export interface EvidenceRecord {
  id: string;
  runId: string;
  sourceKind: EvidenceSourceKind;
  sourceId: string;
  title?: string;
  uri?: string;
  quote?: string;
  hash?: string;
  retrievedAt: string;
  relevance?: EvidenceRelevance;
}

export interface EvidenceRelevance {
  relationship:
    | "direct_implementation"
    | "caller"
    | "test"
    | "config"
    | "doc"
    | "name_only"
    | "unknown";
  score: number;
  reason: string;
}

export interface RunMetrics {
  eventCount: number;
  nodeCount: number;
  artifactCount: number;
  evidenceCount: number;
  commandCount: number;
  approvalCount: number;
  durationMs?: number;
  tokenCount?: number;
}

export interface AgentOutcome {
  status: "succeeded" | "failed" | "cancelled" | "blocked";
  summary: string;
  evidenceRecordIds: string[];
  testArtifactIds: string[];
}

export type AgentRunView = AgentRun;
export type AgentRunEventView = AgentEvent;
