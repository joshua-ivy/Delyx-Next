//! Lifecycle commands for Delyx-managed local model profiles: import, list,
//! unload (from memory), and remove (profile only — never the model file).

use crate::model_embedded::EmbeddedRuntimeState;
use crate::model_embedded_persistence::{
    delete_profile_from_path, import_profile_to_path, list_profiles_from_path,
    load_profile_from_path, set_sampling_to_path, ImportLocalModelRequest, LocalModelProfile,
    ModelSamplingRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelLifecycleView {
    pub status: String,
    pub message: String,
    pub profile: Option<LocalModelProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelIdRequest {
    pub id: String,
}

#[tauri::command]
pub fn local_model_import(
    runtime: tauri::State<crate::runtime_bridge::RuntimeBridgeState>,
    request: ImportLocalModelRequest,
) -> Result<LocalModelLifecycleView, String> {
    let profile = import_profile_to_path(runtime.database_path(), request)?;
    Ok(LocalModelLifecycleView {
        status: "imported".to_string(),
        message: format!("Imported {}.", profile.display_name),
        profile: Some(profile),
    })
}

#[tauri::command]
pub fn local_model_list(
    runtime: tauri::State<crate::runtime_bridge::RuntimeBridgeState>,
) -> Result<Vec<LocalModelProfile>, String> {
    list_profiles_from_path(runtime.database_path())
}

#[tauri::command]
pub fn local_model_set_sampling(
    runtime: tauri::State<crate::runtime_bridge::RuntimeBridgeState>,
    request: ModelSamplingRequest,
) -> Result<LocalModelLifecycleView, String> {
    let id = request.id.clone();
    set_sampling_to_path(runtime.database_path(), request)?;
    let profile = load_profile_from_path(runtime.database_path(), &id)?;
    Ok(LocalModelLifecycleView {
        status: "sampling_updated".to_string(),
        message: format!("Updated sampling for {}.", profile.display_name),
        profile: Some(profile),
    })
}

#[tauri::command]
pub async fn local_model_unload(
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: LocalModelIdRequest,
) -> Result<LocalModelLifecycleView, String> {
    let removed = embedded.unload(&request.id).await;
    Ok(LocalModelLifecycleView {
        status: if removed { "unloaded" } else { "not_loaded" }.to_string(),
        message: if removed {
            format!("Unloaded {} from memory.", request.id)
        } else {
            format!("{} was not loaded.", request.id)
        },
        profile: None,
    })
}

#[tauri::command]
pub async fn local_model_remove_profile(
    runtime: tauri::State<'_, crate::runtime_bridge::RuntimeBridgeState>,
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: LocalModelIdRequest,
) -> Result<LocalModelLifecycleView, String> {
    embedded.unload(&request.id).await;
    delete_profile_from_path(runtime.database_path(), &request.id)?;
    Ok(LocalModelLifecycleView {
        status: "removed".to_string(),
        message: format!(
            "Removed model profile {}. The model file was not deleted.",
            request.id
        ),
        profile: None,
    })
}
