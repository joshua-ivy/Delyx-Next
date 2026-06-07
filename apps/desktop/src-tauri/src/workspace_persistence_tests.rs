#[cfg(test)]
mod tests {
    use crate::workspace_bridge::workspace_snapshot_from_path;
    use crate::workspace_persistence::{load_recent_project, save_recent_project};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn recent_workspace_project_survives_sqlite_reload() {
        let db_path = temp_path("workspace-db");
        let fixture = Fixture::new("workspace-persist");
        fixture.file("AGENTS.md", "# Rules\n");
        fixture.file("src/main.rs", "fn main() {}\n");
        let snapshot = workspace_snapshot_from_path(fixture.root(), 20).unwrap();

        save_recent_project(&db_path, &snapshot).unwrap();
        let bytes = fs::read(&db_path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let loaded = load_recent_project(&db_path).unwrap().unwrap();
        assert_eq!(loaded.id, snapshot.id);
        assert_eq!(loaded.approved_roots, snapshot.approved_roots);
        assert!(loaded.rules_files.iter().any(|file| file.kind == "AGENTS.md"));
        assert!(loaded.indexed_files.iter().any(|file| file == "src/main.rs"));
        let _ = fs::remove_file(db_path);
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

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
