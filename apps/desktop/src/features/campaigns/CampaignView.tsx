import { useEffect, useMemo, useRef, useState } from "react";
import { FocusIcon } from "../../app/focusAtoms";
import { MarkdownMessage } from "../../app/focusMarkdown";
import { cancelActiveModelStream } from "../models/modelClient";
import type { ModelSettingsView } from "../models/modelTypes";
import {
  compactCampaignMemoryIfDue,
  createCampaign,
  loadCampaignPacks,
  loadCampaignSnapshot,
  loadCampaignTurns,
  runCampaignTurnQaqc,
  sendCampaignTurn,
  setCampaignRating,
} from "./campaignClient";
import type {
  CampaignTurnView,
  CampaignView as CampaignRecordView,
  EraPackView,
  EraScenarioView,
  TurnResolutionView,
} from "./campaignTypes";

const QUICK_ACTIONS = ["Look around", "Talk to the squad", "Press on", "Take cover"];
const RATINGS: Array<{ key: "story" | "heroic" | "historical"; label: string; hint: string }> = [
  { key: "story", label: "Story", hint: "adventure-novel tone (default)" },
  { key: "heroic", label: "Heroic", hint: "PG-13 war film intensity" },
  { key: "historical", label: "Historical", hint: "honest to the era" },
];

interface CampaignScreenProps {
  projectId: string;
  workingDirectory: string;
  modelSettings: ModelSettingsView;
  qaqcAdapterId?: string;
  onExit: () => void;
}

export function CampaignScreen(props: CampaignScreenProps) {
  const [packs, setPacks] = useState<EraPackView[]>([]);
  const [campaigns, setCampaigns] = useState<CampaignRecordView[]>([]);
  const [active, setActive] = useState<CampaignRecordView | undefined>();
  const [turns, setTurns] = useState<CampaignTurnView[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    let alive = true;
    Promise.all([loadCampaignPacks(), loadCampaignSnapshot(props.projectId)])
      .then(([packList, snapshot]) => {
        if (!alive) {
          return;
        }
        setPacks(packList);
        setCampaigns(snapshot.campaigns);
      })
      .catch((problem) => alive && setError(describe(problem)));
    return () => {
      alive = false;
    };
  }, [props.projectId]);

  const openCampaign = (campaign: CampaignRecordView) => {
    setActive(campaign);
    setTurns([]);
    loadCampaignTurns(campaign.id)
      .then(setTurns)
      .catch((problem) => setError(describe(problem)));
  };

  const onCreated = (snapshot: CampaignRecordView[]) => {
    setCampaigns(snapshot);
    const newest = snapshot[snapshot.length - 1];
    if (newest) {
      openCampaign(newest);
    }
  };

  return (
    <div className="stage campaign-stage" data-screen-label="Campaigns">
      <div className="strip">
        <div className="name">
          <strong>Campaigns</strong> / {active ? active.title : "choose your era"}
        </div>
        <div className="right">
          {active && (
            <button className="gchip" onClick={() => setActive(undefined)} type="button">
              All campaigns
            </button>
          )}
          <button className="gchip" onClick={props.onExit} type="button">
            Exit
          </button>
        </div>
      </div>

      {error && <div className="campaign-error mono">{error}</div>}

      {!active && (
        <CampaignHome
          campaigns={campaigns}
          onCreate={onCreated}
          onError={setError}
          onResume={openCampaign}
          packs={packs}
          projectId={props.projectId}
        />
      )}
      {active && (
        <CampaignPlay
          campaign={active}
          modelSettings={props.modelSettings}
          onCampaign={(next) => {
            setActive(next);
            setCampaigns((current) =>
              current.map((item) => (item.id === next.id ? next : item)),
            );
          }}
          onError={setError}
          onTurns={setTurns}
          packs={packs}
          qaqcAdapterId={props.qaqcAdapterId}
          turns={turns}
          workingDirectory={props.workingDirectory}
        />
      )}
    </div>
  );
}

function CampaignHome({
  campaigns,
  onCreate,
  onError,
  onResume,
  packs,
  projectId,
}: {
  campaigns: CampaignRecordView[];
  onCreate: (campaigns: CampaignRecordView[]) => void;
  onError: (message: string) => void;
  onResume: (campaign: CampaignRecordView) => void;
  packs: EraPackView[];
  projectId: string;
}) {
  const [packId, setPackId] = useState("");
  const [scenarioId, setScenarioId] = useState("");
  const [playerName, setPlayerName] = useState("");
  const [rating, setRating] = useState<"story" | "heroic" | "historical">("story");
  const [creating, setCreating] = useState(false);
  const pack = packs.find((item) => item.id === packId);
  const scenario = pack?.scenarios.find((item) => item.id === scenarioId);

  const create = () => {
    if (!pack || !scenario || !playerName.trim() || creating) {
      return;
    }
    setCreating(true);
    createCampaign({
      projectId,
      eraPackId: pack.id,
      scenarioId: scenario.id,
      playerName: playerName.trim(),
      playerRole: "",
      contentRating: rating,
    })
      .then((snapshot) => onCreate(snapshot.campaigns))
      .catch((problem) => onError(describe(problem)))
      .finally(() => setCreating(false));
  };

  return (
    <div className="campaign-home">
      {campaigns.length > 0 && (
        <section>
          <h2 className="campaign-h2">Continue</h2>
          <div className="campaign-cards">
            {campaigns.map((campaign) => (
              <button
                className="campaign-card"
                key={campaign.id}
                onClick={() => onResume(campaign)}
                type="button"
              >
                <b>{campaign.title}</b>
                <span className="mono">
                  {campaign.worldDate} - {campaign.location}
                </span>
                <span className="campaign-card-meta">
                  {campaign.turnCount} scenes - {campaign.contentRating}
                </span>
              </button>
            ))}
          </div>
        </section>
      )}

      <section>
        <h2 className="campaign-h2">New campaign</h2>
        <div className="campaign-cards">
          {packs.map((item) => (
            <button
              className={`campaign-card${item.id === packId ? " on" : ""}`}
              key={item.id}
              onClick={() => {
                setPackId(item.id);
                setScenarioId(item.scenarios[0]?.id ?? "");
              }}
              type="button"
            >
              <b>{item.title}</b>
              <span className="campaign-card-meta">
                {item.scenarios.length} scenario{item.scenarios.length === 1 ? "" : "s"}
              </span>
            </button>
          ))}
        </div>

        {pack && (
          <div className="campaign-cards">
            {pack.scenarios.map((item) => (
              <button
                className={`campaign-card${item.id === scenarioId ? " on" : ""}`}
                key={item.id}
                onClick={() => setScenarioId(item.id)}
                type="button"
              >
                <b>{item.title}</b>
                <span className="mono">
                  {item.startDate} - {item.startLocation}
                </span>
                <span className="campaign-card-meta">
                  Squad: {item.squadNames.join(", ")}
                </span>
              </button>
            ))}
          </div>
        )}

        {scenario && (
          <div className="campaign-create">
            <input
              className="in"
              onChange={(event) => setPlayerName(event.target.value)}
              placeholder="Your character's name"
              value={playerName}
            />
            <div className="seg">
              {RATINGS.map((item) => (
                <button
                  className={rating === item.key ? "on" : ""}
                  key={item.key}
                  onClick={() => setRating(item.key)}
                  title={item.hint}
                  type="button"
                >
                  {item.label}
                </button>
              ))}
            </div>
            <button
              className="btn-send"
              disabled={!playerName.trim() || creating}
              onClick={create}
              type="button"
            >
              {creating ? "Mustering..." : "Begin campaign"}
            </button>
          </div>
        )}
      </section>
    </div>
  );
}

function CampaignPlay({
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

function TurnBlock({ turn }: { turn: CampaignTurnView }) {
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

function DiceBlock({ resolution }: { resolution: TurnResolutionView }) {
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

function characterDot(status: string) {
  return status === "active" ? "success" : status === "dead" ? "danger" : "warn";
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

function describe(problem: unknown): string {
  return problem instanceof Error ? problem.message : String(problem);
}
