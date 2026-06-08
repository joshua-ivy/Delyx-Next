#[cfg(test)]
mod tests {
    use crate::workspace_bridge::{workspace_read_files_from_path, workspace_snapshot_from_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn workspace_snapshot_exposes_real_rules_and_indexed_files() {
        let fixture = Fixture::new("workspace-snapshot");
        fixture.file(".git/HEAD", "ref: refs/heads/main\n");
        fixture.file("AGENTS.md", "# Local rules\n");
        fixture.file("src/main.rs", "fn main() {}\n");
        fixture.file("node_modules/ignored.js", "ignored");

        let snapshot = workspace_snapshot_from_path(fixture.root(), 20).unwrap();

        assert!(!snapshot.path.starts_with("//?/"));
        assert_eq!(snapshot.git.branch, "main");
        assert!(snapshot
            .rules_files
            .iter()
            .any(|file| file.kind == "AGENTS.md"));
        assert!(snapshot
            .indexed_files
            .iter()
            .any(|file| file == "src/main.rs"));
        assert!(!snapshot
            .indexed_files
            .iter()
            .any(|file| file.contains("node_modules")));
    }

    #[test]
    fn workspace_snapshot_reports_missing_project_path() {
        let missing = std::env::temp_dir().join("delyx-next-missing-workspace-snapshot");

        let result = workspace_snapshot_from_path(&missing, 20);

        assert!(result.is_err());
    }

    #[test]
    fn workspace_read_files_reads_relative_files_inside_project() {
        let fixture = Fixture::new("workspace-read");
        fixture.file("src/lib.rs", "pub fn answer() -> usize { 42 }\n");

        let files =
            workspace_read_files_from_path(fixture.root(), &["src/lib.rs".to_string()], 200)
                .unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/lib.rs");
        assert_eq!(files[0].contents, "pub fn answer() -> usize { 42 }\n");
        assert!(!files[0].truncated);
    }

    #[test]
    fn workspace_read_files_truncates_large_content() {
        let fixture = Fixture::new("workspace-read-truncated");
        fixture.file("README.md", "abcdef");

        let files =
            workspace_read_files_from_path(fixture.root(), &["README.md".to_string()], 3).unwrap();

        assert_eq!(files[0].contents, "abc");
        assert!(files[0].truncated);
    }

    #[test]
    fn workspace_read_files_rejects_parent_path_escape() {
        let fixture = Fixture::new("workspace-read-escape");
        fixture.file("src/lib.rs", "safe");

        let result =
            workspace_read_files_from_path(fixture.root(), &["../outside.rs".to_string()], 200);

        assert!(result.unwrap_err().contains("inside the project"));
    }

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}"));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn root(&self) -> &PathBuf {
            &self.root
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
}
