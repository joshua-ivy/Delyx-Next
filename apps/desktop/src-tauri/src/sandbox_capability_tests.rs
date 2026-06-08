#[cfg(test)]
mod tests {
    use crate::sandbox_capability::sandbox_capability_from;

    #[test]
    fn reports_checkpoint_and_worktree_when_git_present() {
        let view = sandbox_capability_from("windows", true);

        assert_eq!(view.platform, "windows");
        let checkpoint = view.modes.iter().find(|m| m.id == "checkpoint").unwrap();
        assert!(checkpoint.available);
        let worktree = view.modes.iter().find(|m| m.id == "git_worktree").unwrap();
        assert!(worktree.available);
        assert!(worktree.detail.contains("worktrees"));
    }

    #[test]
    fn worktree_unavailable_without_git() {
        let view = sandbox_capability_from("linux", false);

        let worktree = view.modes.iter().find(|m| m.id == "git_worktree").unwrap();
        assert!(!worktree.available);
        assert!(worktree.detail.contains("not found"));
    }

    #[test]
    fn os_process_sandbox_is_reported_unavailable_truthfully() {
        let view = sandbox_capability_from("macos", true);

        // Delyx wires no OS process sandbox; the report must not claim one.
        let os = view
            .modes
            .iter()
            .find(|m| m.id == "os_process_sandbox")
            .unwrap();
        assert!(!os.available);
    }
}
