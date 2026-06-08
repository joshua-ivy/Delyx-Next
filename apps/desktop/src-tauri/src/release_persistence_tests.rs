#[cfg(test)]
mod tests {
    use crate::release::{default_release_profile, export_support_bundle};
    use crate::release_persistence::{
        load_profile_from_path, load_support_bundle_from_path, save_profile_to_path,
        save_support_bundle_to_path,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn release_profile_and_support_bundle_survive_sqlite_reload() {
        let path = temp_path("release");
        let mut profile = default_release_profile();
        profile.signing.certificate_thumbprint = Some("thumbprint".to_string());
        profile.signing.digest_algorithm = Some("sha256".to_string());
        profile.update_metadata.published = true;
        let bundle = export_support_bundle(
            &profile,
            vec![("OPENAI_API_KEY", "sk-test"), ("workspace", "C:/work")],
            vec![
                ("runtime", "ok"),
                ("provider", "Authorization: Bearer abc123"),
            ],
            42,
        );

        save_profile_to_path(&profile, &path).unwrap();
        save_support_bundle_to_path(&bundle, &path).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let loaded_profile = load_profile_from_path(&path).unwrap().unwrap();
        let loaded_bundle = load_support_bundle_from_path(&path).unwrap().unwrap();
        assert_eq!(
            loaded_profile.signing.digest_algorithm.as_deref(),
            Some("sha256")
        );
        assert!(loaded_profile.update_metadata.published);
        assert_eq!(loaded_bundle.config_summary[0].value, "[redacted]");
        assert_eq!(loaded_bundle.config_summary[1].value, "C:/work");
        assert_eq!(loaded_bundle.logs[1].line, "[redacted log line]");
        assert!(!format!("{:?}", loaded_bundle).contains("sk-test"));
        assert!(!format!("{:?}", loaded_bundle).contains("abc123"));
        let _ = fs::remove_file(path);
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
