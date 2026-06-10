import { useEffect, useRef, useState } from "react";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSelectionKey, ModelSettingsView } from "../features/models/modelTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { FocusHome } from "./FocusHome";
import { FocusCommandPalette, FocusModelMenu, FocusThreadsMenu } from "./FocusOverlays";
import { FocusSettings } from "./FocusSettings";
import { FocusThread } from "./FocusThread";
import { FocusIcon, RailIconButton } from "./focusAtoms";
import { focusMode, selectedModel, type FocusMode } from "./focusFormat";
import type { DesktopShellStatusView } from "./runtimeBridge";

type FocusView = "home" | "thread" | "settings";
type FocusOverlay = "palette" | "threads" | "models" | undefined;

interface FocusShellProps {
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  desktopShell: DesktopShellStatusView | undefined;
  modelSettings: ModelSettingsView;
  onApplyPatch: (patchId: string) => void;
  onArchiveActive: () => void;
  onApprovePlan: () => void;
  onDecideProposal: (proposalId: string, status: "approved" | "denied") => void;
  onOpenWorkspace: () => void;
  onRecordFinal: () => void;
  onRefreshModels: () => void;
  onLocalModelsChanged?: () => void;
  onRequestRepair: (reportId: string, findingId: string) => void;
  onResumeRun: () => void;
  onRunReview: () => void;
  onRunTests: () => void;
  onRunCommand: (commandId: string) => void;
  onSelectModel: (selection: ModelSelectionKey) => void;
  onSelectQaqc: (adapterId: string | undefined) => void;
  onSelectQaqcModel?: (model: string) => void;
  onSelectWorker?: (adapterId: string | undefined, mode?: "read_only" | "workspace_write") => void;
  onLaunchWorker?: () => void;
  nativeProjectId?: string;
  qaqcAdapterId?: string;
  qaqcModel?: string;
  workerAdapterId?: string;
  workerMode?: "read_only" | "workspace_write";
  onSelectThread: (threadId: string) => void;
  onSendInstruction: (value: string, newThread?: boolean) => void;
  patches: PatchProposalView[];
  proposals: ActionProposalView[];
  reviews: ReviewReportView[];
  schedulerDecision: AgentScheduleDecisionView | undefined;
  tests: TestArtifactView[];
  threads: TaskThread[];
}

export function FocusShell(props: FocusShellProps) {
  const [view, setView] = useState<FocusView>(props.activeThread ? "thread" : "home");
  const [overlay, setOverlay] = useState<FocusOverlay>();
  const [fallbackMode, setFallbackMode] = useState<FocusMode>("build");
  // The workflow-derived mode follows thread status; a manual chip pick overrides
  // it until the workflow actually advances (then we clear the override).
  const derivedMode = focusMode(props.activeThread, fallbackMode);
  const [userMode, setUserMode] = useState<FocusMode | undefined>(undefined);
  const lastDerivedMode = useRef(derivedMode);
  useEffect(() => {
    if (lastDerivedMode.current !== derivedMode) {
      lastDerivedMode.current = derivedMode;
      setUserMode(undefined);
    }
  }, [derivedMode]);
  const mode = userMode ?? derivedMode;
  const visibleThreads = props.threads.filter((thread) => !thread.archived);
  const activePatches = props.activeRun ? props.patches.filter((patch) => patch.runId === props.activeRun?.id) : [];
  const activeProposals = props.activeRun ? props.proposals.filter((proposal) => proposal.runId === props.activeRun?.id) : [];
  const activeTests = props.activeRun ? props.tests.filter((test) => test.runId === props.activeRun?.id) : [];

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const key = event.key.toLowerCase();
      if ((event.ctrlKey || event.metaKey) && key === "k") {
        event.preventDefault();
        setOverlay((current) => current === "palette" ? undefined : "palette");
      } else if ((event.ctrlKey || event.metaKey) && key === "p") {
        event.preventDefault();
        setOverlay("threads");
      } else if ((event.ctrlKey || event.metaKey) && key === ",") {
        event.preventDefault();
        setView("settings");
      } else if ((event.ctrlKey || event.metaKey) && key === "n") {
        event.preventDefault();
        setView("home");
      } else if (key === "escape") {
        setOverlay(undefined);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const send = (value: string) => {
    // The home / new-chat composer starts a fresh thread; the in-thread composer
    // continues the open one.
    const newThread = view === "home" || !props.activeThread;
    props.onSendInstruction(value, newThread);
    setView("thread");
  };
  const setMode = (next: FocusMode) => {
    setUserMode(next);
    setFallbackMode(next);
  };

  return (
    <div className="focus-app" data-mode={mode}>
      <aside className="rail">
        <button className="rail-logo" onClick={() => setView("home")} title="Home" type="button">D</button>
        <RailIconButton active={view === "home"} icon="home" label="Home" onClick={() => setView("home")} />
        <RailIconButton active={overlay === "threads"} icon="threads" label="Threads" onClick={() => setOverlay("threads")} />
        <button className="rail-btn" title="Commands (Ctrl K)" type="button" onClick={() => setOverlay("palette")}><FocusIcon name="cmd" /></button>
        <div className="rail-spacer" />
        <div className="rail-pipe">{[0, 1, 2, 3, 4].map((item) => <span className={`rail-pd${item === stepForMode(mode) ? " on" : ""}`} key={item} />)}</div>
        <RailIconButton active={view === "settings"} icon="settings" label="Settings" onClick={() => setView("settings")} />
      </aside>

      {view === "home" && <FocusHome mode={mode} modelSettings={props.modelSettings} onModeChange={setMode} onOpenModels={() => setOverlay("models")} onOpenPalette={() => setOverlay("palette")} onOpenWorkspace={props.onOpenWorkspace} onSend={send} project={props.activeProject} projectId={props.nativeProjectId} />}
      {view === "thread" && props.activeThread && <FocusThread activePlan={props.activePlan} mode={mode} model={selectedModel(props.modelSettings)} onApplyPatch={props.onApplyPatch} onApprovePlan={props.onApprovePlan} onDecideProposal={props.onDecideProposal} onModeChange={setMode} onOpenPalette={() => setOverlay("palette")} onRecordFinal={props.onRecordFinal} onRequestRepair={props.onRequestRepair} onResumeRun={props.onResumeRun} onRunReview={props.onRunReview} onRunTests={props.onRunTests} onSend={send} onLaunchWorker={props.onLaunchWorker} projectId={props.nativeProjectId} patches={activePatches} proposals={activeProposals} reviews={props.reviews.filter((report) => report.runId === props.activeRun?.id)} run={props.activeRun} schedulerDecision={props.schedulerDecision} tests={activeTests} thread={props.activeThread} />}
      {view === "thread" && !props.activeThread && <FocusHome mode={mode} modelSettings={props.modelSettings} onModeChange={setMode} onOpenModels={() => setOverlay("models")} onOpenPalette={() => setOverlay("palette")} onOpenWorkspace={props.onOpenWorkspace} onSend={send} project={props.activeProject} projectId={props.nativeProjectId} />}
      {view === "settings" && <FocusSettings activeRun={props.activeRun} desktopShell={props.desktopShell} mode={mode} modelSettings={props.modelSettings} onLocalModelsChanged={props.onLocalModelsChanged} onModeChange={setMode} onRefreshModels={props.onRefreshModels} onSelectModel={props.onSelectModel} project={props.activeProject} threads={visibleThreads} />}

      {overlay === "palette" && <FocusCommandPalette onArchiveActive={props.onArchiveActive} onClose={() => setOverlay(undefined)} onOpenModels={() => setOverlay("models")} onOpenThreads={() => setOverlay("threads")} onOpenWorkspace={props.onOpenWorkspace} onRunCommand={props.onRunCommand} onView={(next) => setView(next)} />}
      {overlay === "threads" && <FocusThreadsMenu activeThreadId={props.activeThread?.id} onClose={() => setOverlay(undefined)} onNewThread={() => setView("home")} onSelectThread={(threadId) => { props.onSelectThread(threadId); setView("thread"); }} threads={visibleThreads} />}
      {overlay === "models" && <FocusModelMenu modelSettings={props.modelSettings} onClose={() => setOverlay(undefined)} onRefreshModels={props.onRefreshModels} onSelectModel={props.onSelectModel} onSelectQaqc={props.onSelectQaqc} onSelectQaqcModel={props.onSelectQaqcModel} onSelectWorker={props.onSelectWorker} qaqcAdapterId={props.qaqcAdapterId} qaqcModel={props.qaqcModel} workerAdapterId={props.workerAdapterId} workerMode={props.workerMode} />}
    </div>
  );
}

function stepForMode(mode: FocusMode) {
  return mode === "explore" ? 0 : mode === "plan" ? 1 : mode === "build" ? 2 : mode === "test" ? 3 : 4;
}
