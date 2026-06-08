import { useEffect, useState } from "react";
import { ShellPreferenceController } from "./ShellPreferenceController";
import { useApprovalPolicy } from "./appShellApprovalPolicy";
import { createRunForThread, threadWithRun, updateRunsForThreadStatus } from "./appShellRunActions";
import { canTransition, createThread, modeForThreadStatus } from "./appShellThreadActions";
import { paletteCommands, runAppShellCommand } from "./appShellCommands";
import { sendComposerInstruction } from "./cockpitComposerBindings";
import { decideFocusApproval } from "./focusApprovalDecision";
import { FocusShell } from "./FocusShell";
import { currentActionProposals } from "../features/approvals/approvalData";
import { externalAgentBridgeUnavailableState, loadExternalAgentStatus } from "../features/externalAgents/externalAgentClient";
import { currentExternalAgentState } from "../features/externalAgents/externalAgentData";
import type { ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import { currentMobileState } from "../features/mobile/mobileData";
import { currentModelSettings } from "../features/models/modelData";
import { refreshOllamaSettings } from "../features/models/ollamaClient";
import type { ModelSettingsView } from "../features/models/modelTypes";
import { loadPatchSnapshot } from "../features/patches/patchClient";
import { currentPatchProposals } from "../features/patches/patchData";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import { currentReviewReports } from "../features/review/reviewData";
import { currentAgentRuns } from "../features/runs/agentRunData";
import { currentTestArtifacts } from "../features/tests/testData";
import { archiveThreadOverBridge, createThreadRunOverBridge, loadThreadRunSnapshot, updateThreadStatusOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import { WorkspaceOverlay } from "../features/workspace/WorkspaceOverlay";
import { currentWorkspaceProject } from "../features/workspace/workspaceData";
import type { WorkspaceProject, WorkspaceUiState } from "../features/workspace/workspaceTypes";
import { loadRuntimeBridgeState, modelSettingsFromRuntimeStatus, webRuntimeBridge, type RuntimeBridgeState } from "./runtimeBridge";
import { usePersistedInspectorState } from "./usePersistedInspectorState";
import { loadWorkspaceProject } from "./workspaceBridge";

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
  const [externalAgentState, setExternalAgentState] = useState<ExternalAgentStateView>(currentExternalAgentState);
  const [patches, setPatches] = useState<PatchProposalView[]>(currentPatchProposals);
  const [runtimeBridge, setRuntimeBridge] = useState<RuntimeBridgeState>(webRuntimeBridge);
  const { automationState, memoryState, releaseState, skillState } = usePersistedInspectorState();
  const riskPolicy = useApprovalPolicy();
  const activeProject = projects[0] ?? currentWorkspaceProject;
  const visibleThreads = threads.filter((thread) => !thread.archived);
  const activeThread = visibleThreads.find((thread) => thread.id === activeThreadId) ?? visibleThreads[0];
  const activeRun = agentRuns.find((run) => run.id === activeThread?.activeRunId)
    ?? agentRuns.find((run) => run.threadId === activeThread?.id);
  const activePlan = plans.find((plan) => plan.threadId === activeThread?.id);
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
    void loadExternalAgentStatus().then((status) => {
      if (!cancelled) {
        setExternalAgentState((current) => ({ ...current, adapters: status.adapters }));
      }
    }).catch(() => {
      if (!cancelled) {
        setExternalAgentState(externalAgentBridgeUnavailableState);
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
    let cancelled = false;
    void loadThreadRunSnapshot(activeProject.id).then((snapshot) => {
      if (!cancelled && snapshot && snapshot.threads.length > 0) {
        setThreads((current) => current.length > 0 ? current : snapshot.threads);
        setAgentRuns((current) => current.length > 0 ? current : snapshot.runs);
        setActiveThreadId((current) => current ?? snapshot.threads[0]?.id);
        setThreadState("ready");
      }
    });
    return () => {
      cancelled = true;
    };
  }, [activeProject.id]);
  useEffect(() => {
    document.documentElement.dataset.mode = activeThread?.mode ?? "build";
  }, [activeThread?.mode]);
  useEffect(() => {
    if (!activeRun?.id) {
      setPatches([]);
      return;
    }
    setPatches([]);
    let cancelled = false;
    void loadPatchSnapshot(activeRun.id).then((snapshot) => {
      if (!cancelled) {
        setPatches(snapshot ?? []);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [activeRun?.id]);
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
  const runPaletteCommand = (commandId: string) => {
    runAppShellCommand(commandId, {
      actionProposals,
      activePlan,
      activeProject,
      activeRun,
      activeThread,
      modelSettings,
      setActionProposals,
      setAgentRuns,
      setExternalAgentState,
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
  const sendInstruction = (value: string) => {
    sendComposerInstruction({
      activeProject,
      activeRun,
      activeThread,
      modelSettings,
      setActiveThreadId,
      setAgentRuns,
      setThreads,
      setThreadState,
      threads,
    }, value);
  };
  const selectModel = (modelId: string) => {
    setModelSettings((current) => ({
      ...current,
      routes: [
        { modelId, providerId: "ollama-local", role: "coding", saved: false },
        ...current.routes.filter((route) => !(route.providerId === "ollama-local" && route.role === "coding")),
      ],
      selectedProviderId: "ollama-local",
    }));
  };
  const decideProposal = (proposalId: string, status: "approved" | "denied") => {
    void decideFocusApproval({
      activeRun,
      activeThread,
      actionProposals,
      setActionProposals,
      setAgentRuns,
      setThreads,
      setThreadState,
    }, proposalId, status);
  };
  const archiveActiveThread = () => {
    if (!activeThread) {
      setThreadState("empty");
      return;
    }
    const now = new Date().toISOString();
    void archiveThreadOverBridge(activeThread.id, now);
    setThreads((current) => current.map((thread) => (
      thread.id === activeThread.id ? { ...thread, archived: true, updatedAt: now } : thread
    )));
    setThreadState(visibleThreads.length <= 1 ? "empty" : "ready");
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
      <ShellPreferenceController />
      <FocusShell
        activePlan={activePlan}
        activeProject={activeProject}
        activeRun={activeRun}
        activeThread={activeThread}
        desktopShell={runtimeBridge.status?.desktopShell}
        modelSettings={modelSettings}
        onArchiveActive={archiveActiveThread}
        onApprovePlan={() => runPaletteCommand("plan.approve")}
        onCreatePlan={() => runPaletteCommand("plan.create")}
        onDecideProposal={decideProposal}
        onOpenWorkspace={() => setWorkspaceOpen(true)}
        onRefreshModels={() => runPaletteCommand("models.ollama.refresh")}
        onRunCommand={runPaletteCommand}
        onSelectModel={selectModel}
        onSelectThread={(threadId) => {
          setActiveThreadId(threadId);
          setThreadState("ready");
        }}
        onSendInstruction={sendInstruction}
        patches={patches}
        proposals={actionProposals}
        tests={currentTestArtifacts}
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
