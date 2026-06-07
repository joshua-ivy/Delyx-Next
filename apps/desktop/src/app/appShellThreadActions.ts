import type { PlanView } from "../features/plans/planTypes";
import type { TaskThread, ThreadStatus } from "../features/threads/threadTypes";

export function createThread(goal: string, projectId: string, index: number): TaskThread | undefined {
  const trimmed = goal.trim();
  if (!trimmed) {
    return undefined;
  }

  return {
    archived: false,
    createdLabel: "Now",
    goal: trimmed,
    id: `${slugFromGoal(trimmed)}-${index}`,
    messages: [{ role: "user", body: trimmed }],
    projectId,
    status: "idle",
    title: trimmed.length > 54 ? `${trimmed.slice(0, 51)}...` : trimmed,
  };
}

export function canTransition(from: ThreadStatus, to: ThreadStatus) {
  if (from === to) {
    return true;
  }
  const allowed: Record<ThreadStatus, ThreadStatus[]> = {
    idle: ["exploring", "planning", "blocked", "failed", "done"],
    exploring: ["planning", "blocked", "failed", "done"],
    planning: ["waiting_for_approval", "building", "blocked", "failed", "done"],
    waiting_for_approval: ["building", "blocked", "failed"],
    building: ["testing", "reviewing", "blocked", "failed", "done"],
    testing: ["reviewing", "blocked", "failed", "done"],
    reviewing: ["building", "blocked", "failed", "done"],
    blocked: ["exploring", "planning", "building", "failed", "done"],
    failed: [],
    done: [],
  };
  return allowed[from].includes(to);
}

export function upsertPlan(plans: PlanView[], plan: PlanView) {
  const next = plans.filter((item) => item.threadId !== plan.threadId);
  return [plan, ...next];
}

function slugFromGoal(goal: string) {
  return goal.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 36) || "thread";
}
