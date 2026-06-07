import type { TaskStatus } from "../app/types";

const statusTone: Record<TaskStatus, string> = {
  idle: "neutral",
  exploring: "info",
  planning: "info",
  waiting_for_approval: "warning",
  building: "info",
  testing: "info",
  reviewing: "info",
  blocked: "danger",
  failed: "danger",
  done: "success",
};

export function StatusPill({ status }: { status: TaskStatus }) {
  const label = status.replaceAll("_", " ");

  return <span className={`status-pill status-${statusTone[status]}`}>{label}</span>;
}
