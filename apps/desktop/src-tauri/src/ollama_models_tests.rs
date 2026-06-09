#[cfg(test)]
mod tests {
    use crate::ollama_models::{discover_ollama_models, import_ollama_profile_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn discovers_a_model_and_resolves_its_blob() {
        let dir = temp_dir("discover");
        // blob
        let blob_dir = dir.join("models").join("blobs");
        fs::create_dir_all(&blob_dir).unwrap();
        let blob = blob_dir.join("sha256-abc123");
        fs::write(&blob, b"fake gguf bytes").unwrap();
        // manifest at manifests/registry.ollama.ai/library/qwen-coder/7b
        let manifest_dir = dir
            .join("models")
            .join("manifests")
            .join("registry.ollama.ai")
            .join("library")
            .join("qwen-coder");
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(
            manifest_dir.join("7b"),
            r#"{"layers":[{"mediaType":"application/vnd.ollama.image.model","digest":"sha256:abc123","size":15},{"mediaType":"application/vnd.ollama.image.params","digest":"sha256:def"}]}"#,
        )
        .unwrap();

        let models = discover_ollama_models(&dir.join("models"));
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "qwen-coder:7b");
        assert_eq!(models[0].blob_path, blob.display().to_string());
        assert_eq!(models[0].size_bytes, Some(15));

        // Import reuses the blob (no .gguf extension) as a Delyx Local profile.
        let db = dir.join("db.sqlite3");
        let profile = import_ollama_profile_to_path(&db, &models[0]).unwrap();
        assert_eq!(profile.format, "gguf");
        assert_eq!(profile.model_path, blob.display().to_string());
        assert!(profile.display_name.contains("Ollama"));
    }

    #[test]
    fn ignores_models_whose_blob_is_missing() {
        let dir = temp_dir("missing-blob");
        let manifest_dir = dir
            .join("models")
            .join("manifests")
            .join("lib")
            .join("ghost");
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(
            manifest_dir.join("latest"),
            r#"{"layers":[{"mediaType":"application/vnd.ollama.image.model","digest":"sha256:gone"}]}"#,
        )
        .unwrap();

        assert!(discover_ollama_models(&dir.join("models")).is_empty());
    }

    fn temp_dir(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("delyx-ollama-{label}-{stamp}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
