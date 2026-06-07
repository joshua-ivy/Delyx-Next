import type { RiskLevel } from "../approvals/approvalTypes";

export interface MobileStateView {
  paired: boolean;
  policy: MobilePolicyView;
  threads: MobileThreadView[];
  pendingApprovals: MobileApprovalView[];
  runs: MobileRunView[];
}

export interface MobilePolicyView {
  allowLowRiskApproval: boolean;
  maxApprovalRisk: RiskLevel;
  canAccessFiles: boolean;
  canAccessTerminal: boolean;
}

export interface MobileThreadView {
  id: string;
  title: string;
  status: string;
}

export interface MobileApprovalView {
  id: string;
  scope: string;
  risk: RiskLevel;
  expiresAt: string;
}

export interface MobileRunView {
  id: string;
  status: string;
  latestEvent: string;
}
