export type RiskyAction =
  | "read_file"
  | "write_file"
  | "edit_file"
  | "run_terminal"
  | "install_dependency"
  | "save_memory"
  | "use_connector"
  | "schedule_work"
  | "external_agent"
  | "external_send";

export type RiskLevel = "low" | "medium" | "high" | "dangerous";
export type ProposalStatus = "pending" | "approved" | "denied" | "expired";
export type PermissionScopeKind =
  | "workspace"
  | "file"
  | "terminal"
  | "dependency"
  | "connector"
  | "memory"
  | "automation"
  | "external_agent";

export interface PermissionScope {
  kind: PermissionScopeKind;
  summary: string;
  projectId?: string;
  root?: string;
  paths?: string[];
  commands?: string[];
  connectorId?: string;
}

export interface RiskTaxonomyEntryView {
  minimumRisk: RiskLevel;
  summary: string;
  rollbackRequired: boolean;
}

export interface RiskTaxonomyBridgeEntryView extends RiskTaxonomyEntryView {
  actionType: RiskyAction;
}

export type RiskTaxonomySnapshotView = Partial<Record<RiskyAction, RiskTaxonomyEntryView>>;

export const riskTaxonomy: Record<RiskyAction, RiskTaxonomyEntryView> = {
  read_file: {
    minimumRisk: "low",
    rollbackRequired: false,
    summary: "file reads must stay inside approved roots",
  },
  write_file: {
    minimumRisk: "high",
    rollbackRequired: true,
    summary: "file writes require checkpoint scope",
  },
  edit_file: {
    minimumRisk: "high",
    rollbackRequired: true,
    summary: "file edits require checkpoint scope",
  },
  run_terminal: {
    minimumRisk: "medium",
    rollbackRequired: false,
    summary: "terminal commands require captured artifacts",
  },
  install_dependency: {
    minimumRisk: "high",
    rollbackRequired: true,
    summary: "dependency installs mutate the project",
  },
  use_connector: {
    minimumRisk: "high",
    rollbackRequired: true,
    summary: "connector writes leave the local trust boundary",
  },
  save_memory: {
    minimumRisk: "medium",
    rollbackRequired: true,
    summary: "durable memory changes future runs",
  },
  schedule_work: {
    minimumRisk: "dangerous",
    rollbackRequired: true,
    summary: "scheduled risky actions can run later without attention",
  },
  external_agent: {
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

export function mergeRiskTaxonomySnapshot(entries: RiskTaxonomyBridgeEntryView[]): RiskTaxonomySnapshotView {
  return entries.reduce<RiskTaxonomySnapshotView>(
    (policy, entry) => ({ ...policy, [entry.actionType]: {
      minimumRisk: entry.minimumRisk,
      rollbackRequired: entry.rollbackRequired,
      summary: entry.summary,
    } }),
    riskTaxonomy,
  );
}

export function riskPolicyLabel(action: RiskyAction, policy: RiskTaxonomySnapshotView = riskTaxonomy) {
  const entry = policy[action] ?? riskTaxonomy[action];
  const rollback = entry.rollbackRequired ? "rollback required" : "rollback optional";
  return `${entry.minimumRisk} minimum; ${rollback}; ${entry.summary}`;
}

export function scopeLabel(scope: PermissionScope) {
  return [scope.summary, scope.root, scope.connectorId].filter(Boolean).join(" - ") || scope.kind;
}

export function scopeArtifactLabel(scope: PermissionScope) {
  const scopedItems = [...(scope.paths ?? []), ...(scope.commands ?? [])];
  return scopedItems.length > 0 ? scopedItems.join(", ") : "No file or command scope listed.";
}

export interface ActionProposal {
  id: string;
  runId: string;
  nodeId: string;
  actionType: RiskyAction;
  riskLabel: RiskLevel;
  requiredPermission: string;
  rationale: string;
  expectedResult: string;
  rollbackPlan?: string;
  scope: PermissionScope;
  expiresAt: string;
  status: ProposalStatus;
}

export type ActionProposalView = ActionProposal;
