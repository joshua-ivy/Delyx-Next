import { useMemo, useState } from "react";

import { currentModelSettings } from "../models/modelData";
import type { AgentRunView } from "../runs/agentRunTypes";
import type { WorkspaceProject, WorkspaceUiState } from "./workspaceTypes";

interface WorkspaceOverlayProps {
  activeThreadCount: number;
  lastRun: AgentRunView | undefined;
  open: boolean;
  project: WorkspaceProject;
  projects: WorkspaceProject[];
  state: WorkspaceUiState;
  onAddProject: (path: string) => void;
  onClose: () => void;
  onRemoveProject: (projectId: string) => void;
  onShowDenied: () => void;
  onSimulateError: () => void;
  onSimulateLoading: () => void;
}

export function WorkspaceOverlay({
  activeThreadCount,
  lastRun,
  onAddProject,
  onClose,
  onRemoveProject,
  onShowDenied,
  onSimulateError,
  onSimulateLoading,
  open,
  project,
  projects,
  state,
}: WorkspaceOverlayProps) {
  const [path, setPath] = useState(project.path);
  const [query, setQuery] = useState("policy");
  const results = useMemo(
    () => project.indexedFiles.filter((file) => file.toLowerCase().includes(query.toLowerCase())),
    [project.indexedFiles, query],
  );
  const pinnedProjects = projects.filter((item) => item.pinned);

  if (!open) {
    return null;
  }

  return (
    <div aria-label="Workspace manager" aria-modal="true" className="workspace-backdrop" role="dialog">
      <section className="workspace-modal">
        <header>
          <div>
            <p>Workspace manager</p>
            <h2>{project.name}</h2>
          </div>
          <button onClick={onClose} type="button">Close</button>
        </header>

        <div className="workspace-grid">
          <section className="workspace-card">
            <h3>Add project</h3>
            <input aria-label="Project path" onChange={(event) => setPath(event.target.value)} value={path} />
            <div className="workspace-actions">
              <button onClick={() => onAddProject(path)} type="button">Add project</button>
              <button onClick={() => onRemoveProject(project.id)} type="button">Remove active</button>
            </div>
            <StateNotice state={state} />
          </section>

          <section className="workspace-card">
            <h3>Recent projects</h3>
            <ul className="workspace-projects">
              {projects.map((item) => (
                <li className={item.id === project.id ? "active" : ""} key={item.id}>
                  <strong>{item.name}</strong>
                  <span>{item.path}</span>
                  <small>{item.pinned ? "Pinned project" : "Recent local project"} · {item.lastOpenedLabel}</small>
                </li>
              ))}
              {projects.length === 0 && <li className="workspace-empty-row">No recent projects linked.</li>}
            </ul>
            <p className="workspace-meta">Pinned projects: {pinnedProjects.length === 0 ? "none" : pinnedProjects.map((item) => item.name).join(", ")}</p>
          </section>

          <section className="workspace-card">
            <h3>Project health</h3>
            <dl>
              <InfoRow label="Approved root" value={project.approvedRoots[0]} />
              <InfoRow label="Git branch" value={project.git.isRepo ? project.git.branch : "not a repo"} />
              <InfoRow label="Uncommitted" value={gitChangesLabel(project)} />
              <InfoRow label="Isolation" value={`${project.isolation.label}: ${project.isolation.detail}`} />
              <InfoRow label="Model profile" value={modelProfileLabel()} />
              <InfoRow label="Last run status" value={lastRunStatusLabel(lastRun)} />
              <InfoRow label="Active threads" value={`${activeThreadCount}`} />
              <InfoRow label="Approval policy" value={project.approvalPolicy} />
              <InfoRow label="Rules files" value={project.rulesFiles.map((file) => file.path).join(", ") || "none"} />
            </dl>
          </section>

          <section className="workspace-card">
            <h3>Read-only search</h3>
            <input aria-label="Search files" onChange={(event) => setQuery(event.target.value)} value={query} />
            <ul className="workspace-results">
              {results.map((file) => (
                <li key={file}>{file}</li>
              ))}
              {results.length === 0 && <li>No indexed files match this query.</li>}
            </ul>
          </section>

          <section className="workspace-card">
            <h3>Safety states</h3>
            <div className="workspace-actions">
              <button onClick={onShowDenied} type="button">Show denied read</button>
              <button onClick={onSimulateLoading} type="button">Loading state</button>
              <button onClick={onSimulateError} type="button">Error state</button>
            </div>
            <p>{projects.length} project linked to this local workspace.</p>
          </section>
        </div>
      </section>
    </div>
  );
}

function gitChangesLabel(project: WorkspaceProject) {
  if (!project.git.isRepo) {
    return "not a repo";
  }
  return project.git.uncommittedChanges === null ? "changes not loaded" : `${project.git.uncommittedChanges}`;
}

function modelProfileLabel() {
  const provider = currentModelSettings.providers.find((item) => item.id === currentModelSettings.selectedProviderId);
  const codingRoute = currentModelSettings.routes.find((route) => route.role === "coding");
  if (!provider) {
    return "No provider selected";
  }
  const route = codingRoute ? ` · coding ${codingRoute.modelId}` : "";
  return `${provider.label} · ${provider.status}${route}`;
}

function lastRunStatusLabel(run: AgentRunView | undefined) {
  return run ? `${run.status} · ${run.id}` : "No AgentRun ledger entries";
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function StateNotice({ state }: { state: WorkspaceUiState }) {
  const messages: Record<WorkspaceUiState, string> = {
    denied: "Denied: attempted read outside approved workspace root.",
    empty: "Empty: no projects are currently linked.",
    error: "Error: project path could not be added in this preview.",
    loading: "Loading: indexing approved workspace files.",
    ready: "Ready: project scope is approved and read-only indexing is available.",
  };

  return <p className={`workspace-state workspace-${state}`}>{messages[state]}</p>;
}
