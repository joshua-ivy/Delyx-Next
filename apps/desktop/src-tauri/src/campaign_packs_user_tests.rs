#[cfg(test)]
mod tests {
    use crate::campaign_packs::builtin_packs;
    use crate::campaign_packs_user::{load_user_packs, merge_packs};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const PACK_JSON: &str = r#"{
        "id": "war-of-1812",
        "title": "War of 1812",
        "gmStyle": "Frigates and frontier militias; keep stakes personal.",
        "checks": ["Dates align with 1812-1815."],
        "ratingOverlays": {
            "story": "Keep violence off-screen.",
            "heroic": "Cinematic, low gore.",
            "historical": "Grounded but never gratuitous."
        }
    }"#;

    const SCENARIOS_JSON: &str = r#"[{
        "id": "lake-erie",
        "title": "Put-in-Bay",
        "startDate": "1813-09-10",
        "startLocation": "Lake Erie",
        "opening": "Signal flags climb the mast of the Lawrence.",
        "squad": [
            {"name": "Eli", "role": "Gunner", "trait": "steady hands"}
        ]
    }]"#;

    #[test]
    fn loads_pack_folder_with_scenarios_and_lore() {
        let dir = temp_dir("load");
        let pack_dir = dir.join("war-of-1812");
        fs::create_dir_all(&pack_dir).unwrap();
        fs::write(pack_dir.join("pack.json"), PACK_JSON).unwrap();
        fs::write(pack_dir.join("scenarios.json"), SCENARIOS_JSON).unwrap();
        fs::write(pack_dir.join("lore.md"), "## Lake Erie\nNine ships.\n").unwrap();

        let packs = load_user_packs(&dir);
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].id, "war-of-1812");
        assert_eq!(packs[0].scenarios.len(), 1);
        assert_eq!(packs[0].scenarios[0].squad[0].trait_label, "steady hands");
        assert_eq!(packs[0].lore.len(), 1);
        assert_eq!(packs[0].lore[0].title, "Lake Erie");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn pack_json_alone_is_enough() {
        let dir = temp_dir("bare");
        let pack_dir = dir.join("war-of-1812");
        fs::create_dir_all(&pack_dir).unwrap();
        fs::write(pack_dir.join("pack.json"), PACK_JSON).unwrap();

        let packs = load_user_packs(&dir);
        assert_eq!(packs.len(), 1);
        assert!(packs[0].scenarios.is_empty());
        assert!(packs[0].lore.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn broken_pack_is_skipped_without_blocking_others() {
        let dir = temp_dir("broken");
        let bad = dir.join("a-broken");
        fs::create_dir_all(&bad).unwrap();
        fs::write(bad.join("pack.json"), "{not json").unwrap();
        let good = dir.join("b-good");
        fs::create_dir_all(&good).unwrap();
        fs::write(good.join("pack.json"), PACK_JSON).unwrap();

        let packs = load_user_packs(&dir);
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].id, "war-of-1812");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_directory_means_no_user_packs() {
        let dir = temp_dir("missing");
        assert!(load_user_packs(&dir).is_empty());
    }

    #[test]
    fn folders_without_manifest_are_ignored() {
        let dir = temp_dir("no-manifest");
        fs::create_dir_all(dir.join("notes")).unwrap();
        assert!(load_user_packs(&dir).is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn user_pack_overrides_builtin_with_same_id() {
        let mut packs = builtin_packs().unwrap();
        let count = packs.len();
        let mut replacement = packs
            .iter()
            .find(|pack| pack.id == "ww1")
            .cloned()
            .unwrap();
        replacement.title = "WW1 (House Rules)".to_string();
        merge_packs(&mut packs, vec![replacement]);
        assert_eq!(packs.len(), count);
        let ww1 = packs.iter().find(|pack| pack.id == "ww1").unwrap();
        assert_eq!(ww1.title, "WW1 (House Rules)");
    }

    #[test]
    fn new_user_pack_appends_after_builtins() {
        let mut packs = builtin_packs().unwrap();
        let count = packs.len();
        let mut extra = packs[0].clone();
        extra.id = "war-of-1812".to_string();
        merge_packs(&mut packs, vec![extra]);
        assert_eq!(packs.len(), count + 1);
        assert_eq!(packs.last().unwrap().id, "war-of-1812");
    }

    fn temp_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-packs-{name}-{stamp}"))
    }
}
