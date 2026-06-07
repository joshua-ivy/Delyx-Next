fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            delyx_next_desktop::runtime_bridge::runtime_status,
            delyx_next_desktop::workspace_bridge::workspace_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running Delyx Next");
}
