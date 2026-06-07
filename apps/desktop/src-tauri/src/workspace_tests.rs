#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::workspace::{RulesFileKind, WorkspaceError, WorkspaceManager};

    #[test]
    fn adds_project_with_git_and_rules_metadata() {
        let fixture = Fixture::new("metadata");
        fixture.file(".git/HEAD", "ref: refs/heads/main\n");
        fixture.file("AGENTS.md", "# Rules\n");

        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();

        assert_eq!(project.name, fixture.root().file_name().unwrap().to_string_lossy());
        assert!(project.git.is_repo);
        assert_eq!(project.git.branch.as_deref(), Some("main"));
        assert_eq!(project.git.uncommitted_changes, None);
        assert_eq!(project.rules_files[0].kind, RulesFileKind::Agents);
        assert_eq!(manager.list_projects().len(), 1);
    }

    #[test]
    fn indexes_and_searches_files_inside_project() {
        let fixture = Fixture::new("search");
        fixture.file("src/tool_policy.rs", "policy");
        fixture.file("node_modules/ignored.js", "ignored");

        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();
        let files = manager.index_files(&project.id, 20).unwrap();
        let search = manager.search_files(&project.id, "policy").unwrap();

        assert!(files.iter().any(|entry| entry.relative_path == "src/tool_policy.rs"));
        assert!(!files.iter().any(|entry| entry.relative_path.contains("node_modules")));
        assert_eq!(search[0].relative_path, "src/tool_policy.rs");
    }

    #[test]
    fn index_skips_symlink_entries_that_can_escape_scope() {
        let fixture = Fixture::new("symlink-scope");
        let outside = Fixture::new("symlink-outside");
        outside.file("leak.txt", "secret");
        let link_path = fixture.root().join("linked-outside");
        if symlink_dir(outside.root(), &link_path).is_err() {
            return;
        }

        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();
        let files = manager.index_files(&project.id, 20).unwrap();

        assert!(!files.iter().any(|entry| entry.relative_path.contains("linked-outside")));
    }

    #[test]
    fn rules_detection_skips_symlinked_rules_that_can_escape_scope() {
        let fixture = Fixture::new("symlink-rules");
        let outside = Fixture::new("symlink-rules-outside");
        outside.file("AGENTS.md", "# Outside rules\n");
        if symlink_file(&outside.root().join("AGENTS.md"), &fixture.root().join("AGENTS.md")).is_err() {
            return;
        }

        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();

        assert!(project.rules_files.is_empty());
    }

    #[test]
    fn denies_reads_outside_approved_roots() {
        let fixture = Fixture::new("denied");
        fixture.file("allowed.txt", "yes");
        let outside = fixture.sibling("outside.txt");
        fs::write(&outside, "no").unwrap();

        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();

        assert_eq!(manager.read_file(&project.id, fixture.root().join("allowed.txt")).unwrap(), "yes");
        assert_eq!(
            manager.read_file(&project.id, outside).unwrap_err(),
            WorkspaceError::OutsideApprovedRoot
        );
    }

    #[test]
    fn removes_project() {
        let fixture = Fixture::new("remove");
        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();

        assert!(manager.remove_project(&project.id));
        assert!(manager.list_projects().is_empty());
    }

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let root = std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}"));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn root(&self) -> &PathBuf {
            &self.root
        }

        fn sibling(&self, name: &str) -> PathBuf {
            self.root.parent().unwrap().join(format!("{}-{name}", self.root.file_name().unwrap().to_string_lossy()))
        }

        fn file(&self, relative_path: &str, contents: &str) {
            let path = self.root.join(relative_path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[cfg(unix)]
    fn symlink_dir(target: &PathBuf, link: &PathBuf) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(unix)]
    fn symlink_file(target: &PathBuf, link: &PathBuf) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn symlink_dir(target: &PathBuf, link: &PathBuf) -> std::io::Result<()> {
        std::os::windows::fs::symlink_dir(target, link)
    }

    #[cfg(windows)]
    fn symlink_file(target: &PathBuf, link: &PathBuf) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }
}
