use crate::campaign::{CampaignInput, CampaignStore, CharacterInput, CharacterKind, TurnInput};
use crate::campaign_bridge_requests::{
    CampaignCreateRequest, CampaignMemoryCommitRequest, CampaignSetRatingRequest,
    CampaignTurnCommitRequest, CampaignTurnQaqcCommitRequest,
};
use crate::campaign_bridge_views::{
    campaign_snapshot_from_store, campaign_view, turn_view, CampaignSnapshotView,
    CampaignTurnCommitView, CampaignTurnView, CampaignView,
};
use crate::campaign_delta::{apply_delta, split_narration_and_delta};
use crate::campaign_packs::{builtin_packs, find_pack, find_scenario, EraPack};
use crate::campaign_persistence::{parse_content_rating, parse_qaqc_status};

pub fn commit_turn_record(
    store: &mut CampaignStore,
    request: CampaignTurnCommitRequest,
) -> Result<CampaignTurnCommitView, String> {
    if request.player_text.trim().is_empty() {
        return Err("A campaign turn requires player input.".to_string());
    }
    let campaign = store
        .campaign(&request.campaign_id)
        .map_err(|error| format!("{error:?}"))?;
    if campaign.status != crate::campaign::CampaignStatus::Active {
        return Err("This campaign is no longer active.".to_string());
    }

    let (narration, delta) = split_narration_and_delta(&request.model_text);
    if narration.trim().is_empty() {
        return Err("A campaign turn requires the Game Master narration.".to_string());
    }
    let turn_index = store.turns_for(&request.campaign_id).len();

    let state_delta_json = match &delta {
        Some(proposal) => {
            let applied = apply_delta(
                store,
                &request.campaign_id,
                turn_index,
                proposal,
                &request.created_at,
            )?;
            if applied.is_empty() && applied.rejected.is_empty() {
                "{}".to_string()
            } else {
                serde_json::to_string(&applied)
                    .map_err(|error| format!("delta serialization failed: {error}"))?
            }
        }
        None => "{}".to_string(),
    };
    let resolution_json = match &request.resolution {
        Some(resolution) => serde_json::to_string(resolution)
            .map_err(|error| format!("resolution serialization failed: {error}"))?,
        None => "{}".to_string(),
    };

    let turn = store
        .append_turn(
            &request.campaign_id,
            TurnInput {
                player_text: request.player_text.trim().to_string(),
                resolution_json,
                narration,
                state_delta_json,
                created_at: request.created_at,
            },
        )
        .map_err(|error| format!("{error:?}"))?;

    let campaign = store
        .campaign(&request.campaign_id)
        .map_err(|error| format!("{error:?}"))?
        .clone();
    Ok(CampaignTurnCommitView {
        turn: turn_view(&turn),
        campaign: campaign_view(store, &campaign),
    })
}

pub fn commit_memory_record(
    store: &mut CampaignStore,
    request: CampaignMemoryCommitRequest,
) -> Result<CampaignView, String> {
    if request.summary.trim().is_empty() {
        return Err("A chronicle update requires summary text.".to_string());
    }
    store
        .set_memory_summary(&request.campaign_id, &request.summary, &request.created_at)
        .map_err(|error| format!("{error:?}"))?;
    let campaign = store
        .campaign(&request.campaign_id)
        .map_err(|error| format!("{error:?}"))?
        .clone();
    Ok(campaign_view(store, &campaign))
}

pub fn commit_turn_qaqc_record(
    store: &mut CampaignStore,
    request: CampaignTurnQaqcCommitRequest,
) -> Result<CampaignTurnView, String> {
    let status = parse_qaqc_status(&request.qaqc_status)?;
    let notes = request
        .qaqc_notes
        .map(|notes| notes.trim().to_string())
        .filter(|notes| !notes.is_empty());
    store
        .set_turn_qaqc(&request.campaign_id, request.turn_index, status, notes)
        .map_err(|error| format!("{error:?}"))?;
    let turn = store
        .turns_for(&request.campaign_id)
        .into_iter()
        .find(|turn| turn.turn_index == request.turn_index)
        .cloned()
        .ok_or_else(|| "Turn not found after QA/QC update.".to_string())?;
    Ok(turn_view(&turn))
}

pub fn set_rating_record(
    store: &mut CampaignStore,
    request: CampaignSetRatingRequest,
) -> Result<CampaignView, String> {
    if !request.parent_confirmed {
        return Err("Changing the content rating requires parent confirmation.".to_string());
    }
    let rating = parse_content_rating(&request.content_rating)?;
    store
        .set_content_rating(&request.campaign_id, rating, &request.updated_at)
        .map_err(|error| format!("{error:?}"))?;
    let campaign = store
        .campaign(&request.campaign_id)
        .map_err(|error| format!("{error:?}"))?
        .clone();
    Ok(campaign_view(store, &campaign))
}

pub fn create_campaign_record(
    store: &mut CampaignStore,
    request: CampaignCreateRequest,
) -> Result<CampaignSnapshotView, String> {
    if request.player_name.trim().is_empty() {
        return Err("Campaign creation requires a player character name.".to_string());
    }
    let content_rating = parse_content_rating(&request.content_rating)?;
    let packs = builtin_packs()?;
    let pack = find_pack(&packs, &request.era_pack_id)?;
    let scenario = find_scenario(&pack, &request.scenario_id)?;

    let title = request
        .title
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| scenario.title.clone());
    let player_role = if request.player_role.trim().is_empty() {
        scenario
            .player_role
            .clone()
            .unwrap_or_else(|| "rifleman".to_string())
    } else {
        request.player_role
    };

    let mut characters = vec![CharacterInput {
        kind: CharacterKind::Player,
        name: request.player_name,
        role: player_role,
        sheet_json: starting_sheet_json(&pack),
        notes: String::new(),
    }];
    for squad_member in &scenario.squad {
        characters.push(CharacterInput {
            kind: CharacterKind::Npc,
            name: squad_member.name.clone(),
            role: squad_member.role.clone(),
            sheet_json: starting_sheet_json(&pack),
            notes: squad_member.trait_label.clone(),
        });
    }

    let project_id = request.project_id.clone();
    store
        .create_campaign(
            CampaignInput {
                project_id: request.project_id,
                era_pack_id: pack.id.clone(),
                scenario_id: Some(scenario.id.clone()),
                title,
                content_rating,
                world_date: scenario.start_date.clone(),
                location: scenario.start_location.clone(),
                created_at: request.created_at,
            },
            characters,
        )
        .map_err(|error| format!("{error:?}"))?;
    Ok(campaign_snapshot_from_store(store, &project_id))
}

fn starting_sheet_json(pack: &EraPack) -> String {
    let sheet: serde_json::Map<String, serde_json::Value> = pack
        .checks
        .iter()
        .map(|check| (check.clone(), serde_json::Value::from(0)))
        .collect();
    serde_json::Value::Object(sheet).to_string()
}
