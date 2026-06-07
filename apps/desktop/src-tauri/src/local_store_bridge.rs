use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LocalStoreBridgeState {
    database_path: PathBuf,
}

impl LocalStoreBridgeState {
    pub fn persistent(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }
}
