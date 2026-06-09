//! Discover already-downloaded Ollama models so Delyx Local can reuse them
//! without re-downloading. Ollama stores GGUF weights as content-addressed blobs
//! plus JSON manifests; this maps manifest -> model-layer blob path.

use crate::model_embedded_persistence::{stable_profile_id, upsert_profile, LocalModelProfile};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModel {
    pub name: String,
    pub blob_path: String,
    pub size_bytes: Option<u64>,
}

pub fn default_ollama_models_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("OLLAMA_MODELS") {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    Some(home.join(".ollama").join("models"))
}

/// Walk `<models_dir>/manifests` and return every model whose weight-layer blob
/// exists on disk, named `<model>:<tag>`.
pub fn discover_ollama_models(models_dir: &Path) -> Vec<OllamaModel> {
    let manifests = models_dir.join("manifests");
    let mut found = Vec::new();
    collect_manifests(&manifests, models_dir, &mut found);
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

fn collect_manifests(dir: &Path, models_dir: &Path, found: &mut Vec<OllamaModel>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_manifests(&path, models_dir, found);
        } else if let Some(model) = model_from_manifest(&path, models_dir) {
            found.push(model);
        }
    }
}

/// Register a discovered Ollama model as a Delyx Local profile pointing at the
/// existing blob (GGUF). Bypasses the `.gguf` extension check because Ollama
/// stores blobs without an extension; the blob's GGUF magic is read at load time.
pub fn import_ollama_profile_to_path(
    db_path: &Path,
    model: &OllamaModel,
) -> Result<LocalModelProfile, String> {
    if !Path::new(&model.blob_path).is_file() {
        return Err(format!("Ollama model blob is missing: {}", model.blob_path));
    }
    let profile = LocalModelProfile {
        id: stable_profile_id(&format!("ollama-{}", model.name)),
        display_name: format!("{} (Ollama)", model.name),
        runtime: "mistralrs".to_string(),
        format: "gguf".to_string(),
        model_path: model.blob_path.clone(),
        chat_template_path: None,
        tokenizer_path: None,
        context_window: 8192,
        supports_tools: false,
        sha256: None,
        size_bytes: model.size_bytes,
        load_status: "unloaded".to_string(),
        last_error: None,
        temperature: None,
        top_p: None,
        top_k: None,
        repeat_penalty: None,
        max_tokens: None,
    };
    let connection =
        crate::sqlite_store::open_migrated_database(db_path).map_err(|error| error.to_string())?;
    upsert_profile(&connection, &profile)?;
    Ok(profile)
}

fn model_from_manifest(manifest: &Path, models_dir: &Path) -> Option<OllamaModel> {
    let tag = manifest.file_name()?.to_str()?.to_string();
    let model_name = manifest.parent()?.file_name()?.to_str()?.to_string();
    let text = std::fs::read_to_string(manifest).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&text).ok()?;
    let layers = json.get("layers")?.as_array()?;
    let layer = layers.iter().find(|layer| {
        layer
            .get("mediaType")
            .and_then(|value| value.as_str())
            .is_some_and(|media| media.contains("image.model"))
    })?;
    let digest = layer.get("digest")?.as_str()?;
    let blob = models_dir.join("blobs").join(digest.replace(':', "-"));
    if !blob.is_file() {
        return None;
    }
    Some(OllamaModel {
        name: format!("{model_name}:{tag}"),
        blob_path: blob.display().to_string(),
        size_bytes: layer.get("size").and_then(|value| value.as_u64()),
    })
}
