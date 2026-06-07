import { useMemo, useState } from "react";

import type { WorkspaceProject, WorkspaceUiState } from "./workspaceTypes";

interface WorkspaceOverlayProps {
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

  if (!open) {
    return null;
  }

  return (
    <div aria-modal="true" className="workspace-backdrop" role="dialog">
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
            <h3>Project health</h3>
            <dl>
              <InfoRow label="Approved root" value={project.approvedRoots[0]} />
              <InfoRow label="Git branch" value={project.git.isRepo ? project.git.branch : "not a repo"} />
              <InfoRow label="Uncommitted" value={gitChangesLabel(project)} />
              <InfoRow label="Isolation" value={`${project.isolation.label}: ${project.isolation.detail}`} />
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
