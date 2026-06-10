import { useEffect, useMemo, useState } from "react";
import { ShellPreferenceController } from "./ShellPreferenceController";
import { buildFocusRunHandlers } from "./appShellRunHandlers";
import { paletteCommands, runAppShellCommand } from "./appShellCommands";
import { sendComposerInstruction } from "./cockpitComposerBindings";
import { FocusShell } from "./FocusShell";
import { externalAgentBridgeUnavailableState, loadExternalAgentStatus } from "../features/externalAgents/externalAgentClient";
import { currentExternalAgentState } from "../features/externalAgents/externalAgentData";
import type { ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import { currentModelSettings } from "../features/models/modelData";
import { refreshOllamaSettings } from "../features/models/ollamaClient";
import type { ModelSelectionKey, ModelSettingsView } from "../features/models/modelTypes";
import type { PlanView } from "../features/plans/planTypes";
import { currentAgentRuns } from "../features/runs/agentRunData";
import { archiveThreadOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import { WorkspaceOverlay } from "../features/workspace/WorkspaceOverlay";
import { currentWorkspaceProject } from "../features/workspace/workspaceData";
import type { WorkspaceProject, WorkspaceUiState } from "../features/workspace/workspaceTypes";
import { loadRuntimeBridgeState, modelSettingsFromRuntimeStatus, webRuntimeBridge, type RuntimeBridgeState } from "./runtimeBridge";
import { mergeCliProviders, selectModelRoute } from "./cliModels";
import { useAgentSelections } from "./useAgentSelections";
import { useRunApprovals } from "./useRunApprovals";
import { useRunReceipts } from "./useRunReceipts";
import { useSchedulerDecision } from "./useSchedulerDecision";
import { useProjectSnapshots } from "./useProjectSnapshots";
import { loadWorkspaceProject } from "./workspaceBridge";

export function AppShell() {
  const [activeThreadId, setActiveThreadId] = useState<string | undefined>();
  const [plans, setPlans] = useState<PlanView[]>([]);
  const [threadOpen, setThreadOpen] = useState(false);
  const [threads, setThreads] = useState<TaskThread[]>([]);
  const [agentRuns, setAgentRuns] = useState(currentAgentRuns);
  const [threadState, setThreadState] = useState<ThreadUiState>("empty");
  const [workspaceOpen, setWorkspaceOpen] = useState(false);
  const [projects, setProjects] = useState<WorkspaceProject[]>([currentWorkspaceProject]);
  const [workspaceState, setWorkspaceState] = useState<WorkspaceUiState>("ready");
  const [baseModelSettings, setModelSettings] = useState<ModelSettingsView>(currentModelSettings);
  const [externalAgentState, setExternalAgentState] = useState<ExternalAgentStateView>(currentExternalAgentState);
  const [runtimeBridge, setRuntimeBridge] = useState<RuntimeBridgeState>(webRuntimeBridge);
  const activeProject = projects[0] ?? currentWorkspaceProject;
  const visibleThreads = threads.filter((thread) => !thread.archived);
  const activeThread = visibleThreads.find((thread) => thread.id === activeThreadId) ?? visibleThreads[0];
  const activeRun = agentRuns.find((run) => run.id === activeThread?.activeRunId)
    ?? agentRuns.find((run) => run.threadId === activeThread?.id);
  const activePlan = plans.find((plan) => plan.threadId === activeThread?.id);
  const { actionProposals, setActionProposals } = useRunApprovals(activeRun?.id);
  const { patches, reviews, setPatches, setReviews, setTests, tests } = useRunReceipts(activeRun?.id);
  const schedulerDecision = useSchedulerDecision({ activePlan, activeProject, activeRun, patches, proposals: actionProposals, reviews, tests });
  const {
    nativeProjectId, qaqcAdapterId, qaqcModel, selectQaqc, selectQaqcModel, selectWorker, workerAdapterId, workerMode,
  } = useAgentSelections(externalAgentState.adapters, activeProject);
  const modelSettings = useMemo(
    () => mergeCliProviders(baseModelSettings, externalAgentState.adapters),
    [baseModelSettings, externalAgentState.adapters],
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
  useProjectSnapshots({ projectId: activeProject.id, setActiveThreadId, setAgentRuns, setPlans, setThreads, setThreadState });
  useEffect(() => {
    document.documentElement.dataset.mode = activeThread?.mode ?? "build";
  }, [activeThread?.mode]);
  const runPaletteCommand = (commandId: string) => {
    runAppShellCommand(commandId, {
      actionProposals,
      activePlan,
      activeProject,
      activeRun, activeThread,
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
  };
  const composerState = () => ({
    activeProject,
    activeRun,
    activeThread,
    modelSettings,
    qaqcAdapterId,
    qaqcModel,
    workerAdapterId,
    workerMode,
    setActionProposals,
    setActiveThreadId,
    setAgentRuns,
    setThreads,
    setThreadState,
    threads,
  });
  const sendInstruction = (value: string, newThread = false) => {
    sendComposerInstruction(composerState(), value, newThread);
  };
  const launchWorker = () => {
    if (!activeThread) {
      return;
    }
    const runId = activeThread.activeRunId;
    void import("./appShellWorkerActions").then(async ({ launchQueuedWorker }) => {
      await launchQueuedWorker(composerState(), activeThread, actionProposals);
      // A write-capable worker promotes its edits into a patch record; reload so
      // the reviewable diff (and its restore action) appears immediately.
      if (runId) {
        const { loadPatchSnapshot } = await import("../features/patches/patchClient");
        const snapshot = await loadPatchSnapshot(runId);
        if (snapshot) {
          setPatches(snapshot);
        }
      }
    });
  };
  const selectModel = (selection: ModelSelectionKey) => {
    setModelSettings((current) => selectModelRoute(current, externalAgentState.adapters, selection));
  };
  // Re-read the full runtime status (Delyx Local + Ollama) after the local model
  // catalog changes. Importing a .gguf only updates the import panel's own list;
  // without this, the `delyx-local` provider stays "not ready" in the picker, so
  // selecting it is silently dropped and chat keeps falling back to Ollama.
  const refreshRuntimeModels = () => {
    void loadRuntimeBridgeState().then((state) => {
      setRuntimeBridge(state);
      const status = state.status;
      if (status) {
        setModelSettings((current) => modelSettingsFromRuntimeStatus(current, status));
      }
    });
  };
  const { applyPatch, decideProposal, recordFinal, requestRepair, resumeRun, runReview, runTests } = buildFocusRunHandlers({
    actionProposals, activePlan, activeProject, activeRun, activeThread, modelSettings,
    patches, reviews, setActionProposals, setAgentRuns, setPatches, setReviews, setTests,
    setThreads, setThreadState, tests,
  });
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
        onDecideProposal={decideProposal}
        onOpenWorkspace={() => setWorkspaceOpen(true)}
        onApplyPatch={applyPatch}
        onRecordFinal={recordFinal}
        onRequestRepair={requestRepair}
        onRefreshModels={() => runPaletteCommand("models.ollama.refresh")}
        onLocalModelsChanged={refreshRuntimeModels}
        onResumeRun={resumeRun}
        onRunReview={runReview}
        onRunTests={runTests}
        onRunCommand={runPaletteCommand}
        onSelectModel={selectModel}
        onSelectQaqc={selectQaqc}
        onSelectQaqcModel={selectQaqcModel}
        onSelectWorker={selectWorker}
        onLaunchWorker={launchWorker}
        nativeProjectId={nativeProjectId}
        qaqcAdapterId={qaqcAdapterId}
        qaqcModel={qaqcModel}
        workerAdapterId={workerAdapterId}
        workerMode={workerMode}
        onSelectThread={(threadId) => {
          setActiveThreadId(threadId);
          setThreadState("ready");
        }}
        onSendInstruction={sendInstruction}
        patches={patches}
        proposals={actionProposals}
        reviews={reviews}
        schedulerDecision={schedulerDecision}
        tests={tests}
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
