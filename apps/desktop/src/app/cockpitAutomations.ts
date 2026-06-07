import type { AutomationStateView, MissionContractView, ScheduledRunView } from "../features/automations/automationTypes";
import { escapeHtml } from "./html";

export function emptyAutomationBlock() {
  return `<div class="dfile automation-review">
        <div class="dh"><span class="fn">Automations</span><span class="dst">paused</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No automation mission contracts. Recurring work starts paused until approved.</span></div>
        </div>
      </div>`;
}

export function automationBlock(state: AutomationStateView) {
  if (state.contracts.length === 0 && state.scheduledRuns.length === 0) {
    return emptyAutomationBlock();
  }

  return `<div class="dfile automation-review">
        <div class="dh"><span class="fn">Automations</span><span class="dst">${state.contracts.length} contract(s)</span></div>
        <div class="dc">
          ${state.contracts.map(contractLine).join("")}
          ${state.scheduledRuns.map(runLine).join("")}
        </div>
      </div>`;
}

function contractLine(contract: MissionContractView) {
  return `<div class="dr ${contract.status === "blocked" ? "m" : ""}"><span class="g">job</span><span class="x">${escapeHtml(contract.title)} &middot; ${escapeHtml(contract.status)} &middot; tools ${escapeHtml(contract.allowedTools.join(", "))} &middot; ${escapeHtml(contract.activeHours)} ${escapeHtml(contract.timezone)} &middot; ${escapeHtml(contract.scope)} &middot; stops: ${escapeHtml(contract.stopCondition)}</span></div>`;
}

function runLine(run: ScheduledRunView) {
  const approval = run.approvalId ? ` &middot; approval ${escapeHtml(run.approvalId)}` : "";
  return `<div class="dr ${run.status === "blocked" ? "m" : ""}"><span class="g">run</span><span class="x">${escapeHtml(run.contractId)} &middot; ${escapeHtml(run.status)} &middot; ${escapeHtml(run.reason)}${approval}</span></div>`;
}
