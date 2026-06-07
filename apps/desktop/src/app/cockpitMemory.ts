import type { MemoryCandidateView, MemoryRecordView, MemoryStateView } from "../features/memory/memoryTypes";
import { escapeHtml } from "./html";

export function emptyMemoryBlock() {
  return `<div class="dfile memory-review">
        <div class="dh"><span class="fn">Memory review</span><span class="dst">empty</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No memory candidates or durable memories exist.</span></div>
        </div>
      </div>`;
}

export function memoryBlock(state: MemoryStateView, runId: string | undefined) {
  const candidates = runId ? state.candidates.filter((item) => item.sourceRunId === runId) : state.candidates;
  const records = runId ? state.records.filter((item) => item.sourceRunId === runId) : state.records;
  if (candidates.length === 0 && records.length === 0) {
    return emptyMemoryBlock();
  }

  return `<div class="dfile memory-review">
        <div class="dh"><span class="fn">Memory review</span><span class="dst">${candidates.length} candidate(s) &middot; ${records.length} saved</span></div>
        <div class="dc">
          ${candidates.map(candidateLine).join("")}
          ${records.map(recordLine).join("")}
        </div>
      </div>`;
}

export function hasMemoryForRun(state: MemoryStateView, runId: string | undefined) {
  if (!runId) {
    return state.candidates.length > 0 || state.records.length > 0;
  }
  return state.candidates.some((item) => item.sourceRunId === runId)
    || state.records.some((item) => item.sourceRunId === runId);
}

function candidateLine(candidate: MemoryCandidateView) {
  return `<div class="dr ${candidate.status === "suppressed" ? "m" : ""}"><span class="g">cand</span><span class="x">${escapeHtml(candidate.scope)}:${escapeHtml(candidate.key)} &middot; ${escapeHtml(candidate.status)} &middot; ${sourceLabel(candidate)}</span></div>`;
}

function recordLine(record: MemoryRecordView) {
  const state = record.suppressed ? "suppressed" : "active";
  const supersedes = record.supersedes ? ` &middot; supersedes ${escapeHtml(record.supersedes)}` : "";
  return `<div class="dr ${record.suppressed ? "m" : "p"}"><span class="g">mem</span><span class="x">${escapeHtml(record.scope)}:${escapeHtml(record.key)} &middot; ${state}${supersedes} &middot; ${sourceLabel(record)}</span></div>`;
}

function sourceLabel(item: MemoryCandidateView | MemoryRecordView) {
  return `run ${escapeHtml(item.sourceRunId)} / thread ${escapeHtml(item.sourceThreadId)}`;
}
