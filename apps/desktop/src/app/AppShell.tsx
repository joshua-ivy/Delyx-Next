import { useEffect, useState } from "react";
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
import type { ModelSettingsView } from "../features/models/modelTypes";
import type { PlanView } from "../features/plans/planTypes";
import { currentAgentRuns } from "../features/runs/agentRunData";
import { archiveThreadOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadUiState } from "../features/threads/threadTypes";
import { WorkspaceOverlay } from "../features/workspace/WorkspaceOverlay";
import { currentWorkspaceProject } from "../features/workspace/workspaceData";
import type { WorkspaceProject, WorkspaceUiState } from "../features/workspace/workspaceTypes";
import { loadRuntimeBridgeState, modelSettingsFromRuntimeStatus, webRuntimeBridge, type RuntimeBridgeState } from "./runtimeBridge";
import { selectOllamaCodingModel } from "./modelSelection";
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
  const [modelSettings, setModelSettings] = useState<ModelSettingsView>(currentModelSettings);
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
    setModelSettings((current) => selectOllamaCodingModel(current, modelId));
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
        onResumeRun={() => { void resumeAndDispatchSchedulerRun({ actionProposals, activePlan, activeProject, activeRun, activeThread, modelSettings, patches, reviews, setActionProposals, setAgentRuns, setPatches, setReviews, setTests, setThreads, setThreadState, tests }); }}
        onRunReview={runReview}
        onRunTests={runTests}
        onRunCommand={runPaletteCommand}
        onSelectModel={selectModel}
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
