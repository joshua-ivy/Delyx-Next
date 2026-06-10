import { useEffect, useMemo, useRef, useState } from "react";
import { ShellPreferenceController } from "./ShellPreferenceController";
import { decideApprovalAndMaybeResume, resumeAndDispatchSchedulerRun } from "./appShellApprovalDecisionActions";
import { applyApprovedPatchForActiveRun } from "./appShellPatchActions";
import { recordFinalSupportForActiveThread } from "./appShellFinalAnswerActions";
import { requestRepairForReviewFinding, runReviewForActiveRun } from "./appShellReviewActions";
import { runTestsForActiveRun } from "./appShellTestActions";
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
import { ensureProject } from "../features/projects/projectClient";
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
  const [qaqcAdapterId, setQaqcAdapterId] = useState<string | undefined>(undefined);
  const [qaqcModel, setQaqcModel] = useState<string | undefined>(undefined);
  const [workerAdapterId, setWorkerAdapterId] = useState<string | undefined>(undefined);
  const [workerMode, setWorkerMode] = useState<"read_only" | "workspace_write">("read_only");
  // Auto-enable QA/QC once when a reviewer CLI is first detected. Tracked by a ref
  // so turning it off manually afterwards isn't re-enabled on the next render.
  const qaqcAutoSelected = useRef(false);
  const [nativeProjectId, setNativeProjectId] = useState<string | undefined>(undefined);
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
  // Default QA/QC on: pick the first detected reviewer CLI (Claude Code, else
  // Codex) so generated code is checked out of the box. Runs once.
  useEffect(() => {
    if (qaqcAutoSelected.current || qaqcAdapterId) {
      return;
    }
    const reviewer =
      externalAgentState.adapters.find((adapter) => adapter.id === "claude-code" && adapter.status === "available")
      ?? externalAgentState.adapters.find((adapter) => adapter.id === "codex-cli" && adapter.status === "available");
    if (reviewer) {
      qaqcAutoSelected.current = true;
      setQaqcAdapterId(reviewer.id);
    }
  }, [externalAgentState.adapters, qaqcAdapterId]);
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
  // Resolve (or create) the native project record for the active workspace so the
  // attachment pipeline has a real project id + policy to classify against.
  useEffect(() => {
    let cancelled = false;
    setNativeProjectId(undefined);
    void ensureProject(activeProject.name, activeProject.path)
      .then((record) => {
        if (!cancelled) setNativeProjectId(record.id);
      })
      .catch(() => {
        // Desktop runtime unavailable — attachments stay disabled.
      });
    return () => {
      cancelled = true;
    };
  }, [activeProject.name, activeProject.path]);
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
  const selectQaqc = (adapterId: string | undefined) => {
    setQaqcAdapterId(adapterId);
    // Reset to the new reviewer's economical default model.
    setQaqcModel(undefined);
  };
  const selectQaqcModel = (model: string) => {
    setQaqcModel(model);
  };
  const selectWorker = (adapterId: string | undefined, mode: "read_only" | "workspace_write" = "read_only") => {
    setWorkerAdapterId(adapterId);
    setWorkerMode(mode);
  };
  const runReview = () => {
    void runReviewForActiveRun({
      activeProject,
      activeRun,
      activeThread,
      patches: activeRun ? patches.filter((patch) => patch.runId === activeRun.id) : [],
      setAgentRuns,
      setReviews,
      setThreads,
      setThreadState,
      tests,
    });
  };
  const runTests = () => {
    void runTestsForActiveRun({
      actionProposals,
      activePlan,
      activeProject,
      activeRun,
      activeThread,
      patches,
      setActionProposals,
      setAgentRuns,
      setTests,
      setThreads,
      setThreadState,
    });
  };
  const recordFinal = () => {
    void recordFinalSupportForActiveThread({
      activeRun,
      activeThread,
      reviews,
      setAgentRuns,
      setThreads,
      setThreadState,
    });
  };
  const requestRepair = (reportId: string, findingId: string) => { void requestRepairForReviewFinding({ actionProposals, activeProject, activeRun, activeThread, reviews, setActionProposals, setAgentRuns, setReviews, setThreads, setThreadState }, reportId, findingId); };
  const applyPatch = (patchId: string) => {
    void applyApprovedPatchForActiveRun({
      actionProposals, activeProject, activeRun, activeThread,
      patch: patches.find((patch) => patch.id === patchId),
      setActionProposals, setAgentRuns,
      setPatches,
      setThreads,
      setThreadState,
    });
  };
  const decideProposal = (proposalId: string, status: "approved" | "denied") => {
    void decideApprovalAndMaybeResume({
      activePlan,
      activeProject,
      activeRun,
      activeThread,
      actionProposals,
      modelSettings,
      patches,
      reviews,
      setActionProposals,
      setAgentRuns,
      setPatches,
      setReviews,
      setTests,
      setThreads,
      setThreadState,
      tests,
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
        onDecideProposal={decideProposal}
        onOpenWorkspace={() => setWorkspaceOpen(true)}
        onApplyPatch={applyPatch}
        onRecordFinal={recordFinal}
        onRequestRepair={requestRepair}
        onRefreshModels={() => runPaletteCommand("models.ollama.refresh")}
        onLocalModelsChanged={refreshRuntimeModels}
        onResumeRun={() => { void resumeAndDispatchSchedulerRun({ actionProposals, activePlan, activeProject, activeRun, activeThread, modelSettings, patches, reviews, setActionProposals, setAgentRuns, setPatches, setReviews, setTests, setThreads, setThreadState, tests }); }}
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
