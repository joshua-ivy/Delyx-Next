import { Clock, ShieldAlert } from "lucide-react";

import { Badge } from "../../design-system/Badge";
import type { ApprovalViewModel, RiskLevel } from "../../app/types";

const riskTone: Record<RiskLevel, "info" | "warning" | "danger"> = {
  low: "info",
  medium: "warning",
  high: "danger",
  dangerous: "danger",
};

export function ApprovalList({ approvals }: { approvals: ApprovalViewModel[] }) {
  return (
    <div className="approval-list" data-testid="approval-panel">
      {approvals.map((approval) => (
        <article className="approval-card" key={approval.id}>
          <header>
            <ShieldAlert size={16} />
            <strong>{approval.action}</strong>
            <Badge tone={riskTone[approval.risk]}>{approval.risk}</Badge>
          </header>
          <p>{approval.reason}</p>
          <dl>
            <div>
              <dt>Scope</dt>
              <dd>{approval.scope}</dd>
            </div>
            <div>
              <dt>Expected</dt>
              <dd>{approval.expectedResult}</dd>
            </div>
          </dl>
          <footer>
            <Clock size={14} />
            <span>{approval.status}</span>
            <span>{approval.expiresAt}</span>
          </footer>
        </article>
      ))}
    </div>
  );
}
