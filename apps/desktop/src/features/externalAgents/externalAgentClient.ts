import { invoke } from "@tauri-apps/api/core";
import type {
  ExternalAgentAdapterView,
  ExternalAgentAdapterKind,
  ExternalAgentCommandContractView,
  ExternalAgentPermissionMode,
  ExternalAgentRunArtifactView,
  ExternalAgentStateView,
} from "./externalAgentTypes";

export type ExternalAgentContractKind = "codex_cli" | "claude_code";

export interface ExternalAgentContractPreviewRequest {
  kind: ExternalAgentContractKind;
  task: string;
  workingDirectory: string;
  permissionMode: ExternalAgentPermissionMode;
  runId: string;
}

export interface ExternalAgentCodexRunRequest {
  runId: string;
  externalApprovalId: string;
  terminalApprovalId: string;
  task: string;
  workingDirectory: string;
  approvedRoots: string[];
  allowedPaths: string[];
  permissionMode: ExternalAgentPermissionMode;
  timeoutMs: number;
  createdAtMs: number;
  checkpointId?: string;
  worktreeId?: string;
  captureDiff: boolean;
  changedFiles: string[];
  testArtifactIds: string[];
}

export async function previewExternalAgentContract(
  request: ExternalAgentContractPreviewRequest,
): Promise<ExternalAgentCommandContractView> {
  return invoke<ExternalAgentCommandContractView>("external_agent_contract_preview", { request });
}

export async function loadExternalAgentStatus() {
  return invoke<{ adapters: ExternalAgentAdapterView[] }>("external_agent_status");
}

export async function runCodexExternalAgent(
  request: ExternalAgentCodexRunRequest,
): Promise<ExternalAgentRunArtifactView> {
  return invoke<ExternalAgentRunArtifactView>("external_agent_run_codex", { request });
}

export async function loadExternalAgentRunSnapshot(runId: string) {
  return invoke<ExternalAgentRunArtifactView[]>("external_agent_run_snapshot", { runId });
}

export function externalAgentBridgeUnavailableState(
  current: ExternalAgentStateView,
): ExternalAgentStateView {
  return {
    ...current,
    adapters: [
      notCheckedAdapter("codex-cli", "codex_cli", "Codex CLI"),
      notCheckedAdapter(
        "claude-code",
        "claude_code",
        "Claude Code",
        "Desktop bridge unavailable in web preview; detection and command-contract preview only.",
      ),
      notCheckedAdapter("generic-terminal", "generic_terminal", "Generic terminal agent"),
    ],
  };
}

function notCheckedAdapter(
  id: string,
  kind: ExternalAgentAdapterKind,
  label: string,
  detail = "Desktop bridge unavailable in web preview; adapter detection was not checked.",
): ExternalAgentAdapterView {
  return {
    detail,
    id,
    kind,
    label,
    status: "not_checked",
  };
}
