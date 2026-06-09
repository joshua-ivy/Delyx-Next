import { invoke } from "@tauri-apps/api/core";

export interface CliChatResult {
  adapterId: string;
  text: string;
}

export async function sendCliChat(
  adapterId: string,
  prompt: string,
  workingDirectory: string,
): Promise<CliChatResult> {
  return invoke<CliChatResult>("cli_chat", {
    request: {
      adapterId,
      prompt,
      startedAtMs: Date.now(),
      timeoutMs: 120_000,
      workingDirectory,
    },
  });
}
