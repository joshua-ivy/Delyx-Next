import { invoke } from "@tauri-apps/api/core";
import type {
  ExternalAgentCommandContractView,
  ExternalAgentPermissionMode,
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
