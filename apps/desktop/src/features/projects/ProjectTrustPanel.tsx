import { useEffect, useState } from "react";
import { ensureProject, saveProject } from "./projectClient";
import type { ProjectSaveRequest, ProjectTrustLevel, ProjectView } from "./projectTypes";

const TRUST_LEVELS: { id: ProjectTrustLevel; label: string; detail: string }[] = [
  { id: "local", label: "Local", detail: "Code you own. Standard local-first trust." },
  { id: "restricted", label: "Restricted", detail: "Shared or sensitive tree — read/write with extra caution." },
  { id: "external", label: "External", detail: "Externally sourced — nothing trusted without approval." },
];

/**
 * Displays (and lets the user set) the native project's trust level and the file
 * scopes Delyx is allowed to read/write. The project record is the durable trust
 * boundary; this surfaces it so the user can see exactly what Delyx may touch.
 */
export function ProjectTrustPanel({ name, rootPath }: { name: string; rootPath: string }) {
  const [project, setProject] = useState<ProjectView | undefined>(undefined);
  const [status, setStatus] = useState<string | undefined>(undefined);
  const [unavailable, setUnavailable] = useState(false);

  useEffect(() => {
    let active = true;
    void ensureProject(name, rootPath)
      .then((record) => {
        if (active) setProject(record);
      })
      .catch(() => {
        if (active) setUnavailable(true);
      });
    return () => {
      active = false;
    };
  }, [name, rootPath]);

  async function changeTrust(trustLevel: ProjectTrustLevel) {
    if (!project || trustLevel === project.trustLevel) {
      return;
    }
    try {
      const updated = await saveProject({ ...toSaveRequest(project), trustLevel });
      setProject(updated);
      setStatus(`Trust level set to ${trustLevel}.`);
    } catch (cause) {
      setStatus(String(cause));
    }
  }

  if (unavailable) {
    return (
      <div className="set-sec">
        <div className="ey">Project trust &amp; scopes</div>
        <Row detail="Native projects need the Delyx desktop app." title="Desktop only">
          <span className="tag off">web preview</span>
        </Row>
      </div>
    );
  }

  if (!project) {
    return (
      <div className="set-sec">
        <div className="ey">Project trust &amp; scopes</div>
        <Row detail="Loading project trust state…" title="Project">
          <span className="tag off">…</span>
        </Row>
      </div>
    );
  }

  return (
    <div className="set-sec">
      <div className="ey">Project trust &amp; scopes</div>
      <div className="set-lead">The project is Delyx&apos;s trust boundary: trust level plus the exact paths Delyx may read or write. Nothing outside an allowed read scope is touched without a fresh approval.</div>
      <Row detail="How much Delyx trusts content in this project." title="Trust level">
        <div className="trust-options">
          {TRUST_LEVELS.map((level) => (
            <button
              className={`select${level.id === project.trustLevel ? " on" : ""}`}
              key={level.id}
              onClick={() => void changeTrust(level.id)}
              title={level.detail}
              type="button"
            >
              {level.label}
            </button>
          ))}
        </div>
      </Row>
      <Row detail={`Mode: ${project.approvalPolicy.mode} · large file ≥ ${formatBytes(project.approvalPolicy.largeFileBytes)} or > ${project.approvalPolicy.folderFileCount} files needs approval.`} title="Approval policy">
        <span className="tag live">{project.approvalPolicy.mode}</span>
      </Row>
      {project.allowedFileScopes.map((scope, index) => (
        <Row detail={scope.reason} key={`${scope.path}-${index}`} title={scope.path}>
          <span className={`tag ${scope.canRead ? "live" : "off"}`}>{scope.canRead ? "read" : "no read"}</span>
          <span className={`tag ${scope.canWrite ? "live" : "off"}`}>{scope.canWrite ? "write" : "no write"}</span>
          <span className="tag off">{scope.recursive ? "recursive" : "top-level"}</span>
        </Row>
      ))}
      {status && <Row detail={status} title="Status"><span className="tag live">ok</span></Row>}
    </div>
  );
}

function toSaveRequest(project: ProjectView): ProjectSaveRequest {
  return {
    id: project.id,
    name: project.name,
    rootPath: project.rootPath,
    trustLevel: project.trustLevel,
    allowedFileScopes: project.allowedFileScopes,
    approvalPolicy: project.approvalPolicy,
    modelPermissions: project.modelPermissions,
    toolPermissions: project.toolPermissions,
    memoryScope: project.memoryScope,
  };
}

function formatBytes(bytes: number): string {
  if (bytes >= 1_000_000) {
    return `${(bytes / 1_000_000).toFixed(1)} MB`;
  }
  if (bytes >= 1_000) {
    return `${Math.round(bytes / 1_000)} KB`;
  }
  return `${bytes} B`;
}

function Row({ children, detail, title }: { children: React.ReactNode; detail: string; title: string }) {
  return (
    <div className="row">
      <div className="rmeta">
        <b>{title}</b>
        {detail && <span>{detail}</span>}
      </div>
      <div className="rctl">{children}</div>
    </div>
  );
}
