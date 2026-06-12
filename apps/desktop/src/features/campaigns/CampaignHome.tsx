import { useEffect, useState } from "react";
import { createCampaign, loadCampaignPackFolder } from "./campaignClient";
import type { CampaignView as CampaignRecordView, EraPackView } from "./campaignTypes";
import { RATINGS, describe } from "./campaignViewShared";

export function CampaignHome({
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
  const [playerTrait, setPlayerTrait] = useState("");
  const [rating, setRating] = useState<"story" | "heroic" | "historical">("story");
  const [creating, setCreating] = useState(false);
  const [packFolder, setPackFolder] = useState("");
  useEffect(() => {
    loadCampaignPackFolder().then(setPackFolder).catch(() => setPackFolder(""));
  }, []);
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
      playerTrait: playerTrait || undefined,
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
                setPlayerTrait("");
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

        {pack && scenario && (
          <div className="campaign-create">
            <input
              className="in"
              onChange={(event) => setPlayerName(event.target.value)}
              placeholder="Your character's name"
              value={playerName}
            />
            <div className="seg" title="Your specialty rolls at +2">
              <button
                className={playerTrait === "" ? "on" : ""}
                onClick={() => setPlayerTrait("")}
                type="button"
              >
                Balanced
              </button>
              {pack.checks.map((check) => (
                <button
                  className={playerTrait === check ? "on" : ""}
                  key={check}
                  onClick={() => setPlayerTrait(check)}
                  type="button"
                >
                  {check.charAt(0).toUpperCase() + check.slice(1)} +2
                </button>
              ))}
            </div>
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

        {packFolder && (
          <p className="campaign-pack-hint">
            Add your own wars: drop a pack folder (pack.json, scenarios.json, lore.md) into{" "}
            <span className="mono">{packFolder}</span>
          </p>
        )}
      </section>
    </div>
  );
}
