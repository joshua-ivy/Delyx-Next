import { useState } from "react";
import { FocusIcon } from "../../app/focusAtoms";
import { MarkdownMessage } from "../../app/focusMarkdown";
import type { CampaignTurnView, TurnResolutionView } from "./campaignTypes";

export function TurnBlock({ turn }: { turn: CampaignTurnView }) {
  const resolution = parseResolution(turn.resolutionJson);
  const [showNotes, setShowNotes] = useState(false);
  return (
    <div className="campaign-turn">
      <div className="campaign-player mono">
        <FocusIcon name="zap" /> {turn.playerText}
      </div>
      {resolution && <DiceBlock resolution={resolution} />}
      <div className="campaign-scene">
        <MarkdownMessage text={turn.narration} />
        <button
          className={`campaign-qaqc ${turn.qaqcStatus}`}
          onClick={() => setShowNotes((current) => !current)}
          title={qaqcTitle(turn)}
          type="button"
        >
          {qaqcGlyph(turn.qaqcStatus)}
        </button>
      </div>
      {showNotes && turn.qaqcNotes && (
        <div className="campaign-qaqc-notes mono">{turn.qaqcNotes}</div>
      )}
    </div>
  );
}

export function DiceBlock({ resolution }: { resolution: TurnResolutionView }) {
  return (
    <div className={`campaign-dice ${resolution.outcome}`}>
      <FocusIcon name="dice" />
      <span className="mono">
        {resolution.check} check: {resolution.roll.join(" + ")}
        {resolution.stat !== 0 &&
          ` ${resolution.stat > 0 ? "+" : "-"} ${Math.abs(resolution.stat)}`}{" "}
        = {resolution.total}
      </span>
      <b>{outcomeLabel(resolution.outcome)}</b>
    </div>
  );
}

function parseResolution(json: string): TurnResolutionView | undefined {
  if (!json || json === "{}") {
    return undefined;
  }
  try {
    const parsed = JSON.parse(json) as TurnResolutionView;
    return parsed.check ? parsed : undefined;
  } catch {
    return undefined;
  }
}

function outcomeLabel(outcome: TurnResolutionView["outcome"]) {
  return outcome === "success" ? "Success" : outcome === "partial" ? "At a cost" : "Setback";
}

function qaqcGlyph(status: CampaignTurnView["qaqcStatus"]) {
  return status === "clean" ? "✓" : status === "corrected" ? "⚠" : status === "skipped" ? "–" : "◌";
}

function qaqcTitle(turn: CampaignTurnView) {
  switch (turn.qaqcStatus) {
    case "clean":
      return "Continuity check passed";
    case "corrected":
      return turn.qaqcNotes ?? "Continuity issues noted";
    case "skipped":
      return "Continuity check skipped";
    default:
      return "Continuity check pending";
  }
}
