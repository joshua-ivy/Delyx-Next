import type { MobileApprovalView, MobileRunView, MobileStateView, MobileThreadView } from "../features/mobile/mobileTypes";
import { escapeHtml } from "./html";

export function emptyMobileBlock() {
  return `<div class="dfile mobile-review">
        <div class="dh"><span class="fn">Mobile companion</span><span class="dst">not paired</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No mobile companion paired. Mobile cannot access files or terminal by default.</span></div>
        </div>
      </div>`;
}

export function mobileBlock(state: MobileStateView) {
  if (
    !state.paired
    && state.threads.length === 0
    && state.pendingApprovals.length === 0
    && state.runs.length === 0
  ) {
    return emptyMobileBlock();
  }

  return `<div class="dfile mobile-review">
        <div class="dh"><span class="fn">Mobile companion</span><span class="dst">${state.paired ? "paired" : "not paired"}</span></div>
        <div class="dc">
          ${policyLine(state)}
          ${state.threads.map(threadLine).join("")}
          ${state.pendingApprovals.map(approvalLine).join("")}
          ${state.runs.map(runLine).join("")}
        </div>
      </div>`;
}

export function hasMobileActivity(state: MobileStateView) {
  return state.paired
    || state.threads.length > 0
    || state.pendingApprovals.length > 0
    || state.runs.length > 0;
}

function policyLine(state: MobileStateView) {
  const approval = state.policy.allowLowRiskApproval ? "low-risk approvals enabled" : "approvals disabled";
  const files = state.policy.canAccessFiles ? "file access allowed" : "no broad file access";
  const terminal = state.policy.canAccessTerminal ? "terminal access allowed" : "no broad terminal access";
  return `<div class="dr"><span class="g">policy</span><span class="x">${approval} &middot; max ${escapeHtml(state.policy.maxApprovalRisk)} &middot; ${files} &middot; ${terminal}</span></div>`;
}

function threadLine(thread: MobileThreadView) {
  return `<div class="dr"><span class="g">thread</span><span class="x">${escapeHtml(thread.title)} &middot; ${escapeHtml(thread.status)} &middot; ${escapeHtml(thread.id)}</span></div>`;
}

function approvalLine(approval: MobileApprovalView) {
  return `<div class="dr"><span class="g">scope</span><span class="x">${escapeHtml(approval.id)} &middot; ${escapeHtml(approval.risk)} &middot; ${escapeHtml(approval.scope)} &middot; expires ${escapeHtml(approval.expiresAt)}</span></div>`;
}

function runLine(run: MobileRunView) {
  return `<div class="dr"><span class="g">run</span><span class="x">${escapeHtml(run.id)} &middot; ${escapeHtml(run.status)} &middot; ${escapeHtml(run.latestEvent)}</span></div>`;
}
