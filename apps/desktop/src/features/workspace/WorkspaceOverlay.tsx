import { useEffect, useMemo, useState } from "react";
import { Check, ChevronDown, FolderOpen, GitBranch, Search, Shield, X } from "lucide-react";

import type { ModelSettingsView } from "../models/modelTypes";
import type { AgentRunView } from "../runs/agentRunTypes";
import type { WorkspaceProject, WorkspaceUiState } from "./workspaceTypes";

interface WorkspaceOverlayProps {
  activeThreadCount: number;
  lastRun: AgentRunView | undefined;
  modelSettings: ModelSettingsView;
  open: boolean;
  project: WorkspaceProject;
  projects: WorkspaceProject[];
  state: WorkspaceUiState;
  onAddProject: (path: string) => void;
  onClose: () => void;
  onRemoveProject: (projectId: string) => void;
}

export function WorkspaceOverlay({
  activeThreadCount,
  lastRun,
  modelSettings,
  onAddProject,
  onClose,
  onRemoveProject,
  open,
  project,
  projects,
  state,
}: WorkspaceOverlayProps) {
  const [path, setPath] = useState(project.path);
  const [switching, setSwitching] = useState(false);
  const query = "";
  const results = useMemo(
    () => project.indexedFiles.filter((file) => file.toLowerCase().includes(query)),
    [project.indexedFiles],
  );
  const pinnedProjects = projects.filter((item) => item.pinned);
  const modelProfile = modelProfileLabel(modelSettings);
  const indexLoaded = isIndexLoaded(project);

  useEffect(() => {
    setPath(project.path);
  }, [project.path]);

  if (!open) {
    return null;
  }

  return (
    <div aria-label="Workspace manager" aria-modal="true" className="workspace-backdrop" role="dialog">
      <section className="workspace-modal workspace-panel">
        <header className="workspace-head">
          <div>
            <p>Workspace</p>
            <h2>{state === "empty" ? "No project linked" : project.name}</h2>
          </div>
          <button aria-label="Close workspace manager" className="workspace-icon" onClick={onClose} title="Close" type="button"><X /></button>
        </header>

        <button aria-expanded={switching} className="workspace-project" onClick={() => setSwitching((current) => !current)} type="button">
          <span className="workspace-picture"><FolderOpen /></span>
          <span className="workspace-project-copy"><b>{project.name}</b><span>{project.path}</span></span>
          <span className={`workspace-ready workspace-${state}`}><span className={`dot ${stateTone(state)}`} />{stateLabel(state)}</span>
          <span className={`workspace-caret${switching ? " open" : ""}`}><ChevronDown /></span>
        </button>

        <div className="workspace-open">
          <label htmlFor="workspace-open-path">Open a project folder</label>
          <div className="workspace-open-row">
            <input
              autoComplete="off"
              id="workspace-open-path"
              onChange={(event) => setPath(event.target.value)}
              onKeyDown={(event) => { if (event.key === "Enter" && path.trim()) onAddProject(path.trim()); }}
              placeholder="Paste a folder path, e.g. C:\\Users\\you\\my-app"
              spellCheck={false}
              value={path}
            />
            <button
              className="workspace-button primary"
              disabled={state === "loading" || path.trim().length === 0}
              onClick={() => onAddProject(path.trim())}
              type="button"
            >
              {state === "loading" ? "Opening…" : "Open"}
            </button>
          </div>
          <p className="workspace-open-hint">Delyx reads this folder read-only as the project root. Like Claude Code / Codex, it's just a directory — no browser dialog needed.</p>
        </div>

        {switching && (
          <div className="workspace-switch">
            <div className="workspace-switch-head">Recent projects</div>
            <ul className="workspace-projects">
              {projects.map((item) => (
                <li className={item.id === project.id ? "active" : ""} key={item.id}>
                  <span className="dot accent" />
                  <span><b>{item.name}</b><small>{item.path}</small></span>
                </li>
              ))}
              {projects.length === 0 && <li className="workspace-empty-row">No recent projects linked.</li>}
            </ul>
            <div className="workspace-add">
              <input aria-label="Project path" onChange={(event) => setPath(event.target.value)} placeholder="Add a local project path..." value={path} />
              <button className="workspace-button" disabled={state === "loading" || path.trim().length === 0} onClick={() => onAddProject(path)} type="button">{state === "loading" ? "Loading" : "Add"}</button>
              <button className="workspace-button ghost" onClick={() => onRemoveProject(project.id)} type="button">Remove</button>
            </div>
            <p className="workspace-meta">Pinned projects: {pinnedProjects.length === 0 ? "none" : pinnedProjects.map((item) => item.name).join(", ")}</p>
          </div>
        )}

        <div className={`workspace-fix${indexLoaded ? " done" : ""}`}>
          <span className={`workspace-fix-icon${indexLoaded ? " ok" : ""}`}>{indexLoaded ? <Check /> : <GitBranch />}</span>
          <span className="workspace-fix-copy"><b>{indexTitle(project, indexLoaded, state)}</b><span>{indexDetail(project, indexLoaded, state)}</span></span>
          {!indexLoaded && (
            <button className="workspace-button primary" disabled={state === "loading"} onClick={() => onAddProject(project.path)} type="button">
              {state === "loading" ? <span className="workspace-spinner" /> : "Load index"}
            </button>
          )}
        </div>

        <div className="workspace-tiles">
          <InfoTile detail={modelProfile.detail} label="Model profile" title={modelProfile.title} tone={modelProfile.tone} />
          <InfoTile detail={lastRunStatusLabel(lastRun)} label="Active threads" title={`${activeThreadCount} active`} />
          <InfoTile detail={project.approvalPolicy} label="Approval policy" title="Writes and shell" />
        </div>

        <div className="workspace-search-line">
          <span><Search /> Read-only search</span>
          <b>{results.length === 0 ? "No indexed files are loaded for this query." : `${results.length} indexed file(s)`}</b>
        </div>

        <footer className="workspace-foot">
          <span><Shield /> Read policy: approved root only. Isolation: {project.isolation.label}. {dirtyCountPolicy(project)}</span>
          <span>Denied state renders when a workspace read is rejected by policy.</span>
        </footer>
      </section>
    </div>
  );
}

function isIndexLoaded(project: WorkspaceProject) {
  return project.indexedFiles.length > 0 || project.git.uncommittedChanges !== null || project.git.branch !== "branch not loaded";
}

function indexTitle(project: WorkspaceProject, loaded: boolean, state: WorkspaceUiState) {
  if (state === "loading") {
    return "Reading workspace index";
  }
  if (!project.git.isRepo) {
    return "Repository not detected";
  }
  return loaded ? `Indexed - ${project.git.branch}` : "Git index not loaded";
}

function indexDetail(project: WorkspaceProject, loaded: boolean, state: WorkspaceUiState) {
  if (state === "loading") {
    return "Read-only scan is staying inside the approved root.";
  }
  if (!project.git.isRepo) {
    return "Add a local Git project path before branch and change state can load.";
  }
  if (!loaded) {
    return "Branch, changes, rules, and search need a read-only workspace snapshot.";
  }
  return `${gitChangesLabel(project)} uncommitted change(s). Loaded from read-only Git index metadata.`;
}

function gitChangesLabel(project: WorkspaceProject) {
  if (!project.git.isRepo) {
    return "not a repo";
  }
  return project.git.uncommittedChanges === null ? "changes not loaded" : `${project.git.uncommittedChanges}`;
}

function dirtyCountPolicy(project: WorkspaceProject) {
  return project.git.uncommittedChanges === null
    ? "Unavailable until read-only Git index metadata exists."
    : "Loaded from read-only Git index metadata.";
}

function modelProfileLabel(settings: ModelSettingsView) {
  const provider = settings.providers.find((item) => item.id === settings.selectedProviderId);
  const route = settings.routes.find((item) => item.providerId === provider?.id && item.role === "coding");
  if (!provider) {
    return { detail: "No provider selected", title: "No provider", tone: "warning" };
  }
  return {
    detail: route ? `${provider.label} - coding ${route.modelId}` : provider.detail,
    title: provider.status === "ready" ? "Ready" : provider.status.replaceAll("_", " "),
    tone: provider.status === "ready" ? "success" : "warning",
  };
}

function lastRunStatusLabel(run: AgentRunView | undefined) {
  return run ? `Last run status: ${run.status.replaceAll("_", " ")}` : "Last run status: no runs yet";
}

function InfoTile({ detail, label, title, tone }: { detail: string; label: string; title: string; tone?: string }) {
  return (
    <div className="workspace-tile">
      <span>{label}</span>
      <b>{tone && <span className={`dot ${tone}`} />}{title}</b>
      <small>{detail}</small>
    </div>
  );
}

function stateLabel(state: WorkspaceUiState) {
  const labels: Record<WorkspaceUiState, string> = {
    denied: "Denied: attempted read outside approved workspace root.",
    empty: "Empty: no projects are currently linked.",
    error: "Error: project path could not be added in this preview.",
    loading: "Loading: indexing approved workspace files.",
    ready: "Ready: project scope is approved and read-only indexing is available.",
  };
  return labels[state].split(":")[0];
}

function stateTone(state: WorkspaceUiState) {
  return state === "ready" ? "success" : state === "denied" || state === "error" ? "danger" : "warning";
}
