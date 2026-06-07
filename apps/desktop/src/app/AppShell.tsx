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
import { refreshOllamaSettings } from "../features/models/ollamaClient";
import type { ModelSettingsView } from "../features/models/modelTypes";
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
import { loadRuntimeBridgeState, modelSettingsFromRuntimeStatus, type RuntimeBridgeState } from "./runtimeBridge";
import { loadWorkspaceProject } from "./workspaceBridge";

const webRuntimeBridge: RuntimeBridgeState = {
  label: "Web preview / Rust bridge unavailable",
  mode: "web",
};

export function AppShell() {
  const [activeThreadId, setActiveThreadId] = useState<string | undefined>();
  const [actionProposals, setActionProposals] = useState(currentActionProposals);
  const [plans, setPlans] = useState<PlanView[]>([]);
  const [threadOpen, setThreadOpen] = useState(false);
  const [threads, setThreads] = useState<TaskThread[]>([]);
  const [agentRuns, setAgentRuns] = useState(currentAgentRuns);
  const [threadState, setThreadState] = useState<ThreadUiState>("empty");
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [workspaceOpen, setWorkspaceOpen] = useState(false);
  const [projects, setProjects] = useState<WorkspaceProject[]>([currentWorkspaceProject]);
  const [workspaceState, setWorkspaceState] = useState<WorkspaceUiState>("ready");
  const [modelSettings, setModelSettings] = useState<ModelSettingsView>(currentModelSettings);
  const [runtimeBridge, setRuntimeBridge] = useState<RuntimeBridgeState>(webRuntimeBridge);
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
      actionProposals,
      currentPatchProposals,
      currentTestArtifacts,
      currentReviewReports,
      modelSettings,
      currentExternalAgentState,
      currentResearchAnswers,
      currentMemoryState,
      currentSkillState,
      currentAutomationState,
      currentMobileState,
      currentReleaseState,
      visibleThreads,
      runtimeBridge,
    ),
    [actionProposals, activePlan, activeProject, activeRun, activeThread, modelSettings, runtimeBridge, visibleThreads],
  );
  useEffect(() => {
    let cancelled = false;
    void loadRuntimeBridgeState().then(async (state) => {
      if (!cancelled) {
        setRuntimeBridge(state);
        if (state.status) {
          setModelSettings(modelSettingsFromRuntimeStatus(currentModelSettings, state.status));
          return;
        }
        const settings = await refreshOllamaSettings(currentModelSettings);
        if (!cancelled) {
          setModelSettings(settings);
        }
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);
  useEffect(() => {
    let cancelled = false;
    setWorkspaceState("loading");
    void loadWorkspaceProject().then((project) => {
      if (!cancelled) {
        setProjects([project]);
        setWorkspaceState("ready");
      }
    }).catch(() => {
      if (!cancelled) {
        setWorkspaceState("error");
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);
  useEffect(() => {
    document.documentElement.dataset.mode = activeThread?.mode ?? "build";
  }, [activeThread?.mode]);
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
  useCockpitDomBindings({
    actionProposals,
    activePlan,
    activeProject,
    activeRun,
    activeThread,
    cockpitHtml,
    modelSettings,
    setActionProposals,
    setActiveThreadId,
    setAgentRuns,
    setPaletteOpen,
    setPlans,
    setThreadOpen,
    setThreadState,
    setThreads,
    setWorkspaceOpen,
    threads,
  });
  const runPaletteCommand = (commandId: string) => {
    runAppShellCommand(commandId, {
      activePlan,
      activeProject,
      activeRun,
      activeThread,
      modelSettings,
      setActionProposals,
      setAgentRuns,
      setModelSettings,
      setPlans,
      setThreadOpen,
      setThreads,
      setThreadState,
      setWorkspaceOpen,
      threads,
    });
    setPaletteOpen(false);
  };
  const addWorkspaceProject = (path: string) => {
    setWorkspaceState("loading");
    void loadWorkspaceProject(path).then((project) => {
      setProjects([project]);
      setWorkspaceState("ready");
    }).catch(() => {
      setWorkspaceState("error");
    });
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
        open={threadOpen}
        state={threadState}
        threads={threads}
      />
      <WorkspaceOverlay
        activeThreadCount={visibleThreads.length}
        lastRun={agentRuns[0]}
        modelSettings={modelSettings}
        onAddProject={addWorkspaceProject}
        onClose={() => setWorkspaceOpen(false)}
        onRemoveProject={() => {
          setProjects([]);
          setWorkspaceState("empty");
        }}
        open={workspaceOpen}
        project={activeProject}
        projects={projects}
        state={workspaceState}
      />
    </>
  );
}
