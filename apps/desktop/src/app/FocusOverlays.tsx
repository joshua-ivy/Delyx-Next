import { useState, type ReactNode } from "react";
import type { ModelProviderView, ModelSettingsView } from "../features/models/modelTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import { FocusIcon, type FocusIconName } from "./focusAtoms";
import { selectedModel, selectedProvider } from "./focusFormat";

type FocusView = "home" | "thread" | "settings";

interface OverlayProps {
  children: ReactNode;
  onClose: () => void;
  position?: "top" | "center";
}

export function Scrim({ children, onClose, position = "top" }: OverlayProps) {
  return <div className={`scrim ${position}`} onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>{children}</div>;
}

export function FocusCommandPalette({
  onClose,
  onArchiveActive,
  onOpenModels,
  onOpenThreads,
  onOpenWorkspace,
  onRunCommand,
  onView,
}: {
  onArchiveActive: () => void;
  onClose: () => void;
  onOpenModels: () => void;
  onOpenThreads: () => void;
  onOpenWorkspace: () => void;
  onRunCommand: (commandId: string) => void;
  onView: (view: FocusView) => void;
}) {
  const [query, setQuery] = useState("");
  const items: PaletteItem[] = [
    { detail: "New centered composer", icon: "home", label: "Go home", run: () => onView("home") },
    { detail: "Switch or browse local threads", icon: "threads", label: "Threads", run: onOpenThreads, shortcut: "Ctrl P" },
    { detail: "Local preferences and runtime state", icon: "settings", label: "Open settings", run: () => onView("settings"), shortcut: "Ctrl ," },
    { detail: "Hide the current thread from active lists", icon: "threads", label: "Archive active thread", run: onArchiveActive },
    { detail: "Ask Ollama for a read-only plan", icon: "plan", label: "Create plan", run: () => onRunCommand("plan.create") },
    { detail: "Queue a scoped build approval proposal", icon: "shield", label: "Approve plan", run: () => onRunCommand("plan.approve") },
    { detail: "Refresh 127.0.0.1:11434", icon: "cpu", label: "Refresh Ollama models", run: () => onRunCommand("models.ollama.refresh") },
    { detail: "Choose from real local models", icon: "cpu", label: "Choose model", run: onOpenModels },
    { detail: "Open approved roots and Git facts", icon: "git", label: "Open workspace", run: onOpenWorkspace },
  ];
  const visible = items.filter((item) => `${item.label} ${item.detail}`.toLowerCase().includes(query.toLowerCase()));
  return (
    <Scrim onClose={onClose}>
      <div className="palette" data-screen-label="Command palette">
        <div className="pal-input"><FocusIcon name="search" /><input autoFocus onChange={(event) => setQuery(event.target.value)} placeholder="Type a command, or ask Delyx to steer the run..." value={query} /><span className="kbd-mini">esc</span></div>
        <div className="pal-list">
          <div className="pal-group">Commands</div>
          {visible.map((item, index) => <button className={`pal-item${index === 0 ? " on" : ""}`} key={item.label} onClick={() => { item.run(); onClose(); }} type="button">
            <span className="pic"><FocusIcon name={item.icon} /></span>
            <span className="pl">{item.label}<small>{item.detail}</small></span>
            {item.shortcut && <span className="pk">{item.shortcut}</span>}
          </button>)}
        </div>
        <div className="pal-foot mono"><span><b>Enter</b> run</span><span><b>Esc</b> close</span></div>
      </div>
    </Scrim>
  );
}

export function FocusThreadsMenu({
  activeThreadId,
  onClose,
  onNewThread,
  onSelectThread,
  threads,
}: {
  activeThreadId: string | undefined;
  onClose: () => void;
  onNewThread: () => void;
  onSelectThread: (threadId: string) => void;
  threads: TaskThread[];
}) {
  const visible = threads.filter((thread) => !thread.archived);
  return <Scrim onClose={onClose}>
    <div className="menu" data-screen-label="Threads">
      <div className="menu-head"><div className="ey">delyx-next</div><h3 className="disp">Threads</h3></div>
      <div className="menu-list">
        <button className="menu-item" onClick={() => { onNewThread(); onClose(); }} type="button"><span className="mi-ic"><FocusIcon name="plus" /></span><span className="mi-tx"><b>New thread</b><span>Start with an instruction</span></span><span className="mi-meta">Ctrl N</span></button>
        {visible.map((thread) => <button className={`menu-item${thread.id === activeThreadId ? " on" : ""}`} key={thread.id} onClick={() => { onSelectThread(thread.id); onClose(); }} type="button"><span className="mi-ic"><FocusIcon name={threadIcon(thread.status)} /></span><span className="mi-tx"><b>{thread.title}</b><span>{thread.mode} / {thread.status}</span></span><span className="mi-meta">{thread.createdLabel}</span></button>)}
        {visible.length === 0 && <div className="menu-empty">No active threads in this project.</div>}
      </div>
    </div>
  </Scrim>;
}

export function FocusModelMenu({
  modelSettings,
  onClose,
  onRefreshModels,
  onSelectModel,
}: {
  modelSettings: ModelSettingsView;
  onClose: () => void;
  onRefreshModels: () => void;
  onSelectModel: (modelId: string) => void;
}) {
  const activeProvider = selectedProvider(modelSettings);
  const activeModel = selectedModel(modelSettings);
  const usable = modelSettings.providers.filter((item) => item.models.length > 0);
  return <Scrim onClose={onClose} position="center">
    <div className="menu" data-screen-label="Model picker">
      <div className="menu-head"><div className="ey">{usable.length} provider(s) ready</div><h3 className="disp">Choose a model</h3></div>
      <div className="menu-list">
        {usable.flatMap((item) => item.models.map((model) => {
          const isActive = item.id === activeProvider?.id && model === activeModel;
          return <button className={`menu-item${isActive ? " on" : ""}`} key={`${item.id}:${model}`} onClick={() => { onSelectModel(model); onClose(); }} type="button"><span className="mi-ic"><FocusIcon name="cpu" /></span><span className="mi-tx"><b>{modelTitle(item, model)}</b><span>{providerSubtitle(item)}</span></span>{isActive && <span className="mi-meta active">active</span>}</button>;
        }))}
        {usable.length === 0 && <button className="menu-item" onClick={onRefreshModels} type="button"><span className="mi-ic"><FocusIcon name="cpu" /></span><span className="mi-tx"><b>No models loaded</b><span>Refresh Ollama, or install a CLI / add a key in Settings.</span></span><span className="mi-meta">refresh</span></button>}
      </div>
    </div>
  </Scrim>;
}

function modelTitle(provider: ModelProviderView, model: string) {
  return provider.kind === "cli" ? provider.label : model;
}

function providerSubtitle(provider: ModelProviderView) {
  if (provider.kind === "cli") {
    return "Subscription CLI · prompts go off-device";
  }
  if (provider.kind === "ollama") {
    return "Local Ollama model";
  }
  return provider.label;
}

interface PaletteItem {
  detail: string;
  icon: FocusIconName;
  label: string;
  run: () => void;
  shortcut?: string;
}

function threadIcon(status: TaskThread["status"]): FocusIconName {
  if (status === "testing") {
    return "flask";
  }
  if (status === "done") {
    return "check";
  }
  if (status === "planning") {
    return "plan";
  }
  return "bolt";
}
