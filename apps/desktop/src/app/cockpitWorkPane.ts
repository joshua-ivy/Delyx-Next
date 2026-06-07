import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import { escapeHtml } from "./html";

export function buildProgressBlock(plan: PlanView | undefined, thread: TaskThread | undefined) {
  const steps = progressSteps(plan, thread);
  return `<div class="deck-progress">
    <span class="ey">Build progress</span>
    ${steps.map((step, index) => progressStep(step, index)).join("")}
  </div>`;
}

export function workDiffBlock(patches: PatchProposalView[]) {
  const files = patches.flatMap((patch) => patch.files.map((file) => ({ file, patch })));
  if (files.length === 0) {
    return `<div class="deck-files">
      <button class="deck-ftab on" type="button"><span class="ftag M">-</span><span class="mono deck-ftab-name">No patch</span><span class="deck-ftab-stat mono">empty</span></button>
    </div>
    <div class="deck-diff card">
      <div class="deck-diff-head mono"><span>Unified diff artifact</span><span class="deck-diff-stat">empty</span></div>
      <div class="deck-diff-body">
        <div class="dl ctx"><span class="ln">-</span><span class="tx">No patch or file change has been proposed.</span></div>
      </div>
    </div>`;
  }
  const tabs = files.map(({ file }, index) => diffTab(file.path, file.diff, index)).join("");
  const panels = files.map(({ file, patch }, index) => diffPanel(file, patch, index)).join("");
  return `<div class="deck-files">${tabs}</div><div class="deck-diff card">${panels}</div>`;
}

export function terminalLabel(run: AgentRunView | undefined) {
  if (!run) {
    return "No terminal command has run";
  }
  const commands = run.metrics.commandCount === 1 ? "1 command" : `${run.metrics.commandCount} commands`;
  return `${run.status.replaceAll("_", " ")} &middot; ${commands}`;
}

export function composerMode(mode: TaskThread["mode"] | undefined) {
  return escapeHtml(mode ? mode[0].toUpperCase() + mode.slice(1) : "Build");
}

function progressSteps(plan: PlanView | undefined, thread: TaskThread | undefined) {
  if (!thread) {
    return [
      { label: "Create a real thread", state: "now", tag: "waiting" },
      { label: "Create a plan", state: "" },
      { label: "Review approval, diff, and tests", state: "" },
    ];
  }
  if (!plan) {
    return [
      { label: "Thread created", state: "done" },
      { label: "Create a plan from this thread", state: "now", tag: "ready" },
      { label: "Approve risky actions before build", state: "" },
    ];
  }
  const approved = plan.decision === "approved";
  return plan.steps.slice(0, 5).map((label, index) => ({
    label,
    state: approved ? "done" : index === 0 ? "now" : "",
    tag: !approved && index === 0 ? plan.decision.replace("_", " ") : undefined,
  }));
}

function progressStep(step: { label: string; state: string; tag?: string }, index: number) {
  const number = step.state === "done" ? "&#10003;" : `${index + 1}`;
  const tag = step.tag ? `<span class="pill warning deck-pstep-tag">${escapeHtml(step.tag)}</span>` : "";
  return `<div class="deck-pstep ${step.state}"><span class="deck-pstep-n">${number}</span><span class="deck-pstep-t">${escapeHtml(step.label)}</span>${tag}</div>`;
}

function diffTab(path: string, diff: PatchProposalView["files"][number]["diff"], index: number) {
  return `<button class="deck-ftab ${index === 0 ? "on" : ""}" type="button" data-diff-file="${index}">
    <span class="ftag ${fileTag(path)}">${fileTag(path)}</span>
    <span class="mono deck-ftab-name">${escapeHtml(shortPath(path))}</span>
    <span class="deck-ftab-stat mono">${diffSummary(diff)}</span>
  </button>`;
}

function diffPanel(file: PatchProposalView["files"][number], patch: PatchProposalView, index: number) {
  const lines = file.diff.map((line, lineIndex) => workDiffLine(line, lineIndex)).join("");
  return `<div class="deck-diff-file-panel" data-diff-file="${index}" ${index === 0 ? "" : "hidden"}>
    <div class="deck-diff-head mono"><span>${escapeHtml(directoryPath(file.path))}/<strong>${escapeHtml(shortPath(file.path))}</strong></span><span class="deck-diff-stat">${diffSummary(file.diff)} &middot; ${escapeHtml(patch.status)}</span></div>
    <div class="deck-diff-body">${lines}</div>
  </div>`;
}

function workDiffLine(line: PatchProposalView["files"][number]["diff"][number], index: number) {
  const kind = line.kind === "added" ? "add" : line.kind === "removed" ? "del" : "ctx";
  const marker = line.kind === "added" ? "+" : line.kind === "removed" ? "-" : `${index + 1}`;
  return `<div class="dl ${kind}"><span class="ln">${marker}</span><span class="tx">${escapeHtml(line.text || " ")}</span></div>`;
}

function diffSummary(lines: PatchProposalView["files"][number]["diff"]) {
  const added = lines.filter((line) => line.kind === "added").length;
  const removed = lines.filter((line) => line.kind === "removed").length;
  return `+${added}${removed > 0 ? ` -${removed}` : ""}`;
}

function fileTag(path: string) {
  return path.endsWith(".new") ? "A" : "M";
}

function shortPath(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}

function directoryPath(path: string) {
  const parts = path.split(/[\\/]/);
  return parts.length > 1 ? parts.slice(0, -1).join("/") : ".";
}
