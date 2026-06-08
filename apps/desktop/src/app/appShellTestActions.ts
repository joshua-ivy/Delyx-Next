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
  if (!state.patches.some((patch) => patch.status === "applied")) {
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
  const approval = activeTestApproval(state, command);
  if (!approval || approval.status !== "approved") {
    await queueTestApproval(state, command, approval);
    return;
  }
  await executeApprovedTest(state, command, approval);
}

async function queueTestApproval(
  state: TestRunState,
  command: RunnableTestCommand,
  existing: ActionProposalView | undefined,
) {
  const proposal = existing ?? testApprovalProposal(state, command);
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

function activeTestApproval(state: TestRunState, command: RunnableTestCommand) {
  const now = Date.now();
  return state.actionProposals.find((proposal) => (
    proposal.runId === state.activeRun?.id
    && proposal.actionType === "run_terminal"
    && proposal.scope.commands?.includes(command.label)
    && proposal.status !== "denied"
    && proposal.status !== "expired"
    && Date.parse(proposal.expiresAt) > now
  ));
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
