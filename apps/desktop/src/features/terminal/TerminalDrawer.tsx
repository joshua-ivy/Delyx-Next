import { ScrollText } from "lucide-react";

import { Badge } from "../../design-system/Badge";
import type { TerminalBlockViewModel } from "../../app/types";

export function TerminalDrawer({ blocks, open }: { blocks: TerminalBlockViewModel[]; open: boolean }) {
  if (!open) {
    return null;
  }

  return (
    <aside className="terminal-drawer" aria-label="Terminal and logs">
      <header>
        <div>
          <ScrollText size={16} />
          <strong>Terminal / logs / external agent stream</strong>
        </div>
        <Badge tone="info">no commands run</Badge>
      </header>
      <div className="terminal-grid">
        {blocks.map((block) => (
          <section className="terminal-block" key={block.id}>
            <header>
              <span>{block.label}</span>
              <Badge tone={block.status === "failed" ? "danger" : "neutral"}>{block.status}</Badge>
            </header>
            <pre>
              {block.lines.map((line) => (
                <code key={line}>{line}</code>
              ))}
            </pre>
          </section>
        ))}
      </div>
    </aside>
  );
}
