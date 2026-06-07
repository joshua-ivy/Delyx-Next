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

export interface RiskTaxonomyEntryView {
  minimumRisk: RiskLevel;
  summary: string;
  rollbackRequired: boolean;
}

export const riskTaxonomy: Record<RiskyAction, RiskTaxonomyEntryView> = {
  file_write: {
    minimumRisk: "high",
    rollbackRequired: true,
    summary: "file writes require checkpoint scope",
  },
  terminal_command: {
    minimumRisk: "medium",
    rollbackRequired: false,
    summary: "terminal commands require captured artifacts",
  },
  dependency_install: {
    minimumRisk: "high",
    rollbackRequired: true,
    summary: "dependency installs mutate the project",
  },
  connector_write: {
    minimumRisk: "high",
    rollbackRequired: true,
    summary: "connector writes leave the local trust boundary",
  },
  durable_memory_save: {
    minimumRisk: "medium",
    rollbackRequired: true,
    summary: "durable memory changes future runs",
  },
  scheduled_risky_action: {
    minimumRisk: "dangerous",
    rollbackRequired: true,
    summary: "scheduled risky actions can run later without attention",
  },
  external_agent_execution: {
    minimumRisk: "high",
    rollbackRequired: true,
    summary: "external agents run inside bounded scope only",
  },
  external_send: {
    minimumRisk: "high",
    rollbackRequired: false,
    summary: "external sends disclose data outside the workspace",
  },
};

export function riskPolicyLabel(action: RiskyAction) {
  const entry = riskTaxonomy[action];
  const rollback = entry.rollbackRequired ? "rollback required" : "rollback optional";
  return `${entry.minimumRisk} minimum; ${rollback}; ${entry.summary}`;
}

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
