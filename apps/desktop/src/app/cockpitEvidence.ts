import type { ClaimAuditView, EvidenceReceiptView, ResearchAnswerView } from "../features/research/researchTypes";
import { escapeHtml } from "./html";

export function emptyEvidenceBlock() {
  return `<div class="sec-h compact"><h4>Evidence &middot; 0 records</h4><span class="ln"></span></div>
      ${evidenceCoverageBlock(undefined)}
      <div class="rcpt"><span class="ri">i</span><div><div class="rn">No evidence records</div><div class="rd">Claims stay unsupported until a real EvidenceRecord is created.</div></div></div>`;
}

export function evidenceBlock(answer: ResearchAnswerView | undefined) {
  if (!answer) {
    return emptyEvidenceBlock();
  }

  return `<div class="sec-h compact"><h4>Evidence &middot; ${answer.receipts.length} records</h4><span class="ln"></span></div>
      <div class="rcpt"><span class="ri">?</span><div><div class="rn">${escapeHtml(answer.summary)}</div><div class="rd">${escapeHtml(answer.question)}</div></div></div>
      ${evidenceCoverageBlock(answer)}
      ${answer.receipts.map(receiptBlock).join("")}
      ${answer.audits.map(auditBlock).join("")}
      ${answer.contradictions.map((item) => `<div class="rcpt"><span class="ri">!</span><div><div class="rn">Conflicting evidence</div><div class="rd">${escapeHtml(item.message)}</div></div></div>`).join("")}`;
}

function receiptBlock(receipt: EvidenceReceiptView) {
  const stance = receipt.stance === "supports" ? "supports" : "contradicts";
  return `<div class="rcpt">
        <span class="ri">${escapeHtml(sourceLabel(receipt.sourceKind))}</span>
        <div><div class="rn">${escapeHtml(receipt.title ?? receipt.sourceId)} <span class="pill ghost micro">${stance}</span></div>
        <div class="rd">${escapeHtml(receipt.uri ?? receipt.sourceId)}</div><div class="rd">${escapeHtml(evidenceQuote(receipt))}</div>
        ${relevanceLine(receipt)}</div>
      </div>`;
}

function auditBlock(audit: ClaimAuditView) {
  const klass = audit.status === "supported" ? "done" : audit.status === "contradicted" ? "blocked" : "ghost";
  const required = audit.requiresSupport ? "requires support" : "support optional";
  return `<div class="rcpt"><span class="ri">C</span><div><div class="rn">${escapeHtml(audit.text)} <span class="pill ${klass} micro">${escapeHtml(audit.status)}</span></div><div class="rd">${required} &middot; ${audit.evidenceIds.length} receipt(s)</div></div></div>`;
}

function evidenceCoverageBlock(answer: ResearchAnswerView | undefined) {
  const receipts = answer?.receipts ?? [];
  const audits = answer?.audits ?? [];
  const testCount = countKind(receipts, "test");
  const needsReview = audits.filter((audit) => audit.status !== "supported").length + (answer?.contradictions.length ?? 0);
  const rows: [string, string][] = [
    ["Why Delyx believes this", `${audits.filter((audit) => audit.status === "supported").length} supported claim(s)`],
    ["What Delyx changed", `${countKind(receipts, "diff")} diff receipt(s)`],
    ["What Delyx tested", `${testCount} test receipt(s)`],
    ["What Delyx did not test", testCount === 0 ? "not tested" : "test receipts present"],
    ["What still needs review", `${needsReview} item(s)`],
    ["Files read", `${countKind(receipts, "local_file")} file receipt(s)`],
    ["Symbols inspected", `${countKind(receipts, "repo_symbol")} symbol receipt(s)`],
    ["Commands run", `${countKind(receipts, "terminal")} terminal receipt(s)`],
    ["Diffs produced", `${countKind(receipts, "diff")} diff receipt(s)`],
    ["Sources cited", `${countKind(receipts, "web")} web receipt(s)`],
    ["Memory used", `${countKind(receipts, "memory")} memory receipt(s)`],
    ["External agent transcript", `${countKind(receipts, "external_agent")} external receipt(s)`],
    ["Model calls", `${countKind(receipts, "model_call")} model receipt(s)`],
    ["Approvals granted", "not recorded as EvidenceRecord yet"],
  ];
  return `<div class="evidence-coverage">${rows.map(coverageRow).join("")}</div>`;
}

function countKind(receipts: EvidenceReceiptView[], kind: EvidenceReceiptView["sourceKind"]) {
  return receipts.filter((receipt) => receipt.sourceKind === kind).length;
}

function coverageRow([label, value]: [string, string]) {
  return `<div><span>${escapeHtml(label)}</span><b>${escapeHtml(value)}</b></div>`;
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

function evidenceQuote(receipt: EvidenceReceiptView) {
  return receipt.quote ?? "No quoted evidence text captured.";
}

function relevanceLine(receipt: EvidenceReceiptView) {
  if (!receipt.relevance) {
    return "";
  }
  const detail = `${receipt.relevance.relationship} (${receipt.relevance.score}): ${receipt.relevance.reason}`;
  return `<div class="rd">${escapeHtml(detail)}</div>`;
}
