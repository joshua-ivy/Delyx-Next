export type RiskyAction =
  | "file_write"
  | "terminal_command"
  | "dependency_install"
  | "connector_write"
  | "durable_memory_save"
  | "scheduled_risky_action"
  | "external_agent_execution"
  | "external_send";

export type RiskLevel = "low" | "medium" | "high" | "dangerous";
export type ProposalStatus = "pending" | "approved" | "denied" | "expired";

export interface ActionProposalView {
  id: string;
  runId: string;
  nodeId: string;
  action: RiskyAction;
  risk: RiskLevel;
  scope: string;
  reason: string;
  expectedResult: string;
  rollbackPlan: string;
  expiresAt: string;
  status: ProposalStatus;
}
