import { Fragment, useEffect, useState } from "react";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import type { TaskThread } from "../features/threads/threadTypes";
import { FocusIcon, Pipe, Think } from "./focusAtoms";
import { focusModes, latestRunEvent, modeLabel, modeStep, planProgress, runStatusLabel, type FocusMode } from "./focusFormat";
import { FocusDiffPeek } from "./FocusDiffPeek";
import { MarkdownMessage } from "./focusMarkdown";
import { FocusActionLine, FocusOutcomePeek, FocusSchedulerPeek, FocusTestPeek } from "./FocusThreadArtifacts";

interface FocusThreadProps {
  activePlan: PlanView | undefined;
  mode: FocusMode;
  model: string;
  onApplyPatch: (patchId: string) => void;
  onApprovePlan: () => void;
  onDecideProposal: (proposalId: string, status: "approved" | "denied") => void;
  onModeChange: (mode: FocusMode) => void;
  onOpenPalette: () => void;
  onRecordFinal: () => void;
  onResumeRun: () => void;
  onRunReview: () => void;
  onRunTests: () => void;
  onSend: (value: string) => void;
  patches: PatchProposalView[];
  proposals: ActionProposalView[];
  reviews: ReviewReportView[];
  run: AgentRunView | undefined;
  schedulerDecision: AgentScheduleDecisionView | undefined;
  tests: TestArtifactView[];
  thread: TaskThread;
}

export function FocusThread(props: FocusThreadProps) {
  const [value, setValue] = useState("");
  const pending = props.proposals.filter((proposal) => proposal.status === "pending");
  const send = () => {
    const trimmed = value.trim();
    if (trimmed) {
      setValue("");
      props.onSend(trimmed);
    }
  };
  return (
    <div className="stage" data-mode={props.mode} data-screen-label="Active thread">
      <div className="strip">
        <div className="name"><strong>delyx-next</strong> / {props.thread.status}</div>
        <div className="right">
          <span className="gchip"><span className={`dot ${statusTone(props.run?.status)}`} />{runStatusLabel(props.run, props.proposals)}</span>
          <button className="gchip" onClick={props.onOpenPalette} type="button">{props.model || "No model selected"}</button>
        </div>
      </div>

      <div className="work">
        <div className="work-scroll">
          <div className="wrap">
            <ThreadHeader mode={props.mode} model={props.model} run={props.run} thread={props.thread} />
            <ThreadTimeline messages={props.thread.messages} mode={props.mode} run={props.run} />
            <FocusSchedulerPeek decision={props.schedulerDecision} onApplyPatch={props.onApplyPatch} onRecordFinal={props.onRecordFinal} onResumeRun={props.onResumeRun} onRunReview={props.onRunReview} onRunTests={props.onRunTests} />
            <PlanBlock activePlan={props.activePlan} onApprovePlan={props.onApprovePlan} />
            <ApprovalBlock onDecideProposal={props.onDecideProposal} proposals={pending} />
            <FocusDiffPeek onPatchAction={props.onApplyPatch} patches={props.patches} proposals={props.proposals} run={props.run} />
            <FocusTestPeek activePlan={props.activePlan} onRunTests={props.onRunTests} patches={props.patches} tests={props.tests} />
            <ReviewPeek onRunReview={props.onRunReview} patches={props.patches} reports={props.reviews} tests={props.tests} />
            <FocusOutcomePeek canRecord={hasAssistantSummary(props.thread)} onRecordFinal={props.onRecordFinal} run={props.run} tests={props.tests} />
          </div>
        </div>
        <div className="dock">
          <div className="bigcomp">
            <textarea
              className="in"
              onChange={(event) => setValue(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  send();
                }
              }}
              placeholder="Steer the run, or send a follow-up..."
              rows={1}
              value={value}
            />
            <div className="ctl">
              <button className="icon-btn" title="Open commands" type="button" onClick={props.onOpenPalette}><FocusIcon name="plus" /></button>
              <div className="seg">{focusModes.slice(0, 3).map((item) => <button className={props.mode === item ? "on" : ""} key={item} onClick={() => props.onModeChange(item)} type="button">{modeLabel(item)}</button>)}</div>
              <button className="btn-send" onClick={send} type="button">Send <span className="kbd">Enter</span></button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ThreadTimeline({ messages, mode, run }: { messages: TaskThread["messages"]; mode: FocusMode; run: AgentRunView | undefined }) {
  const latestUserIndex = latestIndex(messages, (message) => message.role === "user");
  if (messages.length === 0) {
    return <RunActivity run={run} />;
  }
  return (
    <>
      {latestUserIndex === -1 && <RunActivity run={run} />}
      {messages.map((message, index) => (
        <Fragment key={`${index}-${message.role}`}>
          <MessageBlock message={message} mode={mode} />
          {index === latestUserIndex && <RunActivity run={run} />}
        </Fragment>
      ))}
    </>
  );
}

function latestIndex<T>(items: T[], matches: (item: T) => boolean) {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    if (matches(items[index])) {
      return index;
    }
  }
  return -1;
}

function ThreadHeader({ mode, model, run, thread }: { mode: FocusMode; model: string; run: AgentRunView | undefined; thread: TaskThread }) {
  return (
    <div className="thread-head">
      <Pipe active={modeStep(mode)} label={`thread / ${thread.id}`} />
      <h1 className="disp">{thread.title}</h1>
      <div className="goal">Local thread / {run?.metrics.eventCount ?? 0} event(s) / {model || "no model selected"}</div>
    </div>
  );
}

function RunActivity({ run }: { run: AgentRunView | undefined }) {
  if (!run) {
    return null;
  }
  const live = isLiveStatus(run.status);
  return (
    <div aria-live="polite" className={`focus-activity${live ? " is-live" : ""}`}>
      <span className={`dot ${statusTone(run.status)}`} />
      <b>{statusTitle(run.status)}</b>
      <LiveRunText run={run} />
      {live && <Think />}
    </div>
  );
}

function LiveRunText({ run }: { run: AgentRunView }) {
  const [tick, setTick] = useState(0);
  const messages = runLiveMessages(run);
  const live = isLiveStatus(run.status);

  useEffect(() => {
    setTick(0);
  }, [run.id, run.status, run.updatedAt]);

  useEffect(() => {
    if (!live || messages.length < 2) {
      return undefined;
    }
    const timer = window.setInterval(() => setTick((current) => current + 1), 1400);
    return () => window.clearInterval(timer);
  }, [live, messages.length, run.id, run.status]);

  return <span className="focus-live-copy">{messages[tick % messages.length]}</span>;
}

function MessageBlock({ message }: { message: TaskThread["messages"][number]; mode: FocusMode }) {
  const isUser = message.role === "user";
  return (
    <div className={`msg ${isUser ? "user" : "delyx"}`}>
      <div className="av">{isUser ? "YOU" : message.role === "assistant" ? "D" : "SYS"}</div>
      <div className="body">
        <div className="who">{isUser ? "You" : message.role === "assistant" ? "Delyx" : "System"}</div>
        <div className="txt">{isUser ? message.body : <MarkdownMessage text={message.body} />}</div>
      </div>
    </div>
  );
}

function PlanBlock({ activePlan, onApprovePlan }: { activePlan: PlanView | undefined; onApprovePlan: () => void }) {
  if (!activePlan) {
    return null;
  }
  const approved = activePlan.decision === "approved";
  return (
    <div className="plan">
      <div className="plan-head"><span className="ey">Plan / {activePlan.decision.replaceAll("_", " ")}</span><Think /></div>
      {planProgress(activePlan, approved).map((step, index) => (
        <div className={`pstep ${step.state}`} key={`${index}-${step.label}`}>
          <span className="pstep-n">{step.state === "done" ? <FocusIcon name="check" /> : index + 1}</span>{step.label}
        </div>
      ))}
      {!approved && <div className="plan-actions"><button className="select" onClick={onApprovePlan} type="button">Queue approval</button></div>}
    </div>
  );
}

function ApprovalBlock({ onDecideProposal, proposals }: { onDecideProposal: (proposalId: string, status: "approved" | "denied") => void; proposals: ActionProposalView[] }) {
  if (proposals.length === 0) {
    return null;
  }
  return <>{proposals.map((proposal) => <div className="plan approval-focus" key={proposal.id}>
    <div className="plan-head"><span className="ey">Approval / {proposal.riskLabel} risk</span><FocusIcon name="shield" /></div>
    <div className="approval-copy"><b>{proposal.actionType}</b><span>{proposal.rationale}</span><span>{proposal.expectedResult}</span></div>
    <div className="plan-actions"><button className="select" onClick={() => onDecideProposal(proposal.id, "approved")} type="button">Approve once</button><button className="select danger" onClick={() => onDecideProposal(proposal.id, "denied")} type="button">Deny</button></div>
  </div>)}</>;
}

function ReviewPeek({ onRunReview, patches, reports, tests }: { onRunReview: () => void; patches: PatchProposalView[]; reports: ReviewReportView[]; tests: TestArtifactView[] }) {
  const report = reports[0];
  const canRun = patches.length > 0 || tests.length > 0;
  if (!report && !canRun) {
    return null;
  }
  if (!report) {
    return <FocusActionLine icon="doc" label="Run review" onClick={onRunReview} text={`${patches.length} diff artifact(s), ${tests.length} test artifact(s)`} />;
  }
  const finding = report.findings[0];
  return <div className="peek">
    <div className="peek-head"><FocusIcon name="doc" /> Review / {report.decision}<span className="stat">{report.findings.length} finding(s)</span></div>
    <div className="approval-copy"><b>{finding?.title ?? report.riskSummary}</b><span>{finding?.detail ?? report.testSummary}</span><span>{report.evidenceSummary}</span></div>
    {canRun && <div className="plan-actions"><button className="select" onClick={onRunReview} type="button">Refresh review</button></div>}
  </div>;
}

function statusTone(status: AgentRunView["status"] | undefined) {
  if (status === "succeeded") {
    return "success";
  }
  if (status === "failed" || status === "blocked" || status === "cancelled") {
    return "danger";
  }
  return status === "waiting_for_approval" ? "warning" : "accent";
}

function statusTitle(status: AgentRunView["status"]) {
  const text = status.replaceAll("_", " ");
  return text.charAt(0).toUpperCase() + text.slice(1);
}

function isLiveStatus(status: AgentRunView["status"]) {
  return status === "created" || status === "running" || status === "repairing" || status === "waiting_for_approval";
}

function runLiveMessages(run: AgentRunView) {
  if (!isLiveStatus(run.status)) {
    return [latestRunEvent(run)];
  }
  const recent = run.events.slice(-3).map((event) => event.message).filter(Boolean);
  const defaults: Record<AgentRunView["status"], string[]> = {
    blocked: [],
    cancelled: [],
    created: ["Starting the local run"],
    failed: [],
    repairing: ["Repairing from the last failed step"],
    running: ["Thinking through the latest instruction", "Waiting on the local model response"],
    succeeded: [],
    waiting_for_approval: ["Waiting for your approval"],
  };
  return unique([...defaults[run.status], ...recent]);
}

function unique(items: string[]) {
  return items.filter((item, index) => item.trim().length > 0 && items.indexOf(item) === index);
}

function hasAssistantSummary(thread: TaskThread) {
  return thread.messages.some((message) => message.role === "assistant" && message.body.trim());
}
