#[cfg(test)]
mod tests {
    use crate::threads::{MessageRole, ThreadError, ThreadManager, ThreadStatus};

    const PROJECT_ID: &str = "c-users-geaux-downloads-delyx-next";

    #[test]
    fn creates_project_linked_thread_with_goal_message() {
        let mut manager = linked_manager();
        let thread = manager
            .create_thread(PROJECT_ID, "Plan a safe workspace workflow")
            .unwrap();

        assert_eq!(thread.project_id, PROJECT_ID);
        assert_eq!(thread.status, ThreadStatus::Idle);
        assert_eq!(thread.goal, "Plan a safe workspace workflow");
        assert_eq!(thread.messages[0].role, MessageRole::User);
        assert_eq!(manager.list_threads(PROJECT_ID, false).len(), 1);
    }

    #[test]
    fn rejects_threads_for_unlinked_projects_or_empty_goals() {
        let mut manager = ThreadManager::new();

        assert_eq!(
            manager
                .create_thread(PROJECT_ID, "Add thread manager")
                .unwrap_err(),
            ThreadError::ProjectNotLinked
        );

        manager.link_project(PROJECT_ID);
        assert_eq!(
            manager.create_thread(PROJECT_ID, "  ").unwrap_err(),
            ThreadError::EmptyGoal
        );
    }

    #[test]
    fn appends_conversation_messages() {
        let mut manager = linked_manager();
        let thread = manager
            .create_thread(PROJECT_ID, "Capture conversation state")
            .unwrap();

        manager
            .append_message(&thread.id, MessageRole::Assistant, "Thread state captured.")
            .unwrap();

        let thread = manager.get_thread(&thread.id).unwrap();
        assert_eq!(thread.messages.len(), 2);
        assert_eq!(thread.messages[1].body, "Thread state captured.");
    }

    #[test]
    fn supports_visible_thread_status_transitions() {
        let mut manager = linked_manager();
        let thread = manager
            .create_thread(PROJECT_ID, "Render state pills")
            .unwrap();

        manager
            .set_status(&thread.id, ThreadStatus::Exploring)
            .unwrap();
        manager
            .set_status(&thread.id, ThreadStatus::Planning)
            .unwrap();
        manager
            .set_status(&thread.id, ThreadStatus::WaitingForApproval)
            .unwrap();
        manager
            .set_status(&thread.id, ThreadStatus::Building)
            .unwrap();
        manager
            .set_status(&thread.id, ThreadStatus::Testing)
            .unwrap();
        manager
            .set_status(&thread.id, ThreadStatus::Reviewing)
            .unwrap();
        manager.set_status(&thread.id, ThreadStatus::Done).unwrap();

        assert_eq!(
            manager.get_thread(&thread.id).unwrap().status,
            ThreadStatus::Done
        );
        assert_eq!(
            manager
                .set_status(&thread.id, ThreadStatus::Building)
                .unwrap_err(),
            ThreadError::InvalidTransition
        );
    }

    #[test]
    fn active_ollama_work_can_return_to_idle() {
        let mut manager = linked_manager();
        let thread = manager.create_thread(PROJECT_ID, "Ask Ollama").unwrap();

        manager
            .set_status(&thread.id, ThreadStatus::Exploring)
            .unwrap();
        manager.set_status(&thread.id, ThreadStatus::Idle).unwrap();
        manager
            .set_status(&thread.id, ThreadStatus::Planning)
            .unwrap();
        manager.set_status(&thread.id, ThreadStatus::Idle).unwrap();

        assert_eq!(
            manager.get_thread(&thread.id).unwrap().status,
            ThreadStatus::Idle
        );
    }

    #[test]
    fn archives_threads_without_erasing_history() {
        let mut manager = linked_manager();
        let thread = manager
            .create_thread(PROJECT_ID, "Archive completed work")
            .unwrap();

        manager.archive_thread(&thread.id).unwrap();

        assert!(manager.list_threads(PROJECT_ID, false).is_empty());
        assert_eq!(manager.list_threads(PROJECT_ID, true).len(), 1);
        assert_eq!(
            manager
                .append_message(&thread.id, MessageRole::System, "Should stay immutable")
                .unwrap_err(),
            ThreadError::ArchivedThread
        );
    }

    fn linked_manager() -> ThreadManager {
        let mut manager = ThreadManager::new();
        manager.link_project(PROJECT_ID);
        manager
    }
}
