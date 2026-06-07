import { useEffect, useMemo, useState } from "react";
import { CommandPalette } from "../design-system/CommandPalette";
import { ShellPreferenceController } from "./ShellPreferenceController";
import { createRunForThread, threadWithRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { canTransition, createThread, modeForThreadStatus, upsertPlan } from "./appShellThreadActions";
import { paletteCommands, runAppShellCommand } from "./appShellCommands";
import { buildCockpitMarkup } from "./cockpitView";
import { currentActionProposals } from "../features/approvals/approvalData";
import { currentAutomationState } from "../features/automations/automationData";
import { currentExternalAgentState } from "../features/externalAgents/externalAgentData";
import { currentMemoryState } from "../features/memory/memoryData";
import { currentMobileState } from "../features/mobile/mobileData";
import { currentModelSettings } from "../features/models/modelData";
import { currentPatchProposals } from "../features/patches/patchData";
import { createPlanFromThread } from "../features/plans/planBuilder";
import type { PlanDecision, PlanView } from "../features/plans/planTypes";
import { currentReleaseState } from "../features/release/releaseData";
import { currentReviewReports } from "../features/review/reviewData";
import { currentAgentRuns } from "../features/runs/agentRunData";
import { currentResearchAnswers } from "../features/research/researchData";
import { currentSkillState } from "../features/skills/skillData";
import { currentTestArtifacts } from "../features/tests/testData";
import { ThreadOverlay } from "../features/threads/ThreadOverlay";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import { WorkspaceOverlay } from "../features/workspace/WorkspaceOverlay";
import { currentWorkspaceProject } from "../features/workspace/workspaceData";
import type { WorkspaceProject, WorkspaceUiState } from "../features/workspace/workspaceTypes";
export function AppShell() {
  const [activeThreadId, setActiveThreadId] = useState<string | undefined>();
  const [plans, setPlans] = useState<PlanView[]>([]);
  const [threadOpen, setThreadOpen] = useState(false);
  const [threads, setThreads] = useState<TaskThread[]>([]);
  const [agentRuns, setAgentRuns] = useState(currentAgentRuns);
  const [threadState, setThreadState] = useState<ThreadUiState>("empty");
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [workspaceOpen, setWorkspaceOpen] = useState(false);
  const [projects, setProjects] = useState<WorkspaceProject[]>([currentWorkspaceProject]);
  const [workspaceState, setWorkspaceState] = useState<WorkspaceUiState>("ready");
  const activeProject = projects[0] ?? currentWorkspaceProject;
  const visibleThreads = threads.filter((thread) => !thread.archived);
  const activeThread = visibleThreads.find((thread) => thread.id === activeThreadId) ?? visibleThreads[0];
  const activeRun = agentRuns.find((run) => run.id === activeThread?.activeRunId)
    ?? agentRuns.find((run) => run.threadId === activeThread?.id);
  const activePlan = plans.find((plan) => plan.threadId === activeThread?.id);
  const cockpitHtml = useMemo(
    () => buildCockpitMarkup(
      activeProject,
      activeThread,
      activeRun,
      activePlan,
      currentActionProposals,
      currentPatchProposals,
      currentTestArtifacts,
      currentReviewReports,
      currentModelSettings,
      currentExternalAgentState,
      currentResearchAnswers,
      currentMemoryState,
      currentSkillState,
      currentAutomationState,
      currentMobileState,
      currentReleaseState,
      visibleThreads,
    ),
    [activePlan, activeProject, activeRun, activeThread, visibleThreads],
  );
  useEffect(() => {
    document.documentElement.dataset.mode = "build";
  }, []);
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen(true);
      }
      if (event.key === "Escape") {
        setPaletteOpen(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
  useEffect(() => {
    const commandButton = document.querySelector(".command-trigger");
    const projectButton = document.querySelector('.rail .rnav[title="Projects"]');
    const threadButton = document.querySelector(".side-h .add");
    const planCreate = document.querySelector(".plan-create");
    const planApprove = document.querySelector(".plan-approve");
    const planRevise = document.querySelector(".plan-revise");
    const planCancel = document.querySelector(".plan-cancel");
    const reviewReviseButtons = Array.from(document.querySelectorAll(".review-revise"));
    const cards = Array.from(document.querySelectorAll<HTMLElement>(".tcard[data-thread-id]"));
    const openProject = () => setWorkspaceOpen(true);
    const openThread = () => setThreadOpen(true);
    const openPalette = () => setPaletteOpen(true);
    const createPlan = () => {
      if (!activeThread) {
        setThreadState("empty");
        return;
      }
      setPlans((current) => upsertPlan(current, createPlanFromThread(activeThread, activeProject)));
    };
    const updatePlanDecision = (decision: PlanDecision) => {
      if (!activePlan) {
        setThreadState(activeThread ? "ready" : "empty");
        return;
      }
      setPlans((current) => current.map((plan) => (
        plan.threadId === activePlan.threadId ? { ...plan, decision } : plan
      )));
    };
    const approvePlan = () => updatePlanDecision("approved");
    const revisePlan = () => updatePlanDecision("revision_requested");
    const cancelPlan = () => updatePlanDecision("cancelled");
    const selectThread = (event: Event) => {
      const threadId = (event.currentTarget as HTMLElement).dataset.threadId;
      if (threadId) {
        setActiveThreadId(threadId);
        setThreadState("ready");
      }
    };
    const activateOnKeyboard = (event: Event) => {
      const key = (event as KeyboardEvent).key;
      if (key === "Enter" || key === " ") {
        event.preventDefault();
        (event.currentTarget as HTMLElement).click();
      }
    };
    projectButton?.setAttribute("role", "button");
    projectButton?.setAttribute("tabindex", "0");
    projectButton?.setAttribute("aria-label", "Open workspace manager");
    threadButton?.setAttribute("role", "button");
    threadButton?.setAttribute("tabindex", "0");
    threadButton?.setAttribute("aria-label", "Open thread manager");
    [planCreate, planApprove, planRevise, planCancel].forEach((button) => {
      button?.setAttribute("role", "button");
      button?.setAttribute("tabindex", "0");
    });
    planCreate?.setAttribute("aria-label", "Create plan");
    planApprove?.setAttribute("aria-label", "Approve plan");
    planRevise?.setAttribute("aria-label", "Revise plan");
    planCancel?.setAttribute("aria-label", "Cancel plan");
    commandButton?.setAttribute("role", "button");
    commandButton?.setAttribute("tabindex", "0");
    commandButton?.setAttribute("aria-label", "Open command palette");
    commandButton?.addEventListener("click", openPalette);
    commandButton?.addEventListener("keydown", activateOnKeyboard);
    projectButton?.addEventListener("click", openProject);
    projectButton?.addEventListener("keydown", activateOnKeyboard);
    threadButton?.addEventListener("click", openThread);
    threadButton?.addEventListener("keydown", activateOnKeyboard);
    planCreate?.addEventListener("click", createPlan);
    planCreate?.addEventListener("keydown", activateOnKeyboard);
    planApprove?.addEventListener("click", approvePlan);
    planApprove?.addEventListener("keydown", activateOnKeyboard);
    planRevise?.addEventListener("click", revisePlan);
    planRevise?.addEventListener("keydown", activateOnKeyboard);
    planCancel?.addEventListener("click", cancelPlan);
    planCancel?.addEventListener("keydown", activateOnKeyboard);
    reviewReviseButtons.forEach((button) => {
      button.setAttribute("role", "button");
      button.setAttribute("tabindex", "0");
      button.setAttribute("aria-label", "Ask Delyx to revise this finding");
      button.addEventListener("click", revisePlan);
      button.addEventListener("keydown", activateOnKeyboard);
    });
    cards.forEach((card) => {
      card.setAttribute("role", "button");
      card.setAttribute("tabindex", "0");
      card.addEventListener("click", selectThread);
      card.addEventListener("keydown", activateOnKeyboard);
    });
    return () => {
      commandButton?.removeEventListener("click", openPalette);
      commandButton?.removeEventListener("keydown", activateOnKeyboard);
      projectButton?.removeEventListener("click", openProject);
      projectButton?.removeEventListener("keydown", activateOnKeyboard);
      threadButton?.removeEventListener("click", openThread);
      threadButton?.removeEventListener("keydown", activateOnKeyboard);
      planCreate?.removeEventListener("click", createPlan);
      planCreate?.removeEventListener("keydown", activateOnKeyboard);
      planApprove?.removeEventListener("click", approvePlan);
      planApprove?.removeEventListener("keydown", activateOnKeyboard);
      planRevise?.removeEventListener("click", revisePlan);
      planRevise?.removeEventListener("keydown", activateOnKeyboard);
      planCancel?.removeEventListener("click", cancelPlan);
      planCancel?.removeEventListener("keydown", activateOnKeyboard);
      reviewReviseButtons.forEach((button) => {
        button.removeEventListener("click", revisePlan);
        button.removeEventListener("keydown", activateOnKeyboard);
      });
      cards.forEach((card) => {
        card.removeEventListener("click", selectThread);
        card.removeEventListener("keydown", activateOnKeyboard);
      });
    };
  }, [activePlan, activeProject, activeThread, cockpitHtml]);
  const runPaletteCommand = (commandId: string) => {
    runAppShellCommand(commandId, {
      activePlan,
      activeProject,
      activeThread,
      setPlans,
      setThreadOpen,
      setThreads,
      setThreadState,
      setWorkspaceOpen,
      setWorkspaceState,
    });
    setPaletteOpen(false);
  };
  return (
    <>
      <CommandPalette commands={paletteCommands} onClose={() => setPaletteOpen(false)} onRun={runPaletteCommand} open={paletteOpen} />
      <ShellPreferenceController />
      <div dangerouslySetInnerHTML={{ __html: cockpitHtml }} />
      <ThreadOverlay
        activeThread={activeThread}
        onArchiveActive={() => {
          if (!activeThread) {
            setThreadState("empty");
            return;
          }
          setThreads((current) => current.map((thread) => (
            thread.id === activeThread.id ? { ...thread, archived: true, updatedAt: new Date().toISOString() } : thread
          )));
          setThreadState(visibleThreads.length <= 1 ? "empty" : "ready");
        }}
        onClose={() => setThreadOpen(false)}
        onCreateThread={(goal) => {
          const thread = createThread(goal, activeProject.id, threads.length + 1);
          if (!thread) {
            setThreadState("error");
            return;
          }
          const run = createRunForThread(thread, activeProject.id, threads.length + 1);
          const runnableThread = threadWithRun(thread, run);
          setAgentRuns((current) => [run, ...current]);
          setThreads((current) => [runnableThread, ...current]);
          setActiveThreadId(runnableThread.id);
          setThreadState("ready");
        }}
        onSelectThread={(threadId) => {
          setActiveThreadId(threadId);
          setThreadState("ready");
        }}
        onSetStatus={(status) => {
          if (!activeThread) {
            setThreadState("empty");
            return;
          }
          if (!canTransition(activeThread.status, status)) {
            setThreadState("error");
            return;
          }
          const now = new Date().toISOString();
          setAgentRuns((current) => updateRunsForThreadStatus(current, activeThread, status, now));
          setThreads((current) => current.map((thread) => (
            thread.id === activeThread.id ? { ...thread, mode: modeForThreadStatus(status), status, updatedAt: now } : thread
          )));
          setThreadState("ready");
        }}
        onShowEmpty={() => {
          setThreads([]);
          setAgentRuns([]);
          setActiveThreadId(undefined);
          setPlans([]);
          setThreadState("empty");
        }}
        open={threadOpen}
        state={threadState}
        threads={threads}
      />
      <WorkspaceOverlay
        activeThreadCount={visibleThreads.length}
        onAddProject={(path) => {
          if (path.trim() === currentWorkspaceProject.path) {
            setProjects([currentWorkspaceProject]);
            setWorkspaceState("ready");
          } else {
            setWorkspaceState("error");
          }
        }}
        onClose={() => setWorkspaceOpen(false)}
        onRemoveProject={() => {
          setProjects([]);
          setWorkspaceState("empty");
        }}
        onShowDenied={() => setWorkspaceState("denied")}
        onSimulateError={() => setWorkspaceState("error")}
        onSimulateLoading={() => setWorkspaceState("loading")}
        open={workspaceOpen}
        project={activeProject}
        projects={projects}
        state={workspaceState}
      />
    </>
  );
}
