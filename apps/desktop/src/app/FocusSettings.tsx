import { useState, type ReactNode } from "react";
import type { ModelSelectionKey, ModelSettingsView } from "../features/models/modelTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { FocusIcon } from "./focusAtoms";
import { FocusProviders } from "./FocusProviders";
import { gitChangeLabel, modeLabel, selectedModel, selectedProvider, type FocusMode } from "./focusFormat";
import type { DesktopShellStatusView } from "./runtimeBridge";

type SettingsTab = "general" | "models" | "providers" | "workspace" | "privacy" | "appearance" | "keys";

interface FocusSettingsProps {
  activeRun: AgentRunView | undefined;
  desktopShell: DesktopShellStatusView | undefined;
  mode: FocusMode;
  modelSettings: ModelSettingsView;
  onModeChange: (mode: FocusMode) => void;
  onRefreshModels: () => void;
  onSelectModel: (selection: ModelSelectionKey) => void;
  project: WorkspaceProject;
  threads: TaskThread[];
}

const tabs: [SettingsTab, string][] = [
  ["general", "General"],
  ["models", "Models & Providers"],
  ["providers", "Providers & Keys"],
  ["workspace", "Workspace"],
  ["privacy", "Privacy & Local Data"],
  ["appearance", "Appearance"],
  ["keys", "Keybindings"],
];

export function FocusSettings(props: FocusSettingsProps) {
  const [tab, setTab] = useState<SettingsTab>("general");
  const [compact, setCompact] = useState(false);
  return (
    <div className="stage" data-mode={props.mode} data-screen-label="Settings">
      <div className="strip"><div className="name"><strong>{props.project.name}</strong> / settings</div></div>
      <div className="settings">
        <div className="set-wrap">
          <div className="set-title disp">Settings</div>
          <div className="set-lead">Everything Delyx needs to run locally. Nothing leaves this machine unless you approve it.</div>
          <div className="set-tabs">{tabs.map(([id, label]) => <button className={`set-tab${tab === id ? " on" : ""}`} key={id} onClick={() => setTab(id)} type="button">{label}</button>)}</div>
          {tab === "general" && <General desktopShell={props.desktopShell} mode={props.mode} onModeChange={props.onModeChange} />}
          {tab === "models" && <Models modelSettings={props.modelSettings} onRefreshModels={props.onRefreshModels} onSelectModel={props.onSelectModel} />}
          {tab === "providers" && <FocusProviders />}
          {tab === "workspace" && <Workspace activeRun={props.activeRun} project={props.project} threads={props.threads} />}
          {tab === "privacy" && <Privacy />}
          {tab === "appearance" && <Appearance compact={compact} mode={props.mode} onCompact={() => setCompact((value) => !value)} onModeChange={props.onModeChange} />}
          {tab === "keys" && <Keys />}
        </div>
      </div>
    </div>
  );
}

function General({ desktopShell, mode, onModeChange }: { desktopShell: DesktopShellStatusView | undefined; mode: FocusMode; onModeChange: (mode: FocusMode) => void }) {
  return <Section label="Behaviour">
    <Row title="Default mode" detail="The mode new instructions visually start in."><SelectButton label={modeLabel(mode)} onClick={() => onModeChange(nextMode(mode))} /></Row>
    <Row title="Plan before build" detail="Use the real PlanAgent command before queueing risky actions."><span className="tag live">approval-gated</span></Row>
    <Row title="Runtime visibility" detail="Run events, approvals, diffs, tests, and receipts stay visible when present."><span className="tag live">on</span></Row>
    {desktopShell && <Row title="Windows shell" detail={desktopShellDetail(desktopShell)}><span className="tag live">desktop</span></Row>}
  </Section>;
}

function Models({ modelSettings, onRefreshModels, onSelectModel }: { modelSettings: ModelSettingsView; onRefreshModels: () => void; onSelectModel: (selection: ModelSelectionKey) => void }) {
  const provider = selectedProvider(modelSettings);
  const activeModel = selectedModel(modelSettings);
  return <>
    <Section label="Providers">
      {modelSettings.providers.map((item) => <div className={`provider${item.id === provider?.id ? " on" : ""}`} key={item.id}>
        <span className="pic"><FocusIcon name="cpu" /></span>
        <span className="pinfo"><b>{item.label}</b><span>{providerDetail(item)}</span></span>
        <span className={`tag ${item.status === "ready" ? "live" : "off"}`}>{item.status.replaceAll("_", " ")}</span>
      </div>)}
      <button className="select" onClick={onRefreshModels} type="button">Refresh Ollama</button>
    </Section>
    <Section label="Routes">
      {(provider?.models ?? []).map((model) => <Row key={model} title={model} detail="Local Ollama model">
        <button className="select" onClick={() => onSelectModel({ modelId: model, providerId: provider?.id ?? modelSettings.selectedProviderId })} type="button">{activeModel === model ? "Active" : "Use model"}</button>
      </Row>)}
      {(provider?.models.length ?? 0) === 0 && <Row title="No local models loaded" detail={provider?.detail ?? "Provider status has not loaded."}><span className="tag off">empty</span></Row>}
    </Section>
  </>;
}

function Workspace({ activeRun, project, threads }: { activeRun: AgentRunView | undefined; project: WorkspaceProject; threads: TaskThread[] }) {
  return <Section label="Repository">
    <Row title="Active project" detail={project.path}><span className="tag live">{project.name}</span></Row>
    <Row title="Git state" detail={gitChangeLabel(project)}><span className={`tag ${project.git.isRepo ? "live" : "off"}`}>{project.git.branch || "no repo"}</span></Row>
    <Row title="Approved root" detail={project.approvedRoots[0] ?? "No approved root loaded."}><span className="tag live">local</span></Row>
    <Row title="Active threads" detail={`${threads.filter((thread) => !thread.archived).length} visible thread(s)`}><span className="tag off">{activeRun?.status ?? "no run"}</span></Row>
  </Section>;
}

function providerDetail(provider: ModelSettingsView["providers"][number]) {
  return provider.version ? `${provider.detail} Version ${provider.version}.` : provider.detail;
}

function Privacy() {
  return <Section label="Local data">
    <Row title="SQLite local store" detail="Threads, runs, approvals, model routes, workspace snapshots, and receipt stores persist on-device."><span className="tag live">on-device</span></Row>
    <Row title="Risky actions" detail="File writes, commands, memory saves, connectors, schedules, and external agents require approval."><span className="tag live">gated</span></Row>
    <Row title="Secrets" detail="Secrets are not stored in the repo; redaction is used for support bundles."><span className="tag live">redacted</span></Row>
  </Section>;
}

function Appearance({ compact, mode, onCompact, onModeChange }: { compact: boolean; mode: FocusMode; onCompact: () => void; onModeChange: (mode: FocusMode) => void }) {
  return <Section label="Accent">
    <Row title="Agent mode accent" detail="Accent follows the current mode token.">
      <div className="swatches">{(["build", "explore", "plan", "test", "review"] as FocusMode[]).map((item) => <button className={`swatch ${item}${mode === item ? " on" : ""}`} key={item} onClick={() => onModeChange(item)} title={item} type="button" />)}</div>
    </Row>
    <Row title="Compact density" detail="A local visual preference for this session."><Toggle on={compact} onClick={onCompact} /></Row>
  </Section>;
}

function Keys() {
  return <Section label="Keyboard">
    {[
      ["Command palette", "Ctrl K"],
      ["Send instruction", "Enter"],
      ["New thread", "Ctrl N"],
      ["Switch thread", "Ctrl P"],
      ["Open settings", "Ctrl ,"],
      ["Close overlay", "Esc"],
    ].map(([label, keys]) => <Row key={label} title={label} detail=""><span className="kgroup">{keys.split(" ").map((key) => <span className="kcap" key={key}>{key}</span>)}</span></Row>)}
  </Section>;
}

function Section({ children, label }: { children: ReactNode; label: string }) {
  return <div className="set-sec"><div className="ey">{label}</div>{children}</div>;
}

function Row({ children, detail, title }: { children: ReactNode; detail: string; title: string }) {
  return <div className="row"><div className="rmeta"><b>{title}</b>{detail && <span>{detail}</span>}</div><div className="rctl">{children}</div></div>;
}

function SelectButton({ label, onClick }: { label: string; onClick: () => void }) {
  return <button className="select" onClick={onClick} type="button">{label}<FocusIcon name="down" /></button>;
}

function Toggle({ on, onClick }: { on: boolean; onClick: () => void }) {
  return <button aria-checked={on} className={`toggle${on ? " on" : ""}`} onClick={onClick} role="switch" type="button" />;
}

function desktopShellDetail(shell: DesktopShellStatusView) {
  const menu = shell.nativeMenuPolicy === "renderer_command_ui" ? "renderer commands" : shell.nativeMenuPolicy.replaceAll("_", " ");
  const reopen = shell.reopenBehavior === "single_instance_focus_main_window" ? "single instance" : shell.reopenBehavior.replaceAll("_", " ");
  const signing = shell.signingPolicy === "unsigned_dev_build" ? "unsigned dev build" : shell.signingPolicy.replaceAll("_", " ");
  return `${reopen}; ${menu}; ${signing}`;
}

function nextMode(mode: FocusMode): FocusMode {
  return mode === "explore" ? "plan" : mode === "plan" ? "build" : mode === "build" ? "test" : mode === "test" ? "review" : "explore";
}
