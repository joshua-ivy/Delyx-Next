#[cfg(test)]
mod tests {
    use crate::release_bridge::{
        capture_release_smoke_record, export_support_bundle_record, release_snapshot_from_path,
        save_release_profile_record, ReleaseBridgeStore, ReleaseProfileSaveRequest,
        ReleaseSmokeCaptureRequest, SupportBundleExportRequest, SupportEntryRequest,
        SupportLogRequest,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn release_bridge_saves_profile_and_support_bundle_to_sqlite() {
        let path = temp_path("release-bridge");
        let mut store = default_store();
        let saved = save_release_profile_record(&mut store, signed_profile_request()).unwrap();
        let exported = export_support_bundle_record(
            &mut store,
            SupportBundleExportRequest {
                config: vec![
                    entry("OPENAI_API_KEY", "sk-test"),
                    entry("workspace", "C:/work"),
                ],
                created_at_ms: 42,
                logs: vec![
                    log("runtime", "ok"),
                    log("provider", "Authorization: Bearer abc"),
                ],
            },
        )
        .unwrap();

        assert_eq!(saved.signing.status, "signed");
        assert_eq!(exported.support_bundle.export_status, "available");
        let smoke = capture_release_smoke_record(
            &mut store,
            ReleaseSmokeCaptureRequest {
                captured_at: "2026-06-08T00:00:00.000Z".to_string(),
                command: "npm run smoke:tauri".to_string(),
                detail: "Tauri smoke passed with NSIS installer artifact.".to_string(),
                installer_path: "target/release/bundle/nsis/Delyx Next_0.0.0_x64-setup.exe"
                    .to_string(),
                status: "passed".to_string(),
            },
        )
        .unwrap();
        assert_eq!(smoke.smoke_status, "passed");
        crate::release_persistence::save_profile_to_path(&store.profile, &path).unwrap();
        crate::release_persistence::save_support_bundle_to_path(
            store.support_bundle.as_ref().unwrap(),
            &path,
        )
        .unwrap();
        crate::release_persistence::save_smoke_to_path(store.smoke.as_ref().unwrap(), &path)
            .unwrap();
        let reloaded = release_snapshot_from_path(&path).unwrap();
        assert_eq!(reloaded.signing.status, "signed");
        assert_eq!(reloaded.support_bundle.export_status, "available");
        assert_eq!(reloaded.smoke.status, "passed");
        assert_eq!(
            reloaded.smoke.installer_path.as_deref(),
            Some("target/release/bundle/nsis/Delyx Next_0.0.0_x64-setup.exe")
        );
        let loaded_bundle = crate::release_persistence::load_support_bundle_from_path(&path)
            .unwrap()
            .unwrap();
        assert_eq!(loaded_bundle.config_summary[0].value, "[redacted]");
        assert_eq!(loaded_bundle.logs[1].line, "[redacted log line]");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn release_bridge_rejects_empty_profile_and_zero_timestamp() {
        let mut store = default_store();

        assert_eq!(
            save_release_profile_record(
                &mut store,
                ReleaseProfileSaveRequest {
                    bundle_target: String::new(),
                    certificate_thumbprint: None,
                    digest_algorithm: None,
                    product_name: String::new(),
                    sign_command: None,
                    target_platform: String::new(),
                    timestamp_url: None,
                    tsp: false,
                    update_channel: String::new(),
                    update_published: false,
                    version: String::new(),
                },
            )
            .unwrap_err(),
            "Release profile requires product, version, target, bundle, and update channel."
        );
        assert_eq!(
            export_support_bundle_record(
                &mut store,
                SupportBundleExportRequest {
                    config: Vec::new(),
                    created_at_ms: 0,
                    logs: Vec::new(),
                },
            )
            .unwrap_err(),
            "Support bundle export requires a creation timestamp."
        );
        assert_eq!(
            capture_release_smoke_record(
                &mut store,
                ReleaseSmokeCaptureRequest {
                    captured_at: String::new(),
                    command: String::new(),
                    detail: String::new(),
                    installer_path: String::new(),
                    status: "passed".to_string(),
                },
            )
            .unwrap_err(),
            "Release smoke capture requires installer path, command, timestamp, and detail."
        );
        assert_eq!(
            capture_release_smoke_record(
                &mut store,
                ReleaseSmokeCaptureRequest {
                    captured_at: "2026-06-08T00:00:00.000Z".to_string(),
                    command: "npm run smoke:tauri".to_string(),
                    detail: "bad status".to_string(),
                    installer_path: "installer.exe".to_string(),
                    status: "unknown".to_string(),
                },
            )
            .unwrap_err(),
            "Release smoke status must be passed or failed."
        );
    }

    fn default_store() -> ReleaseBridgeStore {
        ReleaseBridgeStore {
            profile: crate::release::default_release_profile(),
            smoke: None,
            support_bundle: None,
            support_bundle_file_export: None,
        }
    }

    fn signed_profile_request() -> ReleaseProfileSaveRequest {
        ReleaseProfileSaveRequest {
            bundle_target: "nsis".to_string(),
            certificate_thumbprint: Some("thumbprint".to_string()),
            digest_algorithm: Some("sha256".to_string()),
            product_name: "Delyx Next".to_string(),
            sign_command: None,
            target_platform: "windows".to_string(),
            timestamp_url: None,
            tsp: false,
            update_channel: "dev-local".to_string(),
            update_published: true,
            version: "0.0.0".to_string(),
        }
    }

    fn entry(key: &str, value: &str) -> SupportEntryRequest {
        SupportEntryRequest {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    fn log(source: &str, line: &str) -> SupportLogRequest {
        SupportLogRequest {
            line: line.to_string(),
            source: source.to_string(),
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
