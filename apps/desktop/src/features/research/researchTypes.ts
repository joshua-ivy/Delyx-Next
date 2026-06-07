import type { EvidenceRecord } from "../runs/agentRunTypes";

export type { EvidenceRecord, EvidenceSourceKind } from "../runs/agentRunTypes";

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

export interface EvidenceReceiptView extends EvidenceRecord {
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
