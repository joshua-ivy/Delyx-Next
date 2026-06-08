import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
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
  if (decision.kind === "run_tests") {
    return <FocusActionLine icon="flask" label="Next / run tests" onClick={onRunTests} text={decision.message} />;
  }
  if (decision.kind === "run_review") {
    return <FocusActionLine icon="doc" label="Next / run review" onClick={onRunReview} text={decision.message} />;
  }
  if (decision.kind === "ready_for_final_support") {
    return <FocusActionLine icon="doc" label="Next / final support" onClick={onRecordFinal} text={decision.message} />;
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
}: {
  canRecord: boolean;
  onRecordFinal: () => void;
  run: AgentRunView | undefined;
}) {
  if (run?.outcome) {
    const evidence = run.outcome.evidenceRecordIds.length;
    const tests = run.outcome.testArtifactIds.length;
    return <FocusActionLine icon="doc" label={`Final support / ${run.outcome.status}`} text={`${evidence} evidence receipt(s), ${tests} passed test receipt(s)`} />;
  }
  if (!run || !canRecord) {
    return null;
  }
  return <FocusActionLine icon="doc" label="Record final support" onClick={onRecordFinal} text="Links existing evidence and passed tests; no new claims." />;
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
