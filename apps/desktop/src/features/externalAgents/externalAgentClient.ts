import { invoke } from "@tauri-apps/api/core";
import type {
  ExternalAgentAdapterView,
  ExternalAgentCommandContractView,
  ExternalAgentPermissionMode,
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

export async function previewExternalAgentContract(
  request: ExternalAgentContractPreviewRequest,
): Promise<ExternalAgentCommandContractView> {
  return invoke<ExternalAgentCommandContractView>("external_agent_contract_preview", { request });
}

export async function loadExternalAgentStatus() {
  return invoke<{ adapters: ExternalAgentAdapterView[] }>("external_agent_status");
}

export function externalAgentBridgeUnavailableState(
  current: ExternalAgentStateView,
): ExternalAgentStateView {
  return {
    ...current,
    adapters: [
      notCheckedAdapter("codex-cli", "Codex CLI"),
      notCheckedAdapter("claude-code", "Claude Code"),
      notCheckedAdapter("generic-terminal", "Generic terminal agent"),
    ],
  };
}

function notCheckedAdapter(id: string, label: string): ExternalAgentAdapterView {
  return {
    detail: "Desktop bridge unavailable in web preview; adapter detection was not checked.",
    id,
    label,
    status: "not_checked",
  };
}
