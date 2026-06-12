import { invoke } from "@tauri-apps/api/core";
import { sendModelChat, sendModelChatStream } from "../models/modelClient";
import type { ModelSettingsView, ThreadRoleMessage } from "../models/modelTypes";
import type {
  CampaignCreateInput,
  CampaignSnapshotView,
  CampaignTurnCommitView,
  CampaignTurnView,
  CampaignView,
  EraPackView,
  TurnResolutionView,
} from "./campaignTypes";

/** Compact the chronicle every N turns so 100-turn campaigns stay in budget. */
const MEMORY_COMPACTION_INTERVAL = 15;

export async function loadCampaignPacks(): Promise<EraPackView[]> {
  return invoke<EraPackView[]>("campaign_pack_list");
}

/** Folder where drop-in era packs live; new wars need no rebuild. */
export async function loadCampaignPackFolder(): Promise<string> {
  return invoke<string>("campaign_pack_folder");
}

export async function loadCampaignSnapshot(projectId: string): Promise<CampaignSnapshotView> {
  return invoke<CampaignSnapshotView>("campaign_snapshot", { projectId });
}

export async function loadCampaignTurns(campaignId: string): Promise<CampaignTurnView[]> {
  return invoke<CampaignTurnView[]>("campaign_turns", { campaignId });
}

export async function createCampaign(input: CampaignCreateInput): Promise<CampaignSnapshotView> {
  return invoke<CampaignSnapshotView>("campaign_create", {
    request: { ...input, createdAt: new Date().toISOString() },
  });
}

export async function setCampaignRating(
  campaignId: string,
  contentRating: "story" | "heroic" | "historical",
  parentConfirmed: boolean,
): Promise<CampaignView> {
  return invoke<CampaignView>("campaign_set_rating", {
    request: { campaignId, contentRating, parentConfirmed, updatedAt: new Date().toISOString() },
  });
}

export interface CampaignTurnResult {
  turn: CampaignTurnView;
  campaign: CampaignView;
  resolution?: TurnResolutionView;
  cancelled: boolean;
}

export interface CampaignTurnHandlers {
  onToken: (accumulated: string, delta: string) => void;
  /** Fires as soon as the app rolls dice, before the model starts narrating. */
  onResolution?: (resolution: TurnResolutionView) => void;
}

/**
 * One Game Master turn: the app rolls any dice, Rust assembles the layered GM
 * prompt from canon, the narration streams through the existing local-model
 * pipeline, and the finished scene is committed - delta extracted, validated,
 * and applied server-side. A cancelled stream keeps the partial scene.
 */
export async function sendCampaignTurn(
  settings: ModelSettingsView,
  campaignId: string,
  playerText: string,
  handlers: CampaignTurnHandlers,
): Promise<CampaignTurnResult> {
  const prompt = await invoke<{ messages: ThreadRoleMessage[]; resolution?: TurnResolutionView }>(
    "campaign_turn_prompt",
    { request: { campaignId, playerText } },
  );
  if (prompt.resolution) {
    handlers.onResolution?.(prompt.resolution);
  }
  const result = await sendModelChatStream(settings, prompt.messages, handlers.onToken);
  if (!result.text.trim()) {
    throw new Error("The Game Master returned an empty scene. Try again.");
  }
  // A scene without a parseable delta gets one schema-locked extraction pass
  // (Delyx Local only). Best-effort: a repair failure never blocks the turn,
  // and cancelled partials are committed as-is — half a scene is not canon.
  let modelText = result.text;
  if (!result.cancelled) {
    const repair = await invoke<{ rawText: string; repaired: boolean }>("campaign_delta_repair", {
      request: { providerId: result.providerId, model: result.model, rawText: result.text },
    }).catch(() => undefined);
    if (repair?.repaired) {
      modelText = repair.rawText;
    }
  }
  const committed = await invoke<CampaignTurnCommitView>("campaign_turn_commit", {
    request: {
      campaignId,
      playerText,
      modelText,
      resolution: prompt.resolution ?? null,
      createdAt: new Date().toISOString(),
    },
  });
  return {
    turn: committed.turn,
    campaign: committed.campaign,
    resolution: prompt.resolution,
    cancelled: result.cancelled,
  };
}

/**
 * Roll the campaign chronicle when due. Call after a committed turn; runs a
 * non-streaming summarize pass on the local model and stores the result.
 * Returns the updated campaign, or undefined when compaction was not due.
 */
export async function compactCampaignMemoryIfDue(
  settings: ModelSettingsView,
  campaignId: string,
  turnCount: number,
): Promise<CampaignView | undefined> {
  if (turnCount === 0 || turnCount % MEMORY_COMPACTION_INTERVAL !== 0) {
    return undefined;
  }
  const messages = await invoke<ThreadRoleMessage[]>("campaign_memory_prompt", { campaignId });
  const result = await sendModelChat(settings, messages);
  if (!result.text.trim()) {
    return undefined;
  }
  return invoke<CampaignView>("campaign_memory_commit", {
    request: { campaignId, summary: result.text.trim(), createdAt: new Date().toISOString() },
  });
}

/**
 * Async continuity QA/QC over the subscription CLI (claude/codex). Append-only
 * by design: the narration is already on screen; this quietly grades it and
 * stores findings on the turn. Never throws into the play loop - failures
 * resolve to a "skipped" turn status.
 */
export async function runCampaignTurnQaqc(
  adapterId: string,
  campaignId: string,
  turnIndex: number,
  workingDirectory: string,
): Promise<CampaignTurnView | undefined> {
  try {
    const prompt = await invoke<string>("campaign_qaqc_prompt", {
      request: { campaignId, turnIndex },
    });
    const reply = await invoke<{ adapterId: string; text: string }>("cli_chat", {
      request: {
        adapterId,
        prompt,
        workingDirectory,
        timeoutMs: 120_000,
        startedAtMs: Date.now(),
      },
    });
    const { status, notes } = parseQaqcReply(reply.text);
    return await invoke<CampaignTurnView>("campaign_turn_qaqc_commit", {
      request: { campaignId, turnIndex, qaqcStatus: status, qaqcNotes: notes ?? null },
    });
  } catch {
    try {
      return await invoke<CampaignTurnView>("campaign_turn_qaqc_commit", {
        request: { campaignId, turnIndex, qaqcStatus: "skipped", qaqcNotes: null },
      });
    } catch {
      return undefined;
    }
  }
}

export function parseQaqcReply(text: string): {
  status: "clean" | "corrected" | "skipped";
  notes?: string;
} {
  const lower = text.toLowerCase();
  if (lower.includes("verdict: clean")) {
    return { status: "clean" };
  }
  if (lower.includes("verdict: issues")) {
    const issueStart = lower.indexOf("verdict: issues");
    const notes = text
      .slice(issueStart + "verdict: issues".length)
      .trim()
      .slice(0, 2000);
    return { status: "corrected", notes: notes || "Issues flagged without detail." };
  }
  return { status: "skipped", notes: text.trim().slice(0, 500) || undefined };
}
