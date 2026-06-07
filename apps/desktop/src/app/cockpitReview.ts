import {
  riskPolicyLabel,
  scopeArtifactLabel,
  scopeLabel,
  type ActionProposalView,
  type ProposalStatus,
} from "../features/approvals/approvalTypes";
import type { DiffLineView, PatchProposalView } from "../features/patches/patchTypes";
import type { ReviewFindingView, ReviewReportView } from "../features/review/reviewTypes";
import type { TestArtifactView, TestStatus } from "../features/tests/testTypes";
import { escapeHtml } from "./html";

export function emptyApprovalBlock() {
  return `<div class="appro">
        <div class="at"><span class="pill ghost">No proposal</span><span class="meta-id">none</span></div>
        <h4>No approval requests</h4>
        <div class="kv"><span class="k">Scope</span><span class="v">No file writes, commands, connectors, memory saves, or external agents requested.</span></div>
        <div class="kv"><span class="k">Risk</span><span class="v">No risky action pending.</span></div>
        <div class="kv"><span class="k">Policy</span><span class="v">Risk taxonomy active; risky actions keep their minimum risk floor.</span></div>
        <div class="kv"><span class="k">Rollback</span><span class="v">No checkpoint exists yet.</span></div>
      </div>`;
}

export function approvalBlock(proposals: ActionProposalView[]) {
  if (proposals.length === 0) {
    return emptyApprovalBlock();
  }

  return proposals.map((proposal) => {
    const status = effectiveProposalStatus(proposal);
    return `<div class="appro">
        <div class="at"><span class="pill ${riskClass(proposal.riskLabel)}">${escapeHtml(proposal.riskLabel)} risk &middot; ${escapeHtml(proposal.actionType)}</span><span class="meta-id">${escapeHtml(proposal.id)}</span></div>
        <h4>${escapeHtml(proposal.actionType)}</h4>
        <div class="kv"><span class="k">Scope</span><span class="v">${escapeHtml(scopeLabel(proposal.scope))}</span></div>
        <div class="kv"><span class="k">Files/commands</span><span class="v">${escapeHtml(scopeArtifactLabel(proposal.scope))}</span></div>
        <div class="kv"><span class="k">Permission</span><span class="v">${escapeHtml(proposal.requiredPermission)}</span></div>
        <div class="kv"><span class="k">Reason</span><span class="v">${escapeHtml(proposal.rationale)}</span></div>
        <div class="kv"><span class="k">Expected</span><span class="v">${escapeHtml(proposal.expectedResult)}</span></div>
        <div class="kv"><span class="k">Policy</span><span class="v">${escapeHtml(riskPolicyLabel(proposal.actionType))}</span></div>
        <div class="kv"><span class="k">Rollback</span><span class="v">${escapeHtml(proposal.rollbackPlan ?? "No rollback plan recorded.")}</span></div>
        <div class="exp">Run ${escapeHtml(proposal.runId)} &middot; Node ${escapeHtml(proposal.nodeId)} &middot; Expires ${escapeHtml(proposal.expiresAt)} &middot; ${escapeHtml(status)}</div>
        ${approvalActions(proposal, status)}
      </div>`;
  }).join("");
}

export function pendingCount(proposals: ActionProposalView[]) {
  return proposals.filter((proposal) => effectiveProposalStatus(proposal) === "pending").length;
}

export function emptyDiffBlock() {
  return `<div class="dfile">
        <div class="dh"><span class="fn">Unified diff artifact</span><span class="dst">empty</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No patch or file change has been proposed.</span></div>
        </div>
        <div class="diff-actions diff-empty-actions"><span class="pill ghost micro">No patch actions available until a PatchProposal exists.</span></div>
      </div>`;
}

export function diffBlock(patches: PatchProposalView[]) {
  const files = patches.flatMap((patch) => patch.files.map((file) => ({ file, patch })));
  if (files.length === 0) {
    return emptyDiffBlock();
  }

  const changedFiles = `<div class="dfile diff-summary">
        <div class="dh"><span class="fn">Changed files</span><span class="dst">${files.length} file(s)</span></div>
        <div class="dc">${files.map(({ file }) => `<div class="dr"><span class="g">file</span><span class="x">${escapeHtml(file.path)}</span></div>`).join("")}</div>
        ${diffActionsBlock()}
      </div>`;
  return `${changedFiles}${files.map(({ file, patch }) => `<div class="dfile">
        <div class="dh"><span class="fn">${escapeHtml(file.path)}</span><span class="dst">${patchSummary(file.diff)} &middot; ${escapeHtml(patch.status)}</span></div>
        <div class="dc">${file.diff.map(diffLine).join("")}</div>
      </div>`).join("")}`;
}

export function patchSummary(lines: DiffLineView[]) {
  const added = lines.filter((line) => line.kind === "added").length;
  const removed = lines.filter((line) => line.kind === "removed").length;
  return `<span class="p">+${added}</span> <span class="m">-${removed}</span>`;
}

function diffActionsBlock() {
  return `<div class="diff-actions"><span class="btn diff-approve">Approve apply</span><span class="btn diff-reject">Reject</span><span class="btn diff-revert">Revert checkpoint</span><span class="btn diff-revise">Ask revision</span></div>`;
}

export function emptyTestBlock() {
  return `<div class="dfile test-artifact">
        <div class="dh"><span class="fn">Test artifact</span><span class="dst">not run</span></div>
        <div class="dc">
          <div class="dr"><span class="g">$</span><span class="x">No test command artifact has been captured.</span></div>
        </div>
      </div>`;
}

export function testBlock(artifacts: TestArtifactView[]) {
  if (artifacts.length === 0) {
    return emptyTestBlock();
  }

  return artifacts.map((artifact) => `<div class="dfile test-artifact">
        <div class="dh"><span class="fn">${escapeHtml(artifact.command)}</span><span class="dst">${testStatus(artifact)} &middot; ${artifact.durationMs}ms</span></div>
        <div class="dc">
          <div class="dr"><span class="g">artifact</span><span class="x">${escapeHtml(artifact.id)}</span></div>
          <div class="dr"><span class="g">run</span><span class="x">${escapeHtml(artifact.runId)}</span></div>
          <div class="dr"><span class="g">approval</span><span class="x">${escapeHtml(artifact.approvalId ?? "No approval linked")}</span></div>
          <div class="dr"><span class="g">started</span><span class="x">${escapeHtml(artifact.startedAt)}</span></div>
          <div class="dr"><span class="g">completed</span><span class="x">${escapeHtml(artifact.completedAt)}</span></div>
          <div class="dr"><span class="g">cwd</span><span class="x">${escapeHtml(artifact.cwd)}</span></div>
          ${failureLine(artifact)}
          ${streamLines("out", artifact.stdout, "")}
          ${streamLines("err", artifact.stderr, statusForArtifact(artifact) === "failed" ? "m" : "")}
        </div>
      </div>`).join("");
}

export function testStat(artifacts: TestArtifactView[]) {
  const latest = artifacts[0];
  if (!latest) {
    return "Not run";
  }
  const status = statusForArtifact(latest);
  return status === "passed" ? "Passed" : status === "failed" ? "Failed" : "Not run";
}

export function emptyReviewBlock() {
  return `<div class="sec-h review-findings compact"><h4>Review &middot; read-only</h4><span class="ln"></span></div>
      <div class="rcpt review-finding"><span class="ri">R</span><div><div class="rn">No review findings</div><div class="rd">Review mode does not edit. Findings appear only after a real ReviewReport is created.</div></div></div>`;
}

export function reviewBlock(report: ReviewReportView | undefined) {
  if (!report) {
    return emptyReviewBlock();
  }

  const findings = report.findings.length > 0
    ? report.findings.map(findingBlock).join("")
    : '<div class="rcpt review-finding"><span class="ri">R</span><div><div class="rn">No review findings</div><div class="rd">No prioritized issues were recorded in this ReviewReport.</div></div></div>';
  return `<div class="sec-h review-findings compact"><h4>Review &middot; ${escapeHtml(report.mode)}</h4><span class="ln"></span><span class="pill ghost">${escapeHtml(report.decision)}</span></div>
      <div class="rcpt"><span class="ri">S</span><div><div class="rn">${escapeHtml(report.riskSummary)}</div><div class="rd">${escapeHtml(report.testSummary)} ${escapeHtml(report.evidenceSummary)}</div></div></div>
      ${findings}`;
}

function findingBlock(finding: ReviewFindingView) {
  return `<div class="rcpt review-finding">
        <span class="ri">${escapeHtml(finding.priority.toUpperCase())}</span>
        <div>
          <div class="rn">${escapeHtml(finding.title)} <span class="pill blocked micro">${escapeHtml(finding.riskLabel)}</span></div>
          <div class="rd">${escapeHtml(finding.filePath)} &middot; ${escapeHtml(finding.hunkLabel)}</div>
          <div class="rd">${escapeHtml(finding.detail)}</div>
          <div class="rd">Suggested fix: ${escapeHtml(finding.suggestedFix)}</div>
          <span class="btn review-revise" data-finding-id="${escapeHtml(finding.id)}">Ask revise</span>
        </div>
      </div>`;
}

function testStatus(artifact: TestArtifactView) {
  const label = statusForArtifact(artifact).replace("_", " ");
  return `${label} &middot; exit ${artifact.exitCode ?? "unknown"}`;
}

function failureLine(artifact: TestArtifactView) {
  const failure = artifact.failureSummary ?? artifact.parsedFailures?.[0]?.message;
  if (!failure) {
    return "";
  }
  return `<div class="dr m"><span class="g">fail</span><span class="x">${escapeHtml(failure)}</span></div>`;
}

function streamLines(label: string, text: string, klass: string) {
  const lines = text.split(/\r?\n/).filter(Boolean).slice(0, 4);
  if (lines.length === 0) {
    return "";
  }
  return lines.map((line) => `<div class="dr ${klass}"><span class="g">${label}</span><span class="x">${escapeHtml(line)}</span></div>`).join("");
}

function diffLine(line: DiffLineView) {
  const sign = line.kind === "added" ? "+" : line.kind === "removed" ? "-" : " ";
  const klass = line.kind === "added" ? "p" : line.kind === "removed" ? "m" : "";
  return `<div class="dr ${klass}"><span class="g">${sign}</span><span class="x">${escapeHtml(line.text)}</span></div>`;
}

function riskClass(risk: ActionProposalView["riskLabel"]) {
  return risk === "low" ? "ghost" : risk === "medium" ? "wait" : "blocked";
}

function approvalActions(proposal: ActionProposalView, status: ProposalStatus) {
  if (status === "expired") {
    const label = proposal.status === "approved" ? "expired: approval no longer executable" : "expired: request a fresh approval";
    return `<div class="approval-actions"><span class="pill blocked micro">${escapeHtml(label)}</span></div>`;
  }
  if (status !== "pending") {
    return `<div class="approval-actions"><span class="pill ghost micro">decision recorded: ${escapeHtml(status)}</span></div>`;
  }
  return `<div class="approval-actions"><span class="btn approval-approve-once" data-proposal-id="${escapeHtml(proposal.id)}">Approve once</span><span class="btn approval-deny" data-proposal-id="${escapeHtml(proposal.id)}">Deny</span>${disabledApprovalAction("Always allow later", "Persistent allow rules are not available yet.")}${disabledApprovalAction("Edit scope", "Scope editing requires a future scope editor.")}</div>`;
}

function disabledApprovalAction(label: string, reason: string) {
  return `<span aria-disabled="true" class="btn approval-unavailable" title="${escapeHtml(reason)}">${escapeHtml(label)}</span>`;
}

function effectiveProposalStatus(proposal: ActionProposalView): ProposalStatus {
  if (proposal.status === "denied" || proposal.status === "expired") {
    return proposal.status;
  }
  const expiresAt = Date.parse(proposal.expiresAt);
  return Number.isFinite(expiresAt) && expiresAt <= Date.now() ? "expired" : proposal.status;
}

function statusForArtifact(artifact: TestArtifactView): TestStatus {
  if (artifact.status) {
    return artifact.status;
  }
  if (artifact.exitCode === null) {
    return "not_run";
  }
  return artifact.exitCode === 0 ? "passed" : "failed";
}
