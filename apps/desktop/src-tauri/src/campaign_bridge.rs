use crate::campaign::CampaignStore;
pub use crate::campaign_bridge_prompts::{
    build_memory_prompt_record, build_qaqc_prompt_record, build_turn_prompt_record,
};
pub use crate::campaign_bridge_records::{
    commit_memory_record, commit_turn_qaqc_record, commit_turn_record, create_campaign_record,
    set_rating_record,
};
pub use crate::campaign_bridge_requests::{
    CampaignCreateRequest, CampaignMemoryCommitRequest, CampaignQaqcPromptRequest,
    CampaignSetRatingRequest, CampaignTurnCommitRequest, CampaignTurnPromptRequest,
    CampaignTurnQaqcCommitRequest,
};
use crate::campaign_bridge_views::pack_view;
pub use crate::campaign_bridge_views::{
    campaign_snapshot_from_store, campaign_turn_views, CampaignCharacterView, CampaignEventView,
    CampaignSnapshotView, CampaignTurnCommitView, CampaignTurnPromptView, CampaignTurnView,
    CampaignView, EraPackView, EraScenarioView,
};
use crate::campaign_packs::builtin_packs;
use crate::campaign_prompt::GmPromptMessage;
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
