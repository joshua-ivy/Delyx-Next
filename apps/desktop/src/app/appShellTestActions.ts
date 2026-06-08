import type { Dispatch, SetStateAction } from "react";

import { proposeApprovalOverBridge } from "../features/approvals/approvalClient";
import type { ActionProposalView } from "../features/approvals/approvalTypes";
import type { PatchProposalView } from "../features/patches/patchTypes";
import type { PlanView } from "../features/plans/planTypes";
import { executeTestRunNodeOverBridge } from "../features/runs/agentExecutorClient";
import type { AgentRunView } from "../features/runs/agentRunTypes";
import { loadTestSnapshot } from "../features/tests/testClient";
import type { TestArtifactView } from "../features/tests/testTypes";
import { loadThreadRunSnapshot, updateThreadStatusOverBridge } from "../features/threads/threadClient";
import type { TaskThread, ThreadStatus, ThreadUiState } from "../features/threads/threadTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { upsertActionProposal } from "./appShellApprovalActions";
import { recordApprovalProposalForRun } from "./appShellRunActions";
import { activeTestApproval } from "./appShellTestApprovalDecision";
import { updateThreadAndRunStatus } from "./cockpitStateTransitions";
import { notifyLocalAction } from "./ShellPreferenceController";
import { firstRunnableTestCommand, type RunnableTestCommand } from "./testCommand";

interface TestRunState {
  actionProposals: ActionProposalView[];
  activePlan: PlanView | undefined;
  activeProject: WorkspaceProject;
  activeRun: AgentRunView | undefined;
  activeThread: TaskThread | undefined;
  patches: PatchProposalView[];
  schedulerTestApprovalId?: string;
  schedulerConfirmedRunTests?: boolean;
  setActionProposals: Dispatch<SetStateAction<ActionProposalView[]>>;
  setAgentRuns: Dispatch<SetStateAction<AgentRunView[]>>;
  setTests: Dispatch<SetStateAction<TestArtifactView[]>>;
  setThreads: Dispatch<SetStateAction<TaskThread[]>>;
  setThreadState: Dispatch<SetStateAction<ThreadUiState>>;
}

export async function runTestsForActiveRun(state: TestRunState) {
  if (!state.activeRun || !state.activeThread) {
    notifyLocalAction("Create a thread with a run before testing", "warning");
    return;
  }
  if (!state.schedulerConfirmedRunTests && !state.patches.some((patch) => patch.status === "applied")) {
    notifyLocalAction("Tests run after a real patch has been applied", "warning");
    return;
  }
  if (state.activeProject.approvedRoots.length === 0) {
    notifyLocalAction("Test execution requires an approved workspace root", "warning");
    return;
  }
  const command = firstRunnableTestCommand(state.activePlan?.testsToRun);
  if (!command) {
    notifyLocalAction("No supported test command exists in the active plan", "warning");
    return;
  }
  const schedulerApproval = state.schedulerTestApprovalId
    ? activeTestApproval(state, command, state.schedulerTestApprovalId)
    : undefined;
  if (state.schedulerTestApprovalId && !schedulerApproval) {
    notifyLocalAction("Scheduler-selected test approval is no longer executable", "warning");
    return;
  }
  const approval = schedulerApproval ?? reusableTestApproval(state, command);
  if (!approval || approval.status !== "approved") {
    await queueTestApproval(state, command, approval ?? latestTestApproval(state, command));
    return;
  }
  await executeApprovedTest(state, command, approval);
}

async function queueTestApproval(
  state: TestRunState,
  command: RunnableTestCommand,
  existing: ActionProposalView | undefined,
) {
  if (existing?.status === "pending" && !approvalExpired(existing)) {
    notifyLocalAction("Approve the test command, then run tests again", "warning");
    return;
  }
  if (existing?.status === "denied") {
    notifyLocalAction("Test approval was denied; Delyx will not run that command", "warning");
    return;
  }
  const fallback = testApprovalProposal(state, command);
  const proposal = existing && approvalExpired(existing)
    ? { ...fallback, id: `${fallback.id}-${Date.now()}` }
    : fallback;
  const recorded = await proposeApprovalOverBridge(proposal) ?? proposal;
  state.setActionProposals((current) => upsertActionProposal(current, recorded));
  if (state.activeThread) {
    const now = new Date().toISOString();
    state.setAgentRuns((current) => recordApprovalProposalForRun(current, state.activeThread!, recorded, now));
    updateThreadAndRunStatus(state, state.activeThread, "waiting_for_approval");
  }
  notifyLocalAction("Approve the test command, then run tests again", "warning");
}

async function executeApprovedTest(
  state: TestRunState,
  command: RunnableTestCommand,
  approval: ActionProposalView,
) {
  updateThreadAndRunStatus(state, state.activeThread!, "testing");
  const now = new Date();
  const result = await executeTestRunNodeOverBridge({
    approvalId: approval.id,
    approvedRoots: state.activeProject.approvedRoots,
    args: command.args,
    createdAtMs: now.getTime(),
    program: command.program,
    runId: state.activeRun!.id,
    startedAt: now.toISOString(),
    timeoutMs: 5 * 60 * 1000,
    workingDirectory: state.activeProject.path,
  });
  if (!result) {
    notifyLocalAction("Desktop bridge is required to run approved tests", "warning");
    return;
  }
  await updateThreadStatusOverBridge(state.activeThread!.id, statusAfterTest(result.status), new Date().toISOString());
  await reloadTestState(state);
  state.setThreadState("ready");
  notifyLocalAction(result.message, result.status === "completed" ? "success" : "warning");
}

async function reloadTestState(state: TestRunState) {
  const [tests, snapshot] = await Promise.all([
    loadTestSnapshot(state.activeRun?.id ?? ""),
    loadThreadRunSnapshot(state.activeProject.id),
  ]);
  if (tests) {
    state.setTests(tests);
  }
  if (snapshot) {
    state.setThreads(snapshot.threads);
    state.setAgentRuns(snapshot.runs);
  }
}

function reusableTestApproval(state: TestRunState, command: RunnableTestCommand) {
  return state.actionProposals.find((proposal) => (
    matchesTestApproval(state, proposal, command)
    && proposal.status !== "denied"
    && proposal.status !== "expired"
    && !approvalExpired(proposal)
  ));
}

function latestTestApproval(state: TestRunState, command: RunnableTestCommand) {
  return state.actionProposals.find((proposal) => matchesTestApproval(state, proposal, command));
}

function matchesTestApproval(
  state: TestRunState,
  proposal: ActionProposalView,
  command: RunnableTestCommand,
) {
  return (
    proposal.runId === state.activeRun?.id
    && proposal.actionType === "run_terminal"
    && proposal.scope.commands?.includes(command.label)
  );
}

function approvalExpired(proposal: ActionProposalView) {
  const expiresAt = Date.parse(proposal.expiresAt);
  return proposal.status === "expired" || !Number.isFinite(expiresAt) || expiresAt <= Date.now();
}

function testApprovalProposal(state: TestRunState, command: RunnableTestCommand): ActionProposalView {
  const runId = state.activeRun!.id;
  return {
    actionType: "run_terminal",
    expectedResult: "Run one approved test command and capture stdout, stderr, exit status, duration, and parsed failures.",
    expiresAt: new Date(Date.now() + 30 * 60 * 1000).toISOString(),
    id: `approval-${runId}-test-${commandKey(command.label)}`,
    nodeId: `${runId}-test-${commandKey(command.label)}`,
    rationale: `Run active plan validation: ${command.label}`,
    requiredPermission: "terminal_command",
    riskLabel: "medium",
    rollbackPlan: "Test commands are read-only validation; discard the captured artifact if it is no longer relevant.",
    runId,
    scope: {
      commands: [command.label],
      kind: "terminal",
      projectId: state.activeProject.id,
      root: state.activeProject.path,
      summary: "Run one supported test command from the approved project root.",
    },
    status: "pending",
  };
}

function statusAfterTest(status: string): ThreadStatus {
  return status === "completed" ? "reviewing" : status === "waiting_for_approval" ? "waiting_for_approval" : "failed";
}

function commandKey(command: string) {
  return command.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 48) || "command";
}
