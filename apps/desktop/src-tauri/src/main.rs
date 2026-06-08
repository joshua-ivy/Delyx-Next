fn main() {
    tauri::Builder::default()
        .manage(
            delyx_next_desktop::approval_bridge::ApprovalBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("approval SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::automation_bridge::AutomationBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("automation SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::local_store_bridge::LocalStoreBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            ),
        )
        .manage(
            delyx_next_desktop::memory_bridge::MemoryBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("memory SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::patch_bridge::PatchBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("patch SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::release_bridge::ReleaseBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("release SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::review_bridge::ReviewBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("review SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::runtime_bridge::RuntimeBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            ),
        )
        .manage(
            delyx_next_desktop::skills_bridge::SkillBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("skill SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::test_runner_bridge::TestRunnerBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("test artifact SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::thread_run_bridge::ThreadRunBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("thread/run SQLite state should open"),
        )
        .manage(
            delyx_next_desktop::workspace_bridge::WorkspaceBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            ),
        )
        .manage(
            delyx_next_desktop::external_agent_run_bridge::ExternalAgentRunBridgeState::persistent(
                delyx_next_desktop::sqlite_store::default_database_path(),
            )
            .expect("external-agent run SQLite state should open"),
        )
        .invoke_handler(tauri::generate_handler![
            delyx_next_desktop::approval_bridge::approval_decide,
            delyx_next_desktop::approval_bridge::approval_propose,
            delyx_next_desktop::approval_bridge::approval_snapshot,
            delyx_next_desktop::approval_bridge_taxonomy::approval_taxonomy,
            delyx_next_desktop::automation_bridge::automation_contract_approve,
            delyx_next_desktop::automation_bridge::automation_contract_create,
            delyx_next_desktop::automation_bridge::automation_contract_pause,
            delyx_next_desktop::automation_bridge::automation_schedule_due_run,
            delyx_next_desktop::automation_bridge::automation_snapshot,
            delyx_next_desktop::external_agent_contract_bridge::external_agent_contract_preview,
            delyx_next_desktop::external_agent_run_bridge::external_agent_run_codex,
            delyx_next_desktop::external_agent_run_bridge::external_agent_run_snapshot,
            delyx_next_desktop::external_agent_status_bridge::external_agent_status,
            delyx_next_desktop::memory_bridge::memory_candidate_propose,
            delyx_next_desktop::memory_bridge::memory_candidate_suppress,
            delyx_next_desktop::memory_bridge::memory_promote_approved,
            delyx_next_desktop::memory_bridge::memory_record_suppress,
            delyx_next_desktop::memory_bridge::memory_snapshot,
            delyx_next_desktop::patch_bridge::patch_propose,
            delyx_next_desktop::patch_bridge::patch_snapshot,
            delyx_next_desktop::release_bridge::release_profile_save,
            delyx_next_desktop::release_bridge::release_snapshot,
            delyx_next_desktop::release_bridge::release_smoke_capture,
            delyx_next_desktop::release_bridge::release_support_bundle_file_export,
            delyx_next_desktop::release_bridge::release_support_bundle_export,
            delyx_next_desktop::review_bridge::review_create,
            delyx_next_desktop::review_bridge::review_snapshot,
            delyx_next_desktop::runtime_bridge::ollama_chat,
            delyx_next_desktop::runtime_bridge::runtime_status,
            delyx_next_desktop::skills_bridge::skill_activate,
            delyx_next_desktop::skills_bridge::skill_disable,
            delyx_next_desktop::skills_bridge::skill_import,
            delyx_next_desktop::skills_bridge::skill_snapshot,
            delyx_next_desktop::skills_bridge::skill_suppress,
            delyx_next_desktop::test_runner_bridge::test_run_approved,
            delyx_next_desktop::test_runner_bridge::test_snapshot,
            delyx_next_desktop::thread_run_bridge::thread_archive,
            delyx_next_desktop::thread_run_final_answer::thread_final_answer_record,
            delyx_next_desktop::thread_run_bridge::thread_message_append,
            delyx_next_desktop::thread_run_bridge::thread_run_create,
            delyx_next_desktop::thread_run_bridge::thread_run_snapshot,
            delyx_next_desktop::thread_run_bridge::thread_status_update,
            delyx_next_desktop::workspace_bridge::workspace_recent_project,
            delyx_next_desktop::workspace_bridge::workspace_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running Delyx Next");
}
