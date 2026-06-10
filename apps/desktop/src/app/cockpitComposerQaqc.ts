import { extractReviewableCode, qaqcVerdictMessage, sendCliReview } from "./cliReviewClient";
import type { TaskThread } from "../features/threads/threadTypes";
import { notifyLocalAction } from "./ShellPreferenceController";
import { appendMessage, type ComposerBindingState } from "./cockpitComposerThreadOps";

// How many times to re-review (and re-fix) the corrected code before showing it.
// Bounded so a failing reply costs at most 1 + this many cheap CLI calls.
const QAQC_VERIFY_ROUNDS = 2;

function reviewerLabel(adapterId: string): string {
  if (adapterId === "claude-code") {
    return "Claude Code";
  }
  if (adapterId === "codex-cli") {
    return "Codex";
  }
  return adapterId;
}

/** Drop the leading "VERDICT: …" line so findings can be reframed without it. */
function reviewFindings(text: string): string {
  return text.replace(/^\s*VERDICT:\s*(PASS|FAIL|UNCLEAR)\b.*\n?/i, "").trim();
}

/**
 * Background QA/QC of a model reply. The answer is already shown; this only
 * appends a follow-up — a PASS note, or the corrected code when a fix is found —
 * so it never blocks or hides the user's answer. No reviewer / no code → no-op.
 */
export async function runQaqcFollowup(state: ComposerBindingState, thread: TaskThread, replyText: string): Promise<void> {
  const adapter = state.qaqcAdapterId;
  if (!adapter) {
    return;
  }
  // Only spend a paid CLI review when there is code to review, and send just the
  // code blocks rather than the model's prose — both cut tokens on every reply.
  const code = extractReviewableCode(replyText);
  if (!code) {
    return;
  }
  const label = reviewerLabel(adapter);
  notifyLocalAction(`QA/QC reviewing via ${label}…`, "info");
  try {
    const result = await sendCliReview(adapter, thread.goal, code, state.activeProject.path, state.qaqcModel);
    const fix = result.fix?.trim();
    if (result.verdict === "fail" && fix) {
      const originalIssues = reviewFindings(result.text);
      // Re-check the corrected code (bounded) so we only claim "verified" once a
      // fresh review of the corrected code actually passes.
      let bestFix = fix;
      let verified = false;
      let remainingFindings = "";
      for (let round = 0; round < QAQC_VERIFY_ROUNDS; round += 1) {
        const recheck = await sendCliReview(adapter, thread.goal, bestFix, state.activeProject.path, state.qaqcModel);
        if (recheck.verdict === "pass") {
          verified = true;
          break;
        }
        remainingFindings = reviewFindings(recheck.text);
        const nextFix = recheck.fix?.trim();
        if (!nextFix) {
          break;
        }
        bestFix = nextFix;
      }
      const verdictTag = verified ? "verified" : "fixed";
      appendMessage(state, thread.id, { role: "assistant", body: `[[qaqc:${verdictTag}:${label}]]\n\n\`\`\`\n${bestFix}\n\`\`\`` }, "idle");
      const note = verified
        ? [`✓ QA/QC (${label}): FIXED & VERIFIED — re-checked and now passes.`, originalIssues && `\nIssues found in the original and corrected:\n\n${originalIssues}`].filter(Boolean).join("\n")
        : `✦ QA/QC (${label}): FIXED (not fully verified)\n\nCorrected the main issues, but a re-check still flagged:\n\n${remainingFindings || originalIssues}`;
      appendMessage(state, thread.id, { role: "system", body: note }, "idle");
      notifyLocalAction(verified ? `QA/QC fixed & verified via ${adapter}` : `QA/QC fixed (unverified) via ${adapter}`, verified ? "success" : "warning");
      return;
    }
    if (result.verdict === "pass") {
      appendMessage(state, thread.id, { role: "system", body: `✓ QA/QC (${label}): PASS` }, "idle");
      notifyLocalAction(`QA/QC pass via ${adapter}`, "success");
      return;
    }
    // Failed with no usable fix (or unclear): surface why (the answer stays put).
    appendMessage(state, thread.id, { role: "system", body: qaqcVerdictMessage(adapter, result) }, "idle");
    notifyLocalAction(`QA/QC ${result.verdict} via ${adapter}`, "warning");
  } catch (error) {
    appendMessage(state, thread.id, { role: "system", body: `QA/QC review via ${adapter} failed: ${error instanceof Error ? error.message : "review failed"}` }, "idle");
  }
}
