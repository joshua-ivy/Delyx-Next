import { useEffect, useState } from "react";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
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
  onRequestRepair: (reportId: string, findingId: string) => void;
  onResumeRun: () => void;
  onRunReview: () => void;
  onRunTests: () => void;
  onRunCommand: (commandId: string) => void;
  onSelectModel: (modelId: string) => void;
  onSelectQaqc: (adapterId: string | undefined) => void;
  qaqcAdapterId?: string;
  onSelectThread: (threadId: string) => void;
  onSendInstruction: (value: string) => void;
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
  const mode = focusMode(props.activeThread, fallbackMode);
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
    props.onSendInstruction(value);
    setView("thread");
  };
  const setMode = (next: FocusMode) => setFallbackMode(next);

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

      {view === "home" && <FocusHome mode={mode} modelSettings={props.modelSettings} onModeChange={setMode} onOpenModels={() => setOverlay("models")} onOpenPalette={() => setOverlay("palette")} onOpenWorkspace={props.onOpenWorkspace} onSend={send} project={props.activeProject} />}
      {view === "thread" && props.activeThread && <FocusThread activePlan={props.activePlan} mode={mode} model={selectedModel(props.modelSettings)} onApplyPatch={props.onApplyPatch} onApprovePlan={props.onApprovePlan} onDecideProposal={props.onDecideProposal} onModeChange={setMode} onOpenPalette={() => setOverlay("palette")} onRecordFinal={props.onRecordFinal} onRequestRepair={props.onRequestRepair} onResumeRun={props.onResumeRun} onRunReview={props.onRunReview} onRunTests={props.onRunTests} onSend={send} patches={activePatches} proposals={activeProposals} reviews={props.reviews.filter((report) => report.runId === props.activeRun?.id)} run={props.activeRun} schedulerDecision={props.schedulerDecision} tests={activeTests} thread={props.activeThread} />}
      {view === "thread" && !props.activeThread && <FocusHome mode={mode} modelSettings={props.modelSettings} onModeChange={setMode} onOpenModels={() => setOverlay("models")} onOpenPalette={() => setOverlay("palette")} onOpenWorkspace={props.onOpenWorkspace} onSend={send} project={props.activeProject} />}
      {view === "settings" && <FocusSettings activeRun={props.activeRun} desktopShell={props.desktopShell} mode={mode} modelSettings={props.modelSettings} onModeChange={setMode} onRefreshModels={props.onRefreshModels} onSelectModel={props.onSelectModel} project={props.activeProject} threads={visibleThreads} />}

      {overlay === "palette" && <FocusCommandPalette onArchiveActive={props.onArchiveActive} onClose={() => setOverlay(undefined)} onOpenModels={() => setOverlay("models")} onOpenThreads={() => setOverlay("threads")} onOpenWorkspace={props.onOpenWorkspace} onRunCommand={props.onRunCommand} onView={(next) => setView(next)} />}
      {overlay === "threads" && <FocusThreadsMenu activeThreadId={props.activeThread?.id} onClose={() => setOverlay(undefined)} onNewThread={() => setView("home")} onSelectThread={(threadId) => { props.onSelectThread(threadId); setView("thread"); }} threads={visibleThreads} />}
      {overlay === "models" && <FocusModelMenu modelSettings={props.modelSettings} onClose={() => setOverlay(undefined)} onRefreshModels={props.onRefreshModels} onSelectModel={props.onSelectModel} onSelectQaqc={props.onSelectQaqc} qaqcAdapterId={props.qaqcAdapterId} />}
    </div>
  );
}

function stepForMode(mode: FocusMode) {
  return mode === "explore" ? 0 : mode === "plan" ? 1 : mode === "build" ? 2 : mode === "test" ? 3 : 4;
}
