import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { AgentScheduleDecisionView } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import { FocusIcon } from "./focusAtoms";
import { firstRunnableTestCommand } from "./testCommand";

export function FocusSchedulerPeek({
  decision,
  onApplyPatch,
  onRecordFinal,
  onResumeRun,
  onRunReview,
  onRunTests,
}: {
  decision: AgentScheduleDecisionView | undefined;
  onApplyPatch: (patchId: string) => void;
  onRecordFinal: () => void;
  onResumeRun: () => void;
  onRunReview: () => void;
  onRunTests: () => void;
}) {
  if (!decision || decision.kind === "complete" || decision.kind === "terminal") {
    return null;
  }
  if (decision.kind === "run_patch_apply" && decision.proposalId) {
    return <FocusActionLine icon="plan" label="Next / apply patch" onClick={() => onApplyPatch(decision.proposalId!)} text={decision.message} />;
  }
  if (decision.kind === "run_patch_draft") {
    return <FocusActionLine icon="plan" label="Next / draft patch" onClick={onResumeRun} text={decision.message} />;
  }
  if (decision.kind === "run_tests") {
    return <FocusActionLine icon="flask" label="Next / run tests" onClick={onRunTests} text={decision.message} />;
  }
  if (decision.kind === "run_review") {
    return <FocusActionLine icon="doc" label="Next / run review" onClick={onRunReview} text={decision.message} />;
  }
  if (decision.kind === "ready_for_final_support") {
    return <FocusActionLine icon="doc" label="Next / final support" onClick={onRecordFinal} text={decision.message} />;
  }
  if (decision.kind === "repair_requested") {
    return <FocusActionLine icon="plan" label="Next / repair requested" text={decision.message} />;
  }
  if (decision.kind === "resume_after_approval") {
    return <FocusActionLine icon="plan" label="Next / resume run" onClick={onResumeRun} text={decision.message} />;
  }
  return <FocusActionLine icon="plan" label={schedulerLabel(decision.kind)} text={decision.message} />;
}

export function FocusTestPeek({
  activePlan,
  onRunTests,
  patches,
  tests,
}: {
  activePlan: PlanView | undefined;
  onRunTests: () => void;
  patches: PatchProposalView[];
  tests: TestArtifactView[];
}) {
  const test = tests[0];
  if (!test) {
    const command = firstRunnableTestCommand(activePlan?.testsToRun);
    if (!command || !patches.some((patch) => patch.status === "applied")) {
      return null;
    }
    return <FocusActionLine icon="flask" label="Run tests" onClick={onRunTests} text={command.label} />;
  }
  return <FocusActionLine icon="flask" label={`${test.command} / ${test.status ?? "captured"}`} text={test.failureSummary ?? `Exit ${test.exitCode ?? "unknown"}`} />;
}

export function FocusOutcomePeek({
  canRecord,
  onRecordFinal,
  run,
  reviews = [],
  tests,
}: {
  canRecord: boolean;
  onRecordFinal: () => void;
  run: AgentRunView | undefined;
  reviews?: ReviewReportView[];
  tests: TestArtifactView[];
}) {
  if (run?.outcome) {
    const evidence = run.outcome.evidenceRecordIds.length;
    const passedTests = run.outcome.testArtifactIds.length;
    const support = finalSupportState(evidence, passedTests);
    return <FocusActionLine icon="doc" label={`Final support / ${support.label(run.outcome.status)}`} text={support.text} />;
  }
  if (!run) {
    return null;
  }
  const blockedReview = unresolvedReview(reviews, run.id);
  if (blockedReview) {
    return <FocusActionLine icon="doc" label="Final support / review blocked" text={`Review ${blockedReview.id} has ${blockedReview.findings.length} finding(s). Request repair before final support.`} />;
  }
  const support = finalSupportState(run.evidence.length, passedTestCount(tests));
  if (!canRecord) {
    return <FocusActionLine icon="doc" label="Final support / insufficient" text={`Needs an assistant answer before support can be recorded. ${support.text}`} />;
  }
  return <FocusActionLine icon="doc" label="Record final support" onClick={onRecordFinal} text={`${support.text} No new claims are generated.`} />;
}

function unresolvedReview(reviews: ReviewReportView[], runId: string) {
  const report = [...reviews].reverse().find((item) => item.runId === runId);
  return report && report.decision !== "accepted" && report.findings.length > 0 ? report : undefined;
}

function passedTestCount(tests: TestArtifactView[]) {
  return tests.filter((test) => test.status === "passed").length;
}

function finalSupportState(evidenceCount: number, passedTestCount: number) {
  const counts = `${evidenceCount} evidence receipt(s), ${passedTestCount} passed test receipt(s).`;
  if (evidenceCount > 0 && passedTestCount > 0) {
    return { label: (status: string) => status, text: counts };
  }
  if (evidenceCount === 0 && passedTestCount === 0) {
    return { label: () => "partial", text: `Unsupported and untested: ${counts}` };
  }
  if (evidenceCount === 0) {
    return { label: () => "partial", text: `Insufficient evidence: ${counts}` };
  }
  return { label: () => "partial", text: `Untested: ${counts}` };
}

function schedulerLabel(kind: AgentScheduleDecisionView["kind"]) {
  return `Next / ${kind.replaceAll("_", " ")}`;
}

export function FocusActionLine({
  icon,
  label,
  onClick,
  text,
}: {
  icon: "doc" | "flask" | "plan";
  label: string;
  onClick?: () => void;
  text: string;
}) {
  return <button className="focus-action-line" onClick={onClick} type="button"><FocusIcon name={icon} /><span><b>{label}</b><em>{text}</em></span></button>;
}
