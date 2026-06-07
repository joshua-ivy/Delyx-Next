fn main() {
    tauri::Builder::default()
        .manage(delyx_next_desktop::thread_run_bridge::ThreadRunBridgeState::default())
        .invoke_handler(tauri::generate_handler![
            delyx_next_desktop::external_agent_contract_bridge::external_agent_contract_preview,
            delyx_next_desktop::external_agent_status_bridge::external_agent_status,
            delyx_next_desktop::runtime_bridge::ollama_chat,
            delyx_next_desktop::runtime_bridge::runtime_status,
            delyx_next_desktop::thread_run_bridge::thread_archive,
            delyx_next_desktop::thread_run_bridge::thread_run_create,
            delyx_next_desktop::thread_run_bridge::thread_run_snapshot,
            delyx_next_desktop::thread_run_bridge::thread_status_update,
            delyx_next_desktop::workspace_bridge::workspace_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running Delyx Next");
}
