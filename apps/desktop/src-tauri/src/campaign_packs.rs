use serde::{Deserialize, Serialize};

const WW1_PACK_JSON: &str = include_str!("../packs/ww1/pack.json");
const WW1_SCENARIOS_JSON: &str = include_str!("../packs/ww1/scenarios.json");
const WW1_LORE_MD: &str = include_str!("../packs/ww1/lore.md");
const REVOLUTION_PACK_JSON: &str = include_str!("../packs/revolutionary-war/pack.json");
const REVOLUTION_SCENARIOS_JSON: &str = include_str!("../packs/revolutionary-war/scenarios.json");
const REVOLUTION_LORE_MD: &str = include_str!("../packs/revolutionary-war/lore.md");
const CIVIL_WAR_PACK_JSON: &str = include_str!("../packs/civil-war/pack.json");
const CIVIL_WAR_SCENARIOS_JSON: &str = include_str!("../packs/civil-war/scenarios.json");
const CIVIL_WAR_LORE_MD: &str = include_str!("../packs/civil-war/lore.md");
const WW2_PACK_JSON: &str = include_str!("../packs/ww2/pack.json");
const WW2_SCENARIOS_JSON: &str = include_str!("../packs/ww2/scenarios.json");
const WW2_LORE_MD: &str = include_str!("../packs/ww2/lore.md");
const KOREA_PACK_JSON: &str = include_str!("../packs/korea/pack.json");
const KOREA_SCENARIOS_JSON: &str = include_str!("../packs/korea/scenarios.json");
const KOREA_LORE_MD: &str = include_str!("../packs/korea/lore.md");
const VIETNAM_PACK_JSON: &str = include_str!("../packs/vietnam/pack.json");
const VIETNAM_SCENARIOS_JSON: &str = include_str!("../packs/vietnam/scenarios.json");
const VIETNAM_LORE_MD: &str = include_str!("../packs/vietnam/lore.md");
const STAR_WARS_PACK_JSON: &str = include_str!("../packs/star-wars/pack.json");
const STAR_WARS_SCENARIOS_JSON: &str = include_str!("../packs/star-wars/scenarios.json");
const STAR_WARS_LORE_MD: &str = include_str!("../packs/star-wars/lore.md");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EraPack {
    pub id: String,
    pub title: String,
    pub gm_style: String,
    pub checks: Vec<String>,
    pub rating_overlays: RatingOverlays,
    #[serde(default)]
    pub scenarios: Vec<EraScenario>,
    #[serde(default)]
    pub lore: Vec<LoreChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoreChunk {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingOverlays {
    pub story: String,
    pub heroic: String,
    pub historical: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EraScenario {
    pub id: String,
    pub title: String,
    pub start_date: String,
    pub start_location: String,
    pub opening: String,
    pub squad: Vec<ScenarioCharacter>,
    #[serde(default)]
    pub timeline_pressure: Vec<String>,
    #[serde(default)]
    pub player_role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioCharacter {
    pub name: String,
    pub role: String,
    #[serde(rename = "trait")]
    pub trait_label: String,
}

pub fn builtin_packs() -> Result<Vec<EraPack>, String> {
    Ok(vec![
        load_pack(
            REVOLUTION_PACK_JSON,
            REVOLUTION_SCENARIOS_JSON,
            REVOLUTION_LORE_MD,
        )?,
        load_pack(
            CIVIL_WAR_PACK_JSON,
            CIVIL_WAR_SCENARIOS_JSON,
            CIVIL_WAR_LORE_MD,
        )?,
        load_pack(WW1_PACK_JSON, WW1_SCENARIOS_JSON, WW1_LORE_MD)?,
        load_pack(WW2_PACK_JSON, WW2_SCENARIOS_JSON, WW2_LORE_MD)?,
        load_pack(KOREA_PACK_JSON, KOREA_SCENARIOS_JSON, KOREA_LORE_MD)?,
        load_pack(VIETNAM_PACK_JSON, VIETNAM_SCENARIOS_JSON, VIETNAM_LORE_MD)?,
        load_pack(
            STAR_WARS_PACK_JSON,
            STAR_WARS_SCENARIOS_JSON,
            STAR_WARS_LORE_MD,
        )?,
    ])
}

pub fn find_pack(packs: &[EraPack], pack_id: &str) -> Result<EraPack, String> {
    packs
        .iter()
        .find(|pack| pack.id == pack_id)
        .cloned()
        .ok_or_else(|| format!("Unknown era pack: {pack_id}"))
}

pub fn find_scenario(pack: &EraPack, scenario_id: &str) -> Result<EraScenario, String> {
    pack.scenarios
        .iter()
        .find(|scenario| scenario.id == scenario_id)
        .cloned()
        .ok_or_else(|| format!("Unknown scenario {scenario_id} in era pack {}.", pack.id))
}

fn load_pack(pack_json: &str, scenarios_json: &str, lore_md: &str) -> Result<EraPack, String> {
    let mut pack: EraPack = serde_json::from_str(pack_json)
        .map_err(|error| format!("Era pack manifest failed to parse: {error}"))?;
    pack.scenarios = serde_json::from_str(scenarios_json)
        .map_err(|error| format!("Era pack scenarios failed to parse: {error}"))?;
    pack.lore = parse_lore_markdown(lore_md);
    Ok(pack)
}

/// Lore files are plain markdown; every `## Heading` starts a chunk. Chunks
/// are injected into the GM prompt by relevance, never wholesale.
pub fn parse_lore_markdown(markdown: &str) -> Vec<LoreChunk> {
    let mut chunks = Vec::new();
    let mut title: Option<String> = None;
    let mut body = String::new();
    for line in markdown.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(current) = title.take() {
                push_chunk(&mut chunks, current, &body);
            }
            title = Some(heading.trim().to_string());
            body.clear();
        } else if title.is_some() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some(current) = title {
        push_chunk(&mut chunks, current, &body);
    }
    chunks
}

fn push_chunk(chunks: &mut Vec<LoreChunk>, title: String, body: &str) {
    let text = body.trim().to_string();
    if !text.is_empty() {
        chunks.push(LoreChunk { title, text });
    }
}
