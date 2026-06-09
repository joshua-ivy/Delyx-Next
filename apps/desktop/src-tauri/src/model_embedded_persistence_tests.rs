#[cfg(test)]
mod tests {
    use crate::model_embedded_persistence::{
        delete_profile_from_path, import_profile_to_path, list_profiles_from_path,
        load_profile_from_path, mark_profile_status, set_sampling_to_path, ImportLocalModelRequest,
        ModelSamplingRequest,
    };
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn imports_lists_and_reloads_a_gguf_profile_without_storing_weights() {
        let dir = temp_dir("import");
        let model = dir.join("qwen-test.Q4_K_M.gguf");
        std::fs::write(&model, b"not real weights, persistence test only").unwrap();
        let db = dir.join("db.sqlite3");

        let profile = import_profile_to_path(
            &db,
            ImportLocalModelRequest {
                model_path: model.display().to_string(),
                display_name: Some("Qwen Test".to_string()),
                chat_template_path: None,
                tokenizer_path: None,
                context_window: Some(4096),
            },
        )
        .unwrap();

        assert_eq!(profile.runtime, "mistralrs");
        assert_eq!(profile.format, "gguf");
        assert_eq!(profile.context_window, 4096);
        assert_eq!(profile.load_status, "unloaded");

        // Reload from a fresh connection: only metadata + path persist.
        let profiles = list_profiles_from_path(&db).unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].model_path, model.display().to_string());

        mark_profile_status(&db, &profile.id, "loaded", None).unwrap();
        assert_eq!(
            load_profile_from_path(&db, &profile.id)
                .unwrap()
                .load_status,
            "loaded"
        );

        // Removing the profile must not delete the model file.
        delete_profile_from_path(&db, &profile.id).unwrap();
        assert!(list_profiles_from_path(&db).unwrap().is_empty());
        assert!(model.is_file());
    }

    #[test]
    fn rejects_non_gguf_and_missing_files() {
        let dir = temp_dir("reject");
        let db = dir.join("db.sqlite3");
        let safetensors = dir.join("model.safetensors");
        std::fs::write(&safetensors, b"x").unwrap();

        assert!(
            import_profile_to_path(&db, request(safetensors.display().to_string()),)
                .unwrap_err()
                .contains("only supports .gguf")
        );

        assert!(
            import_profile_to_path(&db, request(dir.join("nope.gguf").display().to_string()))
                .unwrap_err()
                .contains("does not exist")
        );
    }

    #[test]
    fn sampling_params_round_trip() {
        let dir = temp_dir("sampling");
        let model = dir.join("m.gguf");
        std::fs::write(&model, b"x").unwrap();
        let db = dir.join("db.sqlite3");
        let profile = import_profile_to_path(&db, request(model.display().to_string())).unwrap();
        // Import seeds balanced defaults rather than null.
        assert_eq!(profile.temperature, Some(0.7));
        assert_eq!(profile.top_k, Some(40));

        set_sampling_to_path(
            &db,
            ModelSamplingRequest {
                id: profile.id.clone(),
                temperature: Some(0.35),
                top_p: Some(0.8),
                top_k: Some(20),
                repeat_penalty: Some(1.05),
                max_tokens: Some(512),
            },
        )
        .unwrap();

        let reloaded = load_profile_from_path(&db, &profile.id).unwrap();
        assert_eq!(reloaded.temperature, Some(0.35));
        assert_eq!(reloaded.top_p, Some(0.8));
        assert_eq!(reloaded.top_k, Some(20));
        assert_eq!(reloaded.max_tokens, Some(512));
        assert!((reloaded.repeat_penalty.unwrap() - 1.05).abs() < 1e-6);
    }

    fn request(model_path: String) -> ImportLocalModelRequest {
        ImportLocalModelRequest {
            model_path,
            display_name: None,
            chat_template_path: None,
            tokenizer_path: None,
            context_window: None,
        }
    }

    fn temp_dir(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("delyx-model-{label}-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
