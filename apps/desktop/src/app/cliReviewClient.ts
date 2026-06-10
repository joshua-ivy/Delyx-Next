import { invoke } from "@tauri-apps/api/core";

export interface CliReviewResult {
  adapterId: string;
  verdict: "pass" | "fail" | "unclear";
  text: string;
  /** Corrected code the reviewer produced for a failing review (no fences). */
  fix?: string | null;
}

export interface QaqcModelOption {
  id: string;
  label: string;
  hint: string;
}

/**
 * Selectable reviewer models per CLI, cheapest first. The first entry is the
 * economical default (matches the backend default in cli_review.rs). Update here
 * when the CLIs ship new models — this is the single source of truth for the UI.
 */
export const QAQC_MODELS: Record<string, QaqcModelOption[]> = {
  "claude-code": [
    { id: "haiku", label: "Haiku", hint: "Cheapest · fast (default)" },
    { id: "sonnet", label: "Sonnet", hint: "Stronger · higher cost" },
    { id: "opus", label: "Opus", hint: "Most thorough · priciest" },
  ],
  "codex-cli": [
    { id: "gpt-5.4-mini", label: "GPT-5.4 mini", hint: "Cheapest · fast (default)" },
    { id: "gpt-5.4", label: "GPT-5.4", hint: "Flagship · higher cost" },
    { id: "gpt-5.5", label: "GPT-5.5", hint: "Most capable · priciest" },
  ],
};

/** The economical default reviewer model for a CLI (the cheapest preset). */
export function defaultQaqcModel(adapterId: string): string | undefined {
  return QAQC_MODELS[adapterId]?.[0]?.id;
}

export async function sendCliReview(
  adapterId: string,
  task: string,
  content: string,
  workingDirectory: string,
  model?: string,
): Promise<CliReviewResult> {
  return invoke<CliReviewResult>("cli_review", {
    request: {
      adapterId,
      content,
      model,
      startedAtMs: Date.now(),
      task,
      timeoutMs: 180_000,
      workingDirectory,
    },
  });
}

/**
 * Pull just the fenced code blocks out of a model reply so QA/QC reviews the code
 * instead of the surrounding prose padding (fewer input tokens per CLI call).
 * Returns null when there is no code worth spending a review call on.
 */
export function extractReviewableCode(content: string): string | null {
  const blocks: string[] = [];
  const fence = /```[^\n]*\n([\s\S]*?)```/g;
  let match: RegExpExecArray | null;
  while ((match = fence.exec(content)) !== null) {
    const code = match[1].trim();
    if (code) {
      blocks.push(code);
    }
  }
  return blocks.length > 0 ? blocks.join("\n\n") : null;
}

/** Render a QA/QC verdict as a thread system message. */
export function qaqcVerdictMessage(adapterId: string, result: CliReviewResult): string {
  const mark = result.verdict === "pass" ? "✓" : result.verdict === "fail" ? "⚠" : "?";
  const head = `${mark} QA/QC (${adapterId}): ${result.verdict.toUpperCase()}`;
  const body = result.text.trim();
  return body ? `${head}\n\n${body}` : head;
}
