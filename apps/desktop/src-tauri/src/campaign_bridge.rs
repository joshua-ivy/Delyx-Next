use crate::campaign::{
    Campaign, CampaignInput, CampaignStore, CharacterInput, CharacterKind, TurnInput,
};
use crate::campaign_delta::{apply_delta, split_narration_and_delta};
use crate::campaign_dice::{detect_check, resolve, sheet_stat, TurnResolution};
use crate::campaign_packs::{builtin_packs, find_pack, find_scenario, EraPack};
use crate::campaign_persistence::{
    campaign_status_key, character_kind_key, character_status_key, content_rating_key,
    parse_content_rating, parse_qaqc_status, qaqc_status_key,
};
use crate::campaign_prompt::{build_turn_messages, rating_overlay, GmPromptMessage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct CampaignBridgeState {
    store: Mutex<CampaignStore>,
    database_path: Option<PathBuf>,
}

impl CampaignBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        let store = crate::campaign_persistence::load_from_path(&database_path)?;
        Ok(Self {
            store: Mutex::new(store),
            database_path: Some(database_path),
        })
    }

    fn save_if_persistent(&self, store: &CampaignStore) -> Result<(), String> {
        match &self.database_path {
            Some(path) => crate::campaign_persistence::save_to_path(store, path),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignCreateRequest {
    pub project_id: String,
    pub era_pack_id: String,
    pub scenario_id: String,
    pub player_name: String,
    pub player_role: String,
    pub content_rating: String,
    pub created_at: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EraPackView {
    pub id: String,
    pub title: String,
    pub gm_style: String,
    pub checks: Vec<String>,
    pub scenarios: Vec<EraScenarioView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EraScenarioView {
    pub id: String,
    pub title: String,
    pub start_date: String,
    pub start_location: String,
    pub opening: String,
    pub squad_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignView {
    pub id: String,
    pub project_id: String,
    pub era_pack_id: String,
    pub scenario_id: Option<String>,
    pub title: String,
    pub status: String,
    pub content_rating: String,
    pub world_date: String,
    pub location: String,
    pub memory_summary: String,
    pub created_at: String,
    pub updated_at: String,
    pub characters: Vec<CampaignCharacterView>,
    pub turn_count: usize,
    pub events: Vec<CampaignEventView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignCharacterView {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub sheet_json: String,
    pub inventory_json: String,
    pub bonds_json: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnView {
    pub turn_index: usize,
    pub player_text: String,
    pub resolution_json: String,
    pub narration: String,
    pub state_delta_json: String,
    pub qaqc_status: String,
    pub qaqc_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignEventView {
    pub id: String,
    pub turn_index: usize,
    pub kind: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignSnapshotView {
    pub campaigns: Vec<CampaignView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnPromptRequest {
    pub campaign_id: String,
    pub player_text: String,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnPromptView {
    pub messages: Vec<GmPromptMessage>,
    pub resolution: Option<TurnResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnCommitRequest {
    pub campaign_id: String,
    pub player_text: String,
    pub model_text: String,
    #[serde(default)]
    pub resolution: Option<TurnResolution>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnCommitView {
    pub turn: CampaignTurnView,
    pub campaign: CampaignView,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignMemoryCommitRequest {
    pub campaign_id: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignQaqcPromptRequest {
    pub campaign_id: String,
    pub turn_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignTurnQaqcCommitRequest {
    pub campaign_id: String,
    pub turn_index: usize,
    pub qaqc_status: String,
    #[serde(default)]
    pub qaqc_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignSetRatingRequest {
    pub campaign_id: String,
    pub content_rating: String,
    pub parent_confirmed: bool,
    pub updated_at: String,
}

#[tauri::command]
pub fn campaign_pack_list() -> Result<Vec<EraPackView>, String> {
    Ok(builtin_packs()?.iter().map(pack_view).collect())
}

#[tauri::command]
pub fn campaign_create(
    state: tauri::State<CampaignBridgeState>,
    request: CampaignCreateRequest,
) -> Result<CampaignSnapshotView, String> {
    let mut store = lock_store(&state)?;
    let view = create_campaign_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(view)
}

#[tauri::command]
pub fn campaign_snapshot(
    state: tauri::State<CampaignBridgeState>,
    project_id: String,
) -> Result<CampaignSnapshotView, String> {
    let store = lock_store(&state)?;
    Ok(campaign_snapshot_from_store(&store, &project_id))
}

#[tauri::command]
pub fn campaign_turn_prompt(
    state: tauri::State<CampaignBridgeState>,
    request: CampaignTurnPromptRequest,
) -> Result<CampaignTurnPromptView, String> {
    let store = lock_store(&state)?;
    let seed = request.seed.unwrap_or_else(wall_clock_seed);
    build_turn_prompt_record(&store, request, seed)
}

#[tauri::command]
pub fn campaign_turn_commit(
    state: tauri::State<CampaignBridgeState>,
    request: CampaignTurnCommitRequest,
) -> Result<CampaignTurnCommitView, String> {
    let mut store = lock_store(&state)?;
    let view = commit_turn_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(view)
}

#[tauri::command]
pub fn campaign_turns(
    state: tauri::State<CampaignBridgeState>,
    campaign_id: String,
) -> Result<Vec<CampaignTurnView>, String> {
    let store = lock_store(&state)?;
    Ok(campaign_turn_views(&store, &campaign_id))
}

#[tauri::command]
pub fn campaign_memory_prompt(
    state: tauri::State<CampaignBridgeState>,
    campaign_id: String,
) -> Result<Vec<GmPromptMessage>, String> {
    let store = lock_store(&state)?;
    build_memory_prompt_record(&store, &campaign_id)
}

#[tauri::command]
pub fn campaign_memory_commit(
    state: tauri::State<CampaignBridgeState>,
    request: CampaignMemoryCommitRequest,
) -> Result<CampaignView, String> {
    let mut store = lock_store(&state)?;
    let view = commit_memory_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(view)
}

#[tauri::command]
pub fn campaign_qaqc_prompt(
    state: tauri::State<CampaignBridgeState>,
    request: CampaignQaqcPromptRequest,
) -> Result<String, String> {
    let store = lock_store(&state)?;
    build_qaqc_prompt_record(&store, &request)
}

#[tauri::command]
pub fn campaign_turn_qaqc_commit(
    state: tauri::State<CampaignBridgeState>,
    request: CampaignTurnQaqcCommitRequest,
) -> Result<CampaignTurnView, String> {
    let mut store = lock_store(&state)?;
    let view = commit_turn_qaqc_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(view)
}

#[tauri::command]
pub fn campaign_set_rating(
    state: tauri::State<CampaignBridgeState>,
    request: CampaignSetRatingRequest,
) -> Result<CampaignView, String> {
    let mut store = lock_store(&state)?;
    let view = set_rating_record(&mut store, request)?;
    state.save_if_persistent(&store)?;
    Ok(view)
}

fn lock_store<'a>(
    state: &'a tauri::State<CampaignBridgeState>,
) -> Result<std::sync::MutexGuard<'a, CampaignStore>, String> {
    state
        .store
        .lock()
        .map_err(|_| "Campaign bridge lock failed.".to_string())
}

fn wall_clock_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as u64 ^ duration.as_secs())
        .unwrap_or(0x5eed)
}

pub fn build_turn_prompt_record(
    store: &CampaignStore,
    request: CampaignTurnPromptRequest,
    seed: u64,
) -> Result<CampaignTurnPromptView, String> {
    let packs = builtin_packs()?;
    let campaign = store
        .campaign(&request.campaign_id)
        .map_err(|error| format!("{error:?}"))?;
    let pack = find_pack(&packs, &campaign.era_pack_id)?;
    let resolution = detect_check(&request.player_text, &pack.checks).map(|check| {
        let stat = store
            .player_character(&request.campaign_id)
            .map(|player| sheet_stat(&player.sheet_json, &check))
            .unwrap_or(0);
        resolve(&check, stat, seed)
    });
    let messages = build_turn_messages(
        store,
        &packs,
        &request.campaign_id,
        &request.player_text,
        resolution.as_ref(),
    )?;
    Ok(CampaignTurnPromptView {
        messages,
        resolution,
    })
}

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

pub fn build_memory_prompt_record(
    store: &CampaignStore,
    campaign_id: &str,
) -> Result<Vec<GmPromptMessage>, String> {
    let campaign = store
        .campaign(campaign_id)
        .map_err(|error| format!("{error:?}"))?;
    let turns = store.turns_for(campaign_id);
    if turns.is_empty() {
        return Err("There is nothing to summarize yet.".to_string());
    }
    let mut chronicle = String::new();
    if !campaign.memory_summary.trim().is_empty() {
        chronicle.push_str(&format!(
            "Previous chronicle:\n{}\n\n",
            campaign.memory_summary
        ));
    }
    chronicle.push_str("Turns to fold in:\n");
    for turn in &turns {
        chronicle.push_str(&format!(
            "Turn {} - Player: {}\nGM: {}\n",
            turn.turn_index, turn.player_text, turn.narration
        ));
    }
    Ok(vec![
        GmPromptMessage {
            role: "system".to_string(),
            content: "You maintain the chronicle of a roleplaying campaign. Rewrite it as a \
single 150-250 word recap in past tense: the arc so far, promises and grudges, unresolved \
threads, and the emotional state of the squad. Keep every concrete fact (names, wounds, \
deaths, items, places). Output only the recap text."
                .to_string(),
        },
        GmPromptMessage {
            role: "user".to_string(),
            content: chronicle,
        },
    ])
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

pub fn build_qaqc_prompt_record(
    store: &CampaignStore,
    request: &CampaignQaqcPromptRequest,
) -> Result<String, String> {
    let campaign = store
        .campaign(&request.campaign_id)
        .map_err(|error| format!("{error:?}"))?;
    let turn = store
        .turns_for(&request.campaign_id)
        .into_iter()
        .find(|turn| turn.turn_index == request.turn_index)
        .cloned()
        .ok_or_else(|| "Turn not found for QA/QC review.".to_string())?;
    let packs = builtin_packs()?;
    let pack = find_pack(&packs, &campaign.era_pack_id)?;

    let mut prompt = format!(
        "You are the continuity and historical-accuracy reviewer for a \"{}\" roleplaying \
campaign. Review ONE Game Master scene against canon. Be strict about anachronisms (wrong-era \
weapons, technology, slang), contradictions of recorded facts, and content-rating violations. \
Ignore style.\n\nContent rating \"{}\": {}\n\nWorld date: {} | Location: {}\n\nRoster:\n",
        pack.title,
        content_rating_key(campaign.content_rating),
        rating_overlay(&pack, campaign.content_rating),
        campaign.world_date,
        campaign.location,
    );
    for character in store.characters_for(&request.campaign_id) {
        prompt.push_str(&format!(
            "- {} ({}) - {}\n",
            character.name,
            character.role,
            character_status_key(character.status)
        ));
    }
    prompt.push_str("\nSettled facts:\n");
    let events = store.events_for(&request.campaign_id);
    if events.is_empty() {
        prompt.push_str("- none recorded yet\n");
    }
    for event in events {
        prompt.push_str(&format!(
            "- (turn {}) {}\n",
            event.turn_index, event.summary
        ));
    }
    prompt.push_str(&format!(
        "\nScene to review (turn {}):\n{}\n\nReply with exactly one line \"VERDICT: clean\" if \
the scene honors canon, era, and rating, or \"VERDICT: issues\" followed by one bullet per \
problem, each a single line starting with \"- \".",
        turn.turn_index, turn.narration
    ));
    Ok(prompt)
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

pub fn campaign_snapshot_from_store(
    store: &CampaignStore,
    project_id: &str,
) -> CampaignSnapshotView {
    CampaignSnapshotView {
        campaigns: store
            .campaigns_for_project(project_id)
            .into_iter()
            .map(|campaign| campaign_view(store, campaign))
            .collect(),
    }
}

pub fn campaign_turn_views(store: &CampaignStore, campaign_id: &str) -> Vec<CampaignTurnView> {
    store
        .turns_for(campaign_id)
        .into_iter()
        .map(turn_view)
        .collect()
}

fn turn_view(turn: &crate::campaign::CampaignTurn) -> CampaignTurnView {
    CampaignTurnView {
        turn_index: turn.turn_index,
        player_text: turn.player_text.clone(),
        resolution_json: turn.resolution_json.clone(),
        narration: turn.narration.clone(),
        state_delta_json: turn.state_delta_json.clone(),
        qaqc_status: qaqc_status_key(turn.qaqc_status).to_string(),
        qaqc_notes: turn.qaqc_notes.clone(),
        created_at: turn.created_at.clone(),
    }
}

fn campaign_view(store: &CampaignStore, campaign: &Campaign) -> CampaignView {
    CampaignView {
        id: campaign.id.clone(),
        project_id: campaign.project_id.clone(),
        era_pack_id: campaign.era_pack_id.clone(),
        scenario_id: campaign.scenario_id.clone(),
        title: campaign.title.clone(),
        status: campaign_status_key(campaign.status).to_string(),
        content_rating: content_rating_key(campaign.content_rating).to_string(),
        world_date: campaign.world_date.clone(),
        location: campaign.location.clone(),
        memory_summary: campaign.memory_summary.clone(),
        created_at: campaign.created_at.clone(),
        updated_at: campaign.updated_at.clone(),
        characters: store
            .characters_for(&campaign.id)
            .into_iter()
            .map(|character| CampaignCharacterView {
                id: character.id.clone(),
                kind: character_kind_key(character.kind).to_string(),
                name: character.name.clone(),
                role: character.role.clone(),
                status: character_status_key(character.status).to_string(),
                sheet_json: character.sheet_json.clone(),
                inventory_json: character.inventory_json.clone(),
                bonds_json: character.bonds_json.clone(),
                notes: character.notes.clone(),
            })
            .collect(),
        turn_count: store.turns_for(&campaign.id).len(),
        events: store
            .events_for(&campaign.id)
            .into_iter()
            .map(|event| CampaignEventView {
                id: event.id.clone(),
                turn_index: event.turn_index,
                kind: event.kind.clone(),
                summary: event.summary.clone(),
                created_at: event.created_at.clone(),
            })
            .collect(),
    }
}

fn pack_view(pack: &EraPack) -> EraPackView {
    EraPackView {
        id: pack.id.clone(),
        title: pack.title.clone(),
        gm_style: pack.gm_style.clone(),
        checks: pack.checks.clone(),
        scenarios: pack
            .scenarios
            .iter()
            .map(|scenario| EraScenarioView {
                id: scenario.id.clone(),
                title: scenario.title.clone(),
                start_date: scenario.start_date.clone(),
                start_location: scenario.start_location.clone(),
                opening: scenario.opening.clone(),
                squad_names: scenario
                    .squad
                    .iter()
                    .map(|member| member.name.clone())
                    .collect(),
            })
            .collect(),
    }
}

fn starting_sheet_json(pack: &EraPack) -> String {
    let sheet: serde_json::Map<String, serde_json::Value> = pack
        .checks
        .iter()
        .map(|check| (check.clone(), serde_json::Value::from(0)))
        .collect();
    serde_json::Value::Object(sheet).to_string()
}
