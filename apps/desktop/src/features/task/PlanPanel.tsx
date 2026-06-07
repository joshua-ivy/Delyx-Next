import { Check, FileSearch, ShieldAlert } from "lucide-react";
import type { ReactNode } from "react";

import { Badge } from "../../design-system/Badge";
import { Panel } from "../../design-system/Panel";
import type { PlanViewModel } from "../../app/types";

export function PlanPanel({ plan }: { plan: PlanViewModel }) {
  return (
    <Panel
      action={<Badge tone="warning">Approval required</Badge>}
      eyebrow="Plan"
      title={plan.goal}
    >
      <p className="plan-summary">{plan.understanding}</p>
      <div className="plan-grid">
        <PlanList icon={<FileSearch size={16} />} items={plan.files} title="Likely files" />
        <PlanList icon={<Check size={16} />} items={plan.steps} title="Steps" />
        <PlanList icon={<ShieldAlert size={16} />} items={plan.risks} title="Risks" />
        <PlanList icon={<Check size={16} />} items={plan.tests} title="Tests" />
      </div>
      <div className="permission-strip">
        {plan.permissions.map((permission) => (
          <span key={permission}>{permission}</span>
        ))}
      </div>
    </Panel>
  );
}

function PlanList({ icon, items, title }: { icon: ReactNode; items: string[]; title: string }) {
  return (
    <section className="plan-list">
      <h3>
        {icon}
        {title}
      </h3>
      <ul>
        {items.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </section>
  );
}
