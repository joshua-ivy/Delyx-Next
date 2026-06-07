export type MissionStatus = "paused" | "active" | "blocked";
export type ScheduledRunStatus = "created" | "waiting_for_approval" | "blocked";

export interface AutomationStateView {
  contracts: MissionContractView[];
  scheduledRuns: ScheduledRunView[];
}

export interface MissionContractView {
  id: string;
  title: string;
  status: MissionStatus;
  scope: string;
  allowedTools: string[];
  activeHours: string;
  timezone: string;
  deliveryTargets: string[];
  stopCondition: string;
  workspaceFingerprint: string;
}

export interface ScheduledRunView {
  id: string;
  contractId: string;
  status: ScheduledRunStatus;
  reason: string;
  approvalId?: string;
}
