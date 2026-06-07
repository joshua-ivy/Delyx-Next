import type { ClaimAuditView, EvidenceReceiptView, ResearchAnswerView } from "../features/research/researchTypes";
import { escapeHtml } from "./html";

export function emptyEvidenceBlock() {
  return `<div class="sec-h" style="margin:16px 0 6px;"><h4 style="font-size:12px;">Evidence &middot; 0 records</h4><span class="ln"></span></div>
      <div class="rcpt"><span class="ri">i</span><div><div class="rn">No evidence records</div><div class="rd">Claims stay unsupported until a real EvidenceRecord is created.</div></div></div>`;
}

export function evidenceBlock(answer: ResearchAnswerView | undefined) {
  if (!answer) {
    return emptyEvidenceBlock();
  }

  return `<div class="sec-h" style="margin:16px 0 6px;"><h4 style="font-size:12px;">Evidence &middot; ${answer.receipts.length} records</h4><span class="ln"></span></div>
      <div class="rcpt"><span class="ri">?</span><div><div class="rn">${escapeHtml(answer.summary)}</div><div class="rd">${escapeHtml(answer.question)}</div></div></div>
      ${answer.receipts.map(receiptBlock).join("")}
      ${answer.audits.map(auditBlock).join("")}
      ${answer.contradictions.map((item) => `<div class="rcpt"><span class="ri">!</span><div><div class="rn">Conflicting evidence</div><div class="rd">${escapeHtml(item.message)}</div></div></div>`).join("")}`;
}

function receiptBlock(receipt: EvidenceReceiptView) {
  const stance = receipt.stance === "supports" ? "supports" : "contradicts";
  return `<div class="rcpt">
        <span class="ri">${escapeHtml(sourceLabel(receipt.sourceKind))}</span>
        <div><div class="rn">${escapeHtml(receipt.title)} <span class="pill ghost" style="font-size:10px;">${stance}</span></div>
        <div class="rd">${escapeHtml(receipt.locator)}</div><div class="rd">${escapeHtml(receipt.excerpt)}</div></div>
      </div>`;
}

function auditBlock(audit: ClaimAuditView) {
  const klass = audit.status === "supported" ? "done" : audit.status === "contradicted" ? "blocked" : "ghost";
  const required = audit.requiresSupport ? "requires support" : "support optional";
  return `<div class="rcpt"><span class="ri">C</span><div><div class="rn">${escapeHtml(audit.text)} <span class="pill ${klass}" style="font-size:10px;">${escapeHtml(audit.status)}</span></div><div class="rd">${required} &middot; ${audit.evidenceIds.length} receipt(s)</div></div></div>`;
}

function sourceLabel(sourceKind: EvidenceReceiptView["sourceKind"]) {
  const labels: Record<EvidenceReceiptView["sourceKind"], string> = {
    diff: "D",
    external_agent: "X",
    local_file: "L",
    memory: "M",
    model_call: "A",
    repo_symbol: "R",
    terminal: "$",
    test: "T",
    web: "W",
  };
  return labels[sourceKind];
}
