import { invoke } from "@tauri-apps/api/core";
import type { ActionProposalView, ProposalStatus } from "./approvalTypes";

export async function proposeApprovalOverBridge(
  proposal: ActionProposalView,
): Promise<ActionProposalView | undefined> {
  const expiresAtMs = Date.parse(proposal.expiresAt);
  if (!hasTauriRuntime() || !Number.isFinite(expiresAtMs)) {
    return undefined;
  }
  try {
    return await invoke<ActionProposalView>("approval_propose", {
      request: { ...proposal, clientId: proposal.id, expiresAtMs },
    });
  } catch {
    return undefined;
  }
}

export async function decideApprovalOverBridge(
  proposalId: string,
  decision: Extract<ProposalStatus, "approved" | "denied">,
  decidedAtMs: number,
): Promise<ActionProposalView | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<ActionProposalView>("approval_decide", {
      request: { decidedAtMs, decision, proposalId },
    });
  } catch {
    return undefined;
  }
}

export async function loadApprovalSnapshot(runId: string): Promise<ActionProposalView[] | undefined> {
  if (!hasTauriRuntime()) {
    return undefined;
  }
  try {
    return await invoke<ActionProposalView[]>("approval_snapshot", { runId });
  } catch {
    return undefined;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
