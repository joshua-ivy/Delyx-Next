import { invoke } from "@tauri-apps/api/core";
import type {
  PatchApplyRequestView,
  PatchProposalRequestView,
  PatchRestoreRequestView,
} from "../patches/patchClient";
import type { TestRunRequestView } from "../tests/testClient";

export interface AgentPatchProposalExecuteRequest extends PatchProposalRequestView {
  createdAtMs: number;
}

export interface AgentPatchDraftExecuteRequest {
  approvalId: string;
  approvedRoots: string[];
  clientId: string;
  createdAtMs: number;
  filesLikelyInvolved: string[];
  goal: string;
  maxBytesPerFile?: number;
  model: string;
  planSteps: string[];
  projectPath: string;
  runId: string;
  scopePaths: string[];
}

export interface AgentPatchDraftDispatchRequest {
  execute: AgentPatchDraftExecuteRequest;
  hasSupportedTestCommand: boolean;
  patchDraftApprovalId?: string;
  testApprovalId?: string;
  nowMs: number;
}

export interface AgentPatchDraftContextRequest {
  approvalId: string;
  hasSupportedTestCommand: boolean;
  maxBytesPerFile?: number;
  model: string;
  nowMs: number;
  projectId: string;
  runId: string;
  testApprovalId?: string;
}

export interface AgentPatchDraftStepRequest {
  maxBytesPerFile?: number;
  model: string;
  nowMs: number;
  projectId: string;
  runId: string;
}

export interface AgentPatchApplyStepRequest {
  nowMs: number;
  runId: string;
  updatedAt: string;
}

export interface AgentExecutionBridgeView {
  status: "completed" | "failed" | "waiting_for_approval";
  runId: string;
  patchId?: string;
  message: string;
}

export interface AgentPatchDraftBridgeView extends AgentExecutionBridgeView {
  model: string;
  providerId: string;
}

export interface AgentTestExecutionBridgeView {
  status: "completed" | "failed" | "waiting_for_approval";
  runId: string;
  testArtifactId?: string;
  message: string;
}

export interface AgentReviewExecutionBridgeView {
  status: "completed" | "failed";
  runId: string;
  reviewReportId?: string;
  message: string;
}

export interface AgentReviewRevisionBridgeView {
  status: "revise_requested";
  runId: string;
  reviewReportId: string;
  findingId: string;
  nextFlow: string[];
  message: string;
}

export interface AgentScheduleDecisionView {
  kind:
    | "blocked"
    | "complete"
    | "ready_for_final_support"
    | "repair_requested"
    | "request_patch_apply_approval"
    | "resume_after_approval"
    | "run_patch_draft"
    | "run_patch_apply"
    | "run_review"
    | "run_tests"
    | "terminal"
    | "wait_for_approval";
  runId: string;
  message: string;
  approvalIds: string[];
  findingId?: string;
  proposalId?: string;
  reviewReportId?: string;
  patchCount: number;
  testCount: number;
  status?: string;
}

export interface AgentScheduleRequestView {
  runId: string;
  hasSupportedTestCommand: boolean;
  patchApplyApprovalId?: string;
  patchDraftApprovalId?: string;
  testApprovalId?: string;
  nowMs: number;
}

export async function executePatchProposalNodeOverBridge(
  request: AgentPatchProposalExecuteRequest,
): Promise<AgentExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentExecutionBridgeView>("agent_execute_patch_proposal", { request });
  } catch {
    return undefined;
  }
}

export async function dispatchPatchDraftNodeOverBridge(
  request: AgentPatchDraftDispatchRequest,
): Promise<AgentPatchDraftBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentPatchDraftBridgeView>("agent_dispatch_patch_draft", { request });
  } catch {
    return undefined;
  }
}

export async function dispatchPatchDraftFromContextOverBridge(
  request: AgentPatchDraftContextRequest,
): Promise<AgentPatchDraftBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentPatchDraftBridgeView>("agent_dispatch_patch_draft_from_context", { request });
  } catch {
    return undefined;
  }
}

export async function runPatchDraftSchedulerStepOverBridge(
  request: AgentPatchDraftStepRequest,
): Promise<AgentPatchDraftBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentPatchDraftBridgeView>("agent_run_patch_draft_step", { request });
  } catch {
    return undefined;
  }
}

export async function runPatchApplySchedulerStepOverBridge(
  request: AgentPatchApplyStepRequest,
): Promise<AgentExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentExecutionBridgeView>("agent_run_patch_apply_step", { request });
  } catch {
    return undefined;
  }
}

export async function executePatchApplyNodeOverBridge(
  request: PatchApplyRequestView,
): Promise<AgentExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentExecutionBridgeView>("agent_execute_patch_apply", { request });
  } catch {
    return undefined;
  }
}

export async function executePatchRestoreNodeOverBridge(
  request: PatchRestoreRequestView,
): Promise<AgentExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentExecutionBridgeView>("agent_execute_patch_restore", { request });
  } catch {
    return undefined;
  }
}

export async function executeTestRunNodeOverBridge(
  request: TestRunRequestView,
): Promise<AgentTestExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentTestExecutionBridgeView>("agent_execute_test_run", { request });
  } catch {
    return undefined;
  }
}

export async function executeReviewNodeOverBridge(
  runId: string,
): Promise<AgentReviewExecutionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentReviewExecutionBridgeView>("agent_execute_review", {
      request: { runId },
    });
  } catch {
    return undefined;
  }
}

export async function requestReviewRevisionOverBridge(
  runId: string,
  reviewReportId: string,
  findingId: string,
): Promise<AgentReviewRevisionBridgeView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentReviewRevisionBridgeView>("agent_request_review_revision", {
      request: { findingId, reviewReportId, runId, updatedAt: new Date().toISOString() },
    });
  } catch {
    return undefined;
  }
}

export async function scheduleNextRunActionOverBridge(
  request: AgentScheduleRequestView,
): Promise<AgentScheduleDecisionView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentScheduleDecisionView>("agent_schedule_next", { request });
  } catch {
    return undefined;
  }
}

export async function resumeWaitingRunOverBridge(
  request: AgentScheduleRequestView,
): Promise<AgentScheduleDecisionView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<AgentScheduleDecisionView>("agent_resume_waiting_run", { request });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
