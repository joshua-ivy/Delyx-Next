export type EvidenceSourceKind = "local_file" | "repo_symbol" | "diff" | "test" | "terminal" | "external_agent" | "web" | "memory" | "model_call";
export type EvidenceStance = "supports" | "contradicts";
export type ClaimStatus = "supported" | "insufficient_evidence" | "contradicted";

export interface ResearchAnswerView {
  runId: string;
  question: string;
  summary: string;
  receipts: EvidenceReceiptView[];
  audits: ClaimAuditView[];
  contradictions: ContradictionView[];
}

export interface EvidenceReceiptView {
  id: string;
  runId: string;
  sourceKind: EvidenceSourceKind;
  title: string;
  locator: string;
  excerpt: string;
  stance: EvidenceStance;
  claimKey: string;
}

export interface ClaimAuditView {
  id: string;
  text: string;
  status: ClaimStatus;
  requiresSupport: boolean;
  evidenceIds: string[];
}

export interface ContradictionView {
  claimId: string;
  supportingEvidenceId: string;
  contradictingEvidenceId: string;
  message: string;
}
