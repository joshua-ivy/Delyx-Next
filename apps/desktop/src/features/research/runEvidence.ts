import type { AgentRunView } from "../runs/agentRunTypes";
import type { EvidenceReceiptView, ResearchAnswerView } from "./researchTypes";

export function researchAnswerFromRunEvidence(run: AgentRunView | undefined): ResearchAnswerView | undefined {
  if (!run || run.evidence.length === 0) {
    return undefined;
  }
  return {
    audits: [],
    contradictions: [],
    question: "What evidence has this run captured?",
    receipts: run.evidence.map(receiptFromRunEvidence),
    runId: run.id,
    summary: "Evidence records were captured for this run. No source-backed claims have been audited yet.",
  };
}

function receiptFromRunEvidence(receipt: AgentRunView["evidence"][number]): EvidenceReceiptView {
  return {
    ...receipt,
    claimKey: "not_audited",
    stance: "recorded",
  };
}
