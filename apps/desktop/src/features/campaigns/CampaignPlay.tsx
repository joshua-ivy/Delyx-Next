import { useEffect, useMemo, useRef, useState } from "react";
import { FocusIcon } from "../../app/focusAtoms";
import { MarkdownMessage } from "../../app/focusMarkdown";
import { cancelActiveModelStream } from "../models/modelClient";
import type { ModelSettingsView } from "../models/modelTypes";
import {
  compactCampaignMemoryIfDue,
  runCampaignTurnQaqc,
  sendCampaignTurn,
  setCampaignRating,
} from "./campaignClient";
import { DiceBlock, TurnBlock } from "./CampaignTurnBlocks";
import type {
  CampaignTurnView,
  CampaignView as CampaignRecordView,
  EraPackView,
  EraScenarioView,
  TurnResolutionView,
} from "./campaignTypes";
import { RATINGS, describe } from "./campaignViewShared";

const QUICK_ACTIONS = ["Look around", "Talk to the squad", "Press on", "Take cover"];

export function CampaignPlay({
  campaign,
  modelSettings,
  onCampaign,
  onError,
  onTurns,
  packs,
  qaqcAdapterId,
  turns,
  workingDirectory,
}: {
  campaign: CampaignRecordView;
  modelSettings: ModelSettingsView;
  onCampaign: (campaign: CampaignRecordView) => void;
  onError: (message: string) => void;
  onTurns: (turns: CampaignTurnView[]) => void;
  packs: EraPackView[];
  qaqcAdapterId?: string;
  turns: CampaignTurnView[];
  workingDirectory: string;
}) {
  const [value, setValue] = useState("");
  const [streaming, setStreaming] = useState("");
  const [pendingPlayer, setPendingPlayer] = useState("");
  const [pendingDice, setPendingDice] = useState<TurnResolutionView | undefined>();
  const [busy, setBusy] = useState(false);
  const timelineRef = useRef<HTMLDivElement | null>(null);
  const scenario = useMemo<EraScenarioView | undefined>(() => {
    const pack = packs.find((item) => item.id === campaign.eraPackId);
    return pack?.scenarios.find((item) => item.id === campaign.scenarioId);
  }, [packs, campaign.eraPackId, campaign.scenarioId]);

  useEffect(() => {
    timelineRef.current?.scrollTo({ top: timelineRef.current.scrollHeight });
  }, [turns.length, streaming]);

  const send = (text: string) => {
    const playerText = text.trim();
    if (!playerText || busy) {
      return;
    }
    setValue("");
    setBusy(true);
    setPendingPlayer(playerText);
    setPendingDice(undefined);
    setStreaming("");
    sendCampaignTurn(modelSettings, campaign.id, playerText, {
      onToken: (accumulated) => setStreaming(accumulated),
      onResolution: setPendingDice,
    })
      .then((result) => {
        const nextTurns = [...turns, result.turn];
        onTurns(nextTurns);
        onCampaign(result.campaign);
        // Post-turn follow-ups are append-only and never block the scene.
        if (qaqcAdapterId) {
          void runCampaignTurnQaqc(
            qaqcAdapterId,
            campaign.id,
            result.turn.turnIndex,
            workingDirectory,
          ).then((graded) => graded && updateTurnAfter(graded));
        }
        void compactCampaignMemoryIfDue(
          modelSettings,
          campaign.id,
          result.campaign.turnCount,
        ).then((updated) => updated && onCampaign(updated));
      })
      .catch((problem) => onError(describe(problem)))
      .finally(() => {
        setBusy(false);
        setStreaming("");
        setPendingPlayer("");
        setPendingDice(undefined);
      });
  };

  // QA/QC may land after more turns were played; update by index, not by list copy.
  const turnsRef = useRef(turns);
  turnsRef.current = turns;
  const updateTurnAfter = (next: CampaignTurnView) => {
    onTurns(
      turnsRef.current.map((turn) => (turn.turnIndex === next.turnIndex ? next : turn)),
    );
  };

  return (
    <div className="campaign-play">
      <div className="campaign-timeline" ref={timelineRef}>
        {scenario && (
          <div className="campaign-scene">
            <MarkdownMessage text={scenario.opening} />
          </div>
        )}
        {turns.map((turn) => (
          <TurnBlock key={turn.turnIndex} turn={turn} />
        ))}
        {pendingPlayer && (
          <div className="campaign-player mono">
            <FocusIcon name="zap" /> {pendingPlayer}
          </div>
        )}
        {pendingDice && <DiceBlock resolution={pendingDice} />}
        {streaming && (
          <div className="campaign-scene streaming">
            <MarkdownMessage text={streaming} />
          </div>
        )}
      </div>

      <aside className="campaign-rail">
        <div className="campaign-rail-block">
          <h3>World</h3>
          <div className="mono">{campaign.worldDate}</div>
          <div>{campaign.location}</div>
        </div>
        <div className="campaign-rail-block">
          <h3>Squad</h3>
          {campaign.characters.map((character) => (
            <div className="campaign-char" key={character.id}>
              <span className={`dot ${characterDot(character.status)}`} />
              <span className="campaign-char-name">
                {character.name}
                {character.kind === "player" ? " (you)" : ""}
              </span>
              <span className="campaign-char-status mono">{character.status}</span>
            </div>
          ))}
        </div>
        {campaign.events.length > 0 && (
          <div className="campaign-rail-block">
            <h3>Settled facts</h3>
            {campaign.events.slice(-8).map((event) => (
              <div className="campaign-event" key={event.id}>
                {event.summary}
              </div>
            ))}
          </div>
        )}
        <div className="campaign-rail-block">
          <h3>Rating</h3>
          <div className="seg">
            {RATINGS.map((item) => (
              <button
                className={campaign.contentRating === item.key ? "on" : ""}
                key={item.key}
                onClick={() => {
                  if (campaign.contentRating === item.key) {
                    return;
                  }
                  // Parent gate: explicit confirmation before the tone dial moves.
                  const confirmed = window.confirm(
                    `Parent check: change this campaign's content rating to "${item.label}" (${item.hint})?`,
                  );
                  if (!confirmed) {
                    return;
                  }
                  setCampaignRating(campaign.id, item.key, true)
                    .then(onCampaign)
                    .catch((problem) => onError(describe(problem)));
                }}
                title={item.hint}
                type="button"
              >
                {item.label}
              </button>
            ))}
          </div>
        </div>
      </aside>

      <div className="campaign-composer">
        <div className="campaign-chips">
          {QUICK_ACTIONS.map((action) => (
            <button
              className="gchip"
              disabled={busy}
              key={action}
              onClick={() => send(action)}
              type="button"
            >
              {action}
            </button>
          ))}
        </div>
        <div className="campaign-input">
          <textarea
            className="in"
            disabled={busy}
            onChange={(event) => setValue(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey) {
                event.preventDefault();
                send(value);
              }
            }}
            placeholder="What do you do?"
            rows={1}
            value={value}
          />
          {busy ? (
            <button className="btn-stop" onClick={() => void cancelActiveModelStream()} type="button">
              Stop
            </button>
          ) : (
            <button className="btn-send" onClick={() => send(value)} type="button">
              Act
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function characterDot(status: string) {
  return status === "active" ? "success" : status === "dead" ? "danger" : "warn";
}
