import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AgentOutcome, AgentRunView } from "../features/runs/agentRunTypes";
import type { ReviewReportView } from "../features/review/reviewTypes";
import type { TestArtifactView } from "../features/tests/testTypes";
import { FocusOutcomePeek } from "./FocusThreadArtifacts";

afterEach(cleanup);

describe("FocusOutcomePeek final support states", () => {
  it("shows supported final receipts when evidence and passed tests are linked", () => {
    renderOutcome({ run: runView({ outcomeEvidence: 1, outcomeTests: 1, status: "succeeded" }) });

    expect(screen.getByText("Final support / succeeded")).not.toBeNull();
    expect(screen.getByText("1 evidence receipt(s), 1 passed test receipt(s).")).not.toBeNull();
  });

  it("shows partial unsupported and untested final support after recording empty links", () => {
    renderOutcome({ run: runView({ outcomeEvidence: 0, outcomeTests: 0, status: "succeeded" }) });

    expect(screen.getByText("Final support / partial")).not.toBeNull();
    expect(screen.getByText("Unsupported and untested: 0 evidence receipt(s), 0 passed test receipt(s).")).not.toBeNull();
  });

  it("shows insufficient evidence before recording when only passed tests exist", () => {
    const onRecordFinal = vi.fn();
    renderOutcome({ onRecordFinal, run: runView(), tests: [testArtifact("passed")] });

    fireEvent.click(screen.getByRole("button", { name: /Record final support/ }));

    expect(onRecordFinal).toHaveBeenCalledTimes(1);
    expect(screen.getByText("Insufficient evidence: 0 evidence receipt(s), 1 passed test receipt(s). No new claims are generated.")).not.toBeNull();
  });

  it("shows untested support before recording when evidence exists without passed tests", () => {
    renderOutcome({ run: runView({ evidence: 1 }), tests: [testArtifact("failed")] });

    expect(screen.getByText("Untested: 1 evidence receipt(s), 0 passed test receipt(s). No new claims are generated.")).not.toBeNull();
  });

  it("shows why final support is insufficient when no assistant answer exists", () => {
    renderOutcome({ canRecord: false, run: runView({ evidence: 1 }), tests: [testArtifact("passed")] });

    expect(screen.getByText("Final support / insufficient")).not.toBeNull();
    expect(screen.getByText("Needs an assistant answer before support can be recorded. 1 evidence receipt(s), 1 passed test receipt(s).")).not.toBeNull();
    expect(screen.queryByRole("button", { name: /Record final support/ })).toBeNull();
  });

  it("shows review-blocked support while findings are unresolved", () => {
    renderOutcome({ reviews: [reviewWithFinding()], run: runView({ evidence: 1 }) });

    expect(screen.getByText("Final support / review blocked")).not.toBeNull();
    expect(screen.getByText("Review review-1 has 1 finding(s). Request repair before final support.")).not.toBeNull();
    expect(screen.queryByRole("button", { name: /Record final support/ })).toBeNull();
  });
});

function renderOutcome({
  canRecord = true,
  onRecordFinal = vi.fn(),
  reviews = [],
  run,
  tests = [],
}: {
  canRecord?: boolean;
  onRecordFinal?: () => void;
  reviews?: ReviewReportView[];
  run?: AgentRunView;
  tests?: TestArtifactView[];
}) {
  return render(<FocusOutcomePeek canRecord={canRecord} onRecordFinal={onRecordFinal} reviews={reviews} run={run} tests={tests} />);
}

function reviewWithFinding(): ReviewReportView {
  return {
    decision: "pending",
    evidenceSummary: "Stored review receipt.",
    findings: [{
      detail: "Runtime panic risk in new code.",
      filePath: "src/main.rs",
      hunkLabel: "patch-1:0",
      id: "finding-1",
      priority: "p1",
      riskLabel: "panic",
      suggestedFix: "Handle the None/Err case explicitly.",
      title: "Added unwrap can panic",
    }],
    id: "review-1",
    mode: "read_only",
    riskSummary: "1 prioritized finding.",
    runId: "run-1",
    testSummary: "Tests passed.",
  };
}

function runView({
  evidence = 0,
  outcomeEvidence,
  outcomeTests,
  status,
}: {
  evidence?: number;
  outcomeEvidence?: number;
  outcomeTests?: number;
  status?: AgentOutcome["status"];
} = {}): AgentRunView {
  return {
    artifacts: [],
    createdAt: "2026-06-08T00:00:00.000Z",
    events: [],
    evidence: Array.from({ length: evidence }, (_, index) => ({
      id: `evidence-${index + 1}`,
      retrievedAt: "2026-06-08T00:00:30.000Z",
      runId: "run-1",
      sourceId: "patch-1",
      sourceKind: "diff",
    })),
    goal: "Ship a supported answer",
    id: "run-1",
    metrics: { approvalCount: 0, artifactCount: 0, commandCount: 0, eventCount: 0, evidenceCount: evidence, nodeCount: 0 },
    mode: "build",
    nodes: [],
    outcome: status ? {
      evidenceRecordIds: Array.from({ length: outcomeEvidence ?? 0 }, (_, index) => `evidence-${index + 1}`),
      status,
      summary: "Final answer summary.",
      testArtifactIds: Array.from({ length: outcomeTests ?? 0 }, (_, index) => `test-${index + 1}`),
    } : undefined,
    status: status ?? "running",
    threadId: "thread-1",
    updatedAt: "2026-06-08T00:01:00.000Z",
  };
}

function testArtifact(status: TestArtifactView["status"]): TestArtifactView {
  return {
    command: ".\\.tools\\npm.cmd test",
    completedAt: "2026-06-08T00:01:00.000Z",
    cwd: "C:\\repo",
    durationMs: 20,
    exitCode: status === "passed" ? 0 : 1,
    id: `test-${status}`,
    runId: "run-1",
    startedAt: "2026-06-08T00:00:59.000Z",
    status,
    stderr: "",
    stdout: "",
  };
}
