export type ReviewMode = "read_only";
export type ReviewDecision = "pending" | "revise_requested" | "accepted" | "revert_requested";
export type ReviewPriority = "p0" | "p1" | "p2" | "p3";

export interface ReviewReportView {
  id: string;
  runId: string;
  mode: ReviewMode;
  decision: ReviewDecision;
  findings: ReviewFindingView[];
  riskSummary: string;
  testSummary: string;
  evidenceSummary: string;
}

export interface ReviewFindingView {
  id: string;
  priority: ReviewPriority;
  title: string;
  detail: string;
  riskLabel: string;
  suggestedFix: string;
  filePath: string;
  hunkLabel: string;
}
