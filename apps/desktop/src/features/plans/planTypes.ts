export type PlanDecision = "pending" | "approved" | "revision_requested" | "cancelled";

export interface ExploreView {
  relevantFiles: string[];
  relevantSymbols: string[];
  architectureSummary: string;
  projectCommands: string[];
  risks: string[];
  unknowns: string[];
  suggestedNextSteps: string[];
}

export interface PlanView {
  threadId: string;
  goalUnderstanding: string;
  filesLikelyInvolved: string[];
  steps: string[];
  risks: string[];
  testsToRun: string[];
  permissionsNeeded: string[];
  rollbackStrategy: string;
  decision: PlanDecision;
  explore: ExploreView;
}
