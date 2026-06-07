import { escapeHtml } from "./html";
import type { AgentRunStatus, AgentRunView } from "../features/runs/agentRunTypes";

export function emptyTimelineBlock() {
  return '<div class="tnode pending"><div class="tr"><span class="kd">empty</span><span class="ms">No AgentRun events have been recorded for this thread.</span><span class="ts">-</span></div></div>';
}

export function runTimeline(run: AgentRunView | undefined) {
  if (!run || run.events.length === 0) {
    return emptyTimelineBlock();
  }

  return run.events.map((event) => (
    `<div class="tnode ${eventClass(run.status)}"><div class="tr"><span class="kd">${escapeHtml(event.kind)}</span><span class="ms">${escapeHtml(event.message)}</span><span class="ts">${escapeHtml(event.createdAt)}</span></div></div>`
  )).join("");
}

export function runStatusPill(run: AgentRunView | undefined) {
  if (!run) {
    return '<span class="pill ghost">No active run</span>';
  }

  return `<span class="pill ${statusClass(run.status)}"><span class="dot"></span>${statusLabel(run.status)}</span>`;
}

export function runLabel(run: AgentRunView | undefined) {
  return run ? ` <span class="pill build micro">${escapeHtml(run.id)}</span>` : "";
}

function eventClass(status: AgentRunStatus) {
  if (status === "succeeded") {
    return "done";
  }
  if (status === "failed" || status === "cancelled" || status === "blocked") {
    return "blocked";
  }
  return "pending";
}

function statusClass(status: AgentRunStatus) {
  const classes: Record<AgentRunStatus, string> = {
    blocked: "blocked",
    cancelled: "ghost",
    created: "ghost",
    failed: "failed",
    repairing: "wait",
    running: "wait",
    succeeded: "done",
    waiting_for_approval: "blocked",
  };
  return classes[status];
}

function statusLabel(status: AgentRunStatus) {
  const labels: Record<AgentRunStatus, string> = {
    blocked: "Blocked",
    cancelled: "Cancelled",
    created: "Created",
    failed: "Failed",
    repairing: "Repairing",
    running: "Running",
    succeeded: "Succeeded",
    waiting_for_approval: "Waiting",
  };
  return labels[status];
}
