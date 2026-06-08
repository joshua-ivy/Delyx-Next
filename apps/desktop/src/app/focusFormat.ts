import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export type FocusMode = "explore" | "plan" | "build" | "test" | "review";

export const focusModes: FocusMode[] = ["explore", "plan", "build", "test", "review"];

export function focusMode(thread: TaskThread | undefined, fallback: FocusMode): FocusMode {
  const mode = thread?.mode;
  return mode && focusModes.includes(mode as FocusMode) ? mode as FocusMode : fallback;
}

export function modeLabel(mode: FocusMode) {
  return mode[0].toUpperCase() + mode.slice(1);
}

export function modeStep(mode: FocusMode) {
  return focusModes.indexOf(mode);
}

export function selectedModel(settings: ModelSettingsView) {
  const provider = selectedProvider(settings);
  return settings.routes.find((route) => route.providerId === provider?.id && route.role === "coding")?.modelId
    ?? provider?.models[0]
    ?? "";
}

export function selectedProvider(settings: ModelSettingsView) {
  return settings.providers.find((provider) => provider.id === settings.selectedProviderId)
    ?? settings.providers[0];
}

export function repoLabel(project: WorkspaceProject) {
  if (!project.git.isRepo) {
    return "no repo";
  }
  if (!project.git.branch || project.git.branch.includes("not loaded")) {
    return "repo unknown";
  }
  return project.git.branch;
}

export function gitChangeLabel(project: WorkspaceProject) {
  if (project.git.uncommittedChanges === null) {
    return "changes unknown";
  }
  return `${project.git.uncommittedChanges} change(s)`;
}

export function runStatusLabel(run: AgentRunView | undefined, proposals: ActionProposalView[]) {
  if (proposals.some((proposal) => proposal.status === "pending")) {
    return "Waiting for approval";
  }
  return run ? run.status.replaceAll("_", " ") : "Ready";
}

export function latestRunEvent(run: AgentRunView | undefined) {
  return run?.events.at(-1)?.message ?? "No run event recorded yet.";
}

export function planProgress(plan: PlanView | undefined, approved: boolean) {
  return (plan?.steps ?? []).slice(0, 5).map((label, index) => ({
    label,
    state: approved || index === 0 ? index === 0 && !approved ? "now" : "done" : "",
  }));
}

export function shortTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}
