use crate::thread_run_bridge::ThreadRunStore;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct ThreadRunBridgeState {
    pub(crate) store: Mutex<ThreadRunStore>,
    database_path: Option<PathBuf>,
}

impl ThreadRunBridgeState {
    pub fn persistent(database_path: PathBuf) -> Result<Self, String> {
        let store = crate::thread_run_persistence::load_from_path(&database_path)?;
        Ok(Self {
            store: Mutex::new(store),
            database_path: Some(database_path),
        })
    }

    pub(crate) fn persist(&self, store: &ThreadRunStore) -> Result<(), String> {
        if let Some(path) = &self.database_path {
            crate::thread_run_persistence::save_to_path(store, path)?;
        }
        Ok(())
    }
}
