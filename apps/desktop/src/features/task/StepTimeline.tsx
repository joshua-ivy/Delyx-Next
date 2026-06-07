import { Circle, CircleCheck, CircleEllipsis, CircleX } from "lucide-react";

import { Panel } from "../../design-system/Panel";
import type { TimelineItem } from "../../app/types";

const icons = {
  active: CircleEllipsis,
  done: CircleCheck,
  failed: CircleX,
  waiting: Circle,
};

export function StepTimeline({ items }: { items: TimelineItem[] }) {
  return (
    <Panel eyebrow="Run ledger" title="Agent progress">
      <ol className="timeline">
        {items.map((item) => {
          const Icon = icons[item.status];

          return (
            <li className={`timeline-item timeline-${item.status}`} key={item.id}>
              <Icon size={17} />
              <div>
                <strong>{item.label}</strong>
                <p>{item.detail}</p>
              </div>
            </li>
          );
        })}
      </ol>
    </Panel>
  );
}
