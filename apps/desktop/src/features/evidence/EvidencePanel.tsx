import { FileCheck2 } from "lucide-react";

import { Badge } from "../../design-system/Badge";
import { StateBlock } from "../../design-system/StateBlock";
import type { EvidenceViewModel } from "../../app/types";

export function EvidencePanel({ evidence }: { evidence: EvidenceViewModel[] }) {
  return (
    <div className="evidence-panel" data-testid="evidence-panel">
      {evidence.length === 0 && (
        <StateBlock
          detail="No evidence records are attached to this answer."
          title="Insufficient evidence"
          tone="warning"
        />
      )}
      {evidence.map((item) => (
        <article className="receipt-card" key={item.id}>
          <header>
            <FileCheck2 size={16} />
            <strong>{item.title}</strong>
            <Badge tone="info">{item.sourceKind}</Badge>
          </header>
          <p>{item.detail}</p>
          <footer>{item.relationship}</footer>
        </article>
      ))}
    </div>
  );
}
