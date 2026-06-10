use crate::campaign_packs::{builtin_packs, parse_lore_markdown, EraPack};
use std::path::{Path, PathBuf};

/// User packs live next to the app database so adding a war never requires a
/// rebuild: drop `<dir>/<pack>/pack.json` (plus optional scenarios.json and
/// lore.md) and the era appears in the list.
pub fn user_packs_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("DELYX_NEXT_PACKS_DIR") {
        return PathBuf::from(path);
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data)
            .join("Delyx Next")
            .join("packs");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("delyx-packs")
}

pub fn available_packs() -> Result<Vec<EraPack>, String> {
    let directory = user_packs_dir();
    let _ = std::fs::create_dir_all(&directory);
    let mut packs = builtin_packs()?;
    merge_packs(&mut packs, load_user_packs(&directory));
    Ok(packs)
}

/// A user pack with a builtin id replaces the builtin; new ids append.
pub fn merge_packs(packs: &mut Vec<EraPack>, user_packs: Vec<EraPack>) {
    for user_pack in user_packs {
        if let Some(existing) = packs.iter_mut().find(|pack| pack.id == user_pack.id) {
            *existing = user_pack;
        } else {
            packs.push(user_pack);
        }
    }
}

/// Broken packs are skipped so one bad folder never empties the era list.
pub fn load_user_packs(directory: &Path) -> Vec<EraPack> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut folders: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();
    let mut packs = Vec::new();
    for folder in folders {
        match load_user_pack(&folder) {
            Ok(Some(pack)) => packs.push(pack),
            Ok(None) => {}
            Err(error) => {
                eprintln!("Skipping campaign pack {}: {error}", folder.display());
            }
        }
    }
    packs
}

fn load_user_pack(directory: &Path) -> Result<Option<EraPack>, String> {
    let manifest = directory.join("pack.json");
    if !manifest.is_file() {
        return Ok(None);
    }
    let pack_json = read_pack_file(&manifest)?;
    let mut pack: EraPack = serde_json::from_str(&pack_json)
        .map_err(|error| format!("pack.json failed to parse: {error}"))?;
    let scenarios = directory.join("scenarios.json");
    if scenarios.is_file() {
        let scenarios_json = read_pack_file(&scenarios)?;
        pack.scenarios = serde_json::from_str(&scenarios_json)
            .map_err(|error| format!("scenarios.json failed to parse: {error}"))?;
    }
    let lore = directory.join("lore.md");
    if lore.is_file() {
        pack.lore = parse_lore_markdown(&read_pack_file(&lore)?);
    }
    Ok(Some(pack))
}

fn read_pack_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|error| format!("{} could not be read: {error}", path.display()))
}
