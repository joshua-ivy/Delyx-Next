import { invoke } from "@tauri-apps/api/core";

export interface CliReviewResult {
  adapterId: string;
  verdict: "pass" | "fail" | "unclear";
  text: string;
}

export async function sendCliReview(
  adapterId: string,
  task: string,
  content: string,
  workingDirectory: string,
): Promise<CliReviewResult> {
  return invoke<CliReviewResult>("cli_review", {
    request: {
      adapterId,
      content,
      startedAtMs: Date.now(),
      task,
      timeoutMs: 180_000,
      workingDirectory,
    },
  });
}

/** Render a QA/QC verdict as a thread system message. */
export function qaqcVerdictMessage(adapterId: string, result: CliReviewResult): string {
  const mark = result.verdict === "pass" ? "✓" : result.verdict === "fail" ? "⚠" : "?";
  const head = `${mark} QA/QC (${adapterId}): ${result.verdict.toUpperCase()}`;
  const body = result.text.trim();
  return body ? `${head}\n\n${body}` : head;
}
