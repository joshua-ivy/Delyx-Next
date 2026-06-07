import { AlertTriangle, CircleDashed, FileText } from "lucide-react";

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

type ThreadStateTone = "neutral" | "warning" | "danger" | "success";

interface VisibleThreadState {
  detail: string;
  title: string;
  tone: ThreadStateTone;
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
  const visibleStates: Record<ThreadSummary["status"], VisibleThreadState> = {
    blocked: {
      detail: "A required approval, model, file scope, or workspace condition is blocking progress.",
      title: "Blocked",
      tone: "danger",
    },
    building: {
      detail: "Approved build work is in progress and must surface diffs before review.",
      title: "Building",
      tone: "neutral",
    },
    done: {
      detail: "This thread completed with evidence receipts and no hidden follow-up work.",
      title: "Done",
      tone: "success",
    },
    exploring: {
      detail: "Read-only file and project exploration is active.",
      title: "Exploring",
      tone: "neutral",
    },
    failed: {
      detail: "A run failed. The failure must stay visible with its artifact or reason.",
      title: "Failed",
      tone: "danger",
    },
    idle: {
      detail: "No AgentRun is active for this thread yet.",
      title: "Idle",
      tone: "neutral",
    },
    planning: {
      detail: "A plan is being prepared before any risky action runs.",
      title: "Planning",
      tone: "neutral",
    },
    reviewing: {
      detail: "Diffs, findings, test artifacts, and evidence are under review.",
      title: "Reviewing",
      tone: "neutral",
    },
    testing: {
      detail: "Approved test commands are running or ready to capture artifacts.",
      title: "Testing",
      tone: "neutral",
    },
    waiting_for_approval: {
      detail: "A risky action is waiting for explicit approval before execution.",
      title: "Waiting for approval",
      tone: "warning",
    },
  };
  const state = visibleStates[status];
  return (
    <StateBlock
      action={status === "waiting_for_approval" ? (
        <Button icon={<AlertTriangle size={16} />}>Review approval</Button>
      ) : undefined}
      detail={state.detail}
      title={state.title}
      tone={state.tone}
    />
  );
}
