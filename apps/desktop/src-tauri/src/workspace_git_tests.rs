#[cfg(test)]
mod tests {
    use crate::workspace::WorkspaceManager;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn git_dirty_count_reads_clean_index_without_running_git() {
        let fixture = Fixture::new("git-clean");
        fixture.file(".git/HEAD", "ref: refs/heads/main\n");
        fixture.file("src/main.rs", "fn main() {}\n");
        fixture.git_index(&["src/main.rs"]);

        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();

        assert_eq!(project.git.branch.as_deref(), Some("main"));
        assert_eq!(project.git.uncommitted_changes, Some(0));
    }

    #[test]
    fn git_dirty_count_counts_modified_deleted_and_untracked_files() {
        let fixture = Fixture::new("git-dirty");
        fixture.file(".git/HEAD", "ref: refs/heads/main\n");
        fixture.file("modified.txt", "old\n");
        fixture.file("deleted.txt", "gone\n");
        fixture.git_index(&["modified.txt", "deleted.txt"]);
        fixture.file("modified.txt", "new contents\n");
        fs::remove_file(fixture.root().join("deleted.txt")).unwrap();
        fixture.file("untracked.txt", "new\n");

        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();

        assert_eq!(project.git.uncommitted_changes, Some(3));
    }

    #[test]
    fn git_dirty_count_stays_unknown_without_index() {
        let fixture = Fixture::new("git-no-index");
        fixture.file(".git/HEAD", "ref: refs/heads/main\n");

        let mut manager = WorkspaceManager::new();
        let project = manager.add_project(fixture.root()).unwrap();

        assert_eq!(project.git.uncommitted_changes, None);
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

        fn git_index(&self, tracked: &[&str]) {
            let mut data = Vec::new();
            data.extend_from_slice(b"DIRC");
            data.extend_from_slice(&2_u32.to_be_bytes());
            data.extend_from_slice(&(tracked.len() as u32).to_be_bytes());
            for path in tracked {
                write_index_entry(&mut data, self.root(), path);
            }
            self.file(".git/index", "");
            fs::write(self.root.join(".git/index"), data).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write_index_entry(data: &mut Vec<u8>, root: &Path, relative: &str) {
        let start = data.len();
        let metadata = fs::metadata(root.join(relative)).unwrap();
        let modified = metadata
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        data.extend_from_slice(&0_u32.to_be_bytes());
        data.extend_from_slice(&0_u32.to_be_bytes());
        data.extend_from_slice(&(modified.as_secs() as u32).to_be_bytes());
        data.extend_from_slice(&modified.subsec_nanos().to_be_bytes());
        for value in [0_u32, 0, 0o100644, 0, 0, metadata.len() as u32] {
            data.extend_from_slice(&value.to_be_bytes());
        }
        data.extend_from_slice(&[0_u8; 20]);
        data.extend_from_slice(&(relative.len() as u16).to_be_bytes());
        data.extend_from_slice(relative.as_bytes());
        data.push(0);
        while (data.len() - start) % 8 != 0 {
            data.push(0);
        }
    }
}
