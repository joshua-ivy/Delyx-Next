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
  if (from === "idle") {
    return ["active", "blocked", "failed", "done"].includes(to);
  }
  if (from === "active") {
    return ["idle", "blocked", "failed", "done"].includes(to);
  }
  if (from === "blocked") {
    return ["active", "failed", "done"].includes(to);
  }
  return false;
}

export function upsertPlan(plans: PlanView[], plan: PlanView) {
  const next = plans.filter((item) => item.threadId !== plan.threadId);
  return [plan, ...next];
}

function slugFromGoal(goal: string) {
  return goal.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 36) || "thread";
}
