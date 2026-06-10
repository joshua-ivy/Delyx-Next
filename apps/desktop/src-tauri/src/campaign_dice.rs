//! App-side action resolution. The app rolls the dice and tells the Game
//! Master what happened; the model narrates an outcome it was given and can
//! never quietly decide the player always wins.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnResolution {
    pub check: String,
    pub roll: Vec<u8>,
    pub stat: i64,
    pub total: i64,
    pub outcome: String,
}

/// Which pack check (if any) the player's action puts at risk. Only checks the
/// era pack actually defines are considered, so packs control their own dials.
pub fn detect_check(player_text: &str, checks: &[String]) -> Option<String> {
    let text = format!(" {} ", player_text.to_lowercase());
    for check in checks {
        let keywords = check_keywords(check);
        if keywords
            .iter()
            .any(|keyword| text.contains(&format!(" {keyword}")) || text.contains(keyword))
        {
            return Some(check.clone());
        }
    }
    None
}

/// 2d6 + stat: 10+ clean success, 7-9 success at a cost, 6- setback. The seed
/// comes from the bridge (wall clock) so the domain stays deterministic.
pub fn resolve(check: &str, stat: i64, seed: u64) -> TurnResolution {
    let mut state = seed;
    let first = roll_d6(&mut state);
    let second = roll_d6(&mut state);
    let total = i64::from(first) + i64::from(second) + stat;
    let outcome = if total >= 10 {
        "success"
    } else if total >= 7 {
        "partial"
    } else {
        "setback"
    };
    TurnResolution {
        check: check.to_string(),
        roll: vec![first, second],
        stat,
        total,
        outcome: outcome.to_string(),
    }
}

/// The player's stat for a check, read from the character sheet JSON.
pub fn sheet_stat(sheet_json: &str, check: &str) -> i64 {
    serde_json::from_str::<serde_json::Value>(sheet_json)
        .ok()
        .and_then(|sheet| sheet.get(check)?.as_i64())
        .unwrap_or(0)
}

fn roll_d6(state: &mut u64) -> u8 {
    // splitmix64 step — tiny, deterministic, no external RNG dependency.
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    (z % 6) as u8 + 1
}

fn check_keywords(check: &str) -> &'static [&'static str] {
    match check {
        "grit" => &[
            "charge",
            "climb",
            "crawl",
            "sprint",
            "push through",
            "hold the line",
            "carry",
            "endure",
            "dig in",
            "wrestle",
            "swim",
            "brace",
        ],
        "aim" => &[
            "shoot",
            "fire",
            "aim",
            "throw",
            "snipe",
            "grenade",
            "take the shot",
            "open fire",
            "blast",
        ],
        "wits" => &[
            "sneak",
            "hide",
            "search",
            "scout",
            "spot",
            "listen for",
            "fix",
            "repair",
            "navigate",
            "disarm",
            "decode",
            "track",
            "slice the",
            "hack",
        ],
        "charm" => &[
            "persuade",
            "convince",
            "negotiate",
            "bluff",
            "calm",
            "rally",
            "barter",
            "plead",
            "inspire",
            "reassure",
            "talk him",
            "talk her",
            "talk them",
            "talk our way",
        ],
        _ => &[],
    }
}
