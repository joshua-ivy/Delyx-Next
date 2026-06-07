import { useEffect, useMemo, useState } from "react";
import { CommandPalette } from "../design-system/CommandPalette";
import { ShellPreferenceController } from "./ShellPreferenceController";
import { createRunForThread, threadWithRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { canTransition, createThread, modeForThreadStatus } from "./appShellThreadActions";
import { paletteCommands, runAppShellCommand } from "./appShellCommands";
import { buildCockpitMarkup } from "./cockpitView";
import { useCockpitDomBindings } from "./useCockpitDomBindings";
import { currentActionProposals } from "../features/approvals/approvalData";
import { currentAutomationState } from "../features/automations/automationData";
import { currentExternalAgentState } from "../features/externalAgents/externalAgentData";
import { currentMemoryState } from "../features/memory/memoryData";
import { currentMobileState } from "../features/mobile/mobileData";
import { currentModelSettings } from "../features/models/modelData";
import { currentPatchProposals } from "../features/patches/patchData";
import type { PlanView } from "../features/plans/planTypes";
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
  useCockpitDomBindings({ activePlan, activeProject, activeThread, cockpitHtml, setActiveThreadId, setPaletteOpen, setPlans, setThreadOpen, setThreadState, setWorkspaceOpen });
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
