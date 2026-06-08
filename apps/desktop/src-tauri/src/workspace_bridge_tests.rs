#[cfg(test)]
mod tests {
    use crate::workspace_bridge::workspace_snapshot_from_path;
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
