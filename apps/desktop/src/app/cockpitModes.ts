import type { ThreadStatus } from "../features/threads/threadTypes";

const steps = [
  { key: "explore", label: "Explore" },
  { key: "plan", label: "Plan" },
  { key: "build", label: "Build" },
  { key: "test", label: "Test" },
  { key: "review", label: "Review" },
] as const;

export function emptyPipelineBlock() {
  return `<div class="pipe">
        <div class="pstep pending"><div class="pn">01</div><div class="ps">Explore</div><span class="pc">-</span></div>
        <div class="pstep pending"><div class="pn">02</div><div class="ps">Plan</div><span class="pc">-</span></div>
        <div class="pstep pending"><div class="pn">03</div><div class="ps">Build</div><span class="pc">-</span></div>
        <div class="pstep pending"><div class="pn">04</div><div class="ps">Test</div><span class="pc">-</span></div>
        <div class="pstep pending"><div class="pn">05</div><div class="ps">Review</div><span class="pc">-</span></div>
      </div>`;
}

export function modePill(status: ThreadStatus | undefined) {
  const mode = modeForStatus(status);
  return `<span class="pill ${mode.klass}"><span class="dot"></span>${mode.label}</span>`;
}

export function pipelineBlock(status: ThreadStatus | undefined) {
  const activeIndex = activeStepIndex(status);
  return `<div class="pipe">
        ${steps.map((step, index) => stepMarkup(index, step.label, activeIndex, status)).join("")}
      </div>`;
}

function stepMarkup(index: number, label: string, activeIndex: number, status: ThreadStatus | undefined) {
  const number = String(index + 1).padStart(2, "0");
  const state = stepState(index, activeIndex, status);
  const marker = state === "done" ? "ok" : state === "active" ? "*" : "-";
  return `<div class="pstep ${state}"><div class="pn">${number}</div><div class="ps">${label}</div><span class="pc">${marker}</span></div>`;
}

function stepState(index: number, activeIndex: number, status: ThreadStatus | undefined) {
  if (status === "done") {
    return "done";
  }
  if (activeIndex < 0) {
    return "pending";
  }
  if (index < activeIndex) {
    return "done";
  }
  return index === activeIndex ? "active" : "pending";
}

function activeStepIndex(status: ThreadStatus | undefined) {
  const active: Partial<Record<ThreadStatus, number>> = {
    building: 2,
    exploring: 0,
    planning: 1,
    reviewing: 4,
    testing: 3,
    waiting_for_approval: 1,
  };
  return status ? active[status] ?? -1 : -1;
}

function modeForStatus(status: ThreadStatus | undefined) {
  const modes: Partial<Record<ThreadStatus, { klass: string; label: string }>> = {
    blocked: { klass: "blocked", label: "BLOCKED" },
    building: { klass: "build", label: "BUILD MODE" },
    done: { klass: "done", label: "DONE" },
    exploring: { klass: "wait", label: "EXPLORE MODE" },
    failed: { klass: "failed", label: "FAILED" },
    idle: { klass: "ghost", label: "THREAD IDLE" },
    planning: { klass: "wait", label: "PLAN MODE" },
    reviewing: { klass: "wait", label: "REVIEW MODE" },
    testing: { klass: "wait", label: "TEST MODE" },
    waiting_for_approval: { klass: "blocked", label: "APPROVAL WAIT" },
  };
  return status ? modes[status] ?? { klass: "ghost", label: "NO MODE" } : { klass: "ghost", label: "NO MODE" };
}
