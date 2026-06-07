import { Archive, FolderOpen, Settings, ShieldCheck, Sparkles } from "lucide-react";

import { Badge } from "../../design-system/Badge";
import { StatusPill } from "../../design-system/StatusPill";
import type { ProjectSummary, ThreadSummary } from "../../app/types";

interface ProjectSidebarProps {
  projects: ProjectSummary[];
  threads: ThreadSummary[];
  selectedThreadId: string;
  onSelectThread: (threadId: string) => void;
}

export function ProjectSidebar({ onSelectThread, projects, selectedThreadId, threads }: ProjectSidebarProps) {
  const project = projects[0];

  return (
    <aside className="sidebar">
      <section className="project-card">
        <FolderOpen size={18} />
        <div>
          <strong>{project.name}</strong>
          <p>{project.path}</p>
        </div>
      </section>

      <nav className="side-nav" aria-label="Primary">
        <a className="active" href="#threads">
          <Sparkles size={16} /> Threads
        </a>
        <a href="#skills">
          <ShieldCheck size={16} /> Skills
        </a>
        <a href="#memory">
          <Archive size={16} /> Memory
        </a>
        <a href="#settings">
          <Settings size={16} /> Settings
        </a>
      </nav>

      <section className="sidebar-section" id="threads">
        <div className="section-heading">
          <span>Task threads</span>
          <Badge tone="info">{threads.length}</Badge>
        </div>
        <div className="thread-list">
          {threads.map((thread) => (
            <button
              className={selectedThreadId === thread.id ? "thread-item active" : "thread-item"}
              key={thread.id}
              onClick={() => onSelectThread(thread.id)}
              type="button"
            >
              <span>{thread.title}</span>
              <small>{thread.updatedAt}</small>
              <StatusPill status={thread.status} />
            </button>
          ))}
        </div>
      </section>

      <section className="scope-card">
        <strong>Approved scope</strong>
        <p>{project.approvedRoots[0]}</p>
        <Badge tone="success">local only</Badge>
      </section>
    </aside>
  );
}
