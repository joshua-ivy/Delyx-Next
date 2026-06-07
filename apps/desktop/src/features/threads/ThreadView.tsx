import { AlertTriangle, CheckCircle2, CircleDashed, FileText } from "lucide-react";

import { Badge } from "../../design-system/Badge";
import { Button } from "../../design-system/Button";
import { Panel } from "../../design-system/Panel";
import { StateBlock } from "../../design-system/StateBlock";
import { StatusPill } from "../../design-system/StatusPill";
import { PlanPanel } from "../task/PlanPanel";
import { StepTimeline } from "../task/StepTimeline";
import type { PlanViewModel, ThreadSummary, TimelineItem } from "../../app/types";

interface ThreadViewProps {
  thread: ThreadSummary;
  plan: PlanViewModel;
  timeline: TimelineItem[];
}

export function ThreadView({ plan, thread, timeline }: ThreadViewProps) {
  return (
    <section className="thread-view">
      <Panel
        action={<StatusPill status={thread.status} />}
        eyebrow={`Mode: ${thread.mode}`}
        title={thread.title}
      >
        <p className="goal">{thread.goal}</p>
        <div className="thread-metrics">
          <Badge tone="info">{thread.changedFilesCount} changed files</Badge>
          <Badge tone={thread.pendingApprovalsCount > 0 ? "warning" : "success"}>
            {thread.pendingApprovalsCount} pending approvals
          </Badge>
        </div>
      </Panel>

      <StateGallery status={thread.status} />
      <PlanPanel plan={plan} />
      <StepTimeline items={timeline} />

      <Panel title="Composer">
        <div className="composer">
          <FileText size={18} />
          <span>Ask Delyx to revise the plan, inspect a file, or continue after approval.</span>
          <Button icon={<CircleDashed size={16} />}>Queue follow-up</Button>
        </div>
      </Panel>
    </section>
  );
}

function StateGallery({ status }: { status: ThreadSummary["status"] }) {
  if (status === "blocked") {
    return (
      <StateBlock
        detail="Provider setup needs a configured model before this task can continue."
        title="Blocked"
        tone="danger"
      />
    );
  }

  if (status === "failed") {
    return (
      <StateBlock
        detail="A previous smoke run failed. The failure is visible in the test panel."
        title="Failed"
        tone="danger"
      />
    );
  }

  if (status === "done") {
    return (
      <StateBlock
        detail="This thread completed with evidence receipts and no hidden follow-up work."
        title="Done"
        tone="success"
      />
    );
  }

  return (
    <StateBlock
      action={<Button icon={<AlertTriangle size={16} />}>Review approval</Button>}
      detail="A risky action is waiting for explicit approval before execution."
      title="Waiting for approval"
      tone="warning"
    />
  );
}
