import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import { FocusIcon } from "./focusAtoms";
import { firstRunnableTestCommand } from "./testCommand";

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
