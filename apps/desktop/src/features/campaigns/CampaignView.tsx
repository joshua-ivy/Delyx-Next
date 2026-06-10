import { useEffect, useState } from "react";
import type { ModelSettingsView } from "../models/modelTypes";
import {
  loadCampaignPacks,
  loadCampaignSnapshot,
  loadCampaignTurns,
} from "./campaignClient";
import { CampaignHome } from "./CampaignHome";
import { CampaignPlay } from "./CampaignPlay";
import type {
  CampaignTurnView,
  CampaignView as CampaignRecordView,
  EraPackView,
} from "./campaignTypes";
import { describe } from "./campaignViewShared";

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
