use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentDiffSnapshot {
    files: Vec<FileSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAgentDiffCapture {
    pub created: usize,
    pub deleted: usize,
    pub modified: usize,
    pub receipts: Vec<String>,
    pub unchanged: usize,
    pub unreadable: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileSnapshot {
    before: FileState,
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FileState {
    Bytes(Vec<u8>),
    Missing,
    Unreadable(String),
}

pub fn snapshot_external_agent_diff(paths: &[PathBuf]) -> ExternalAgentDiffSnapshot {
    ExternalAgentDiffSnapshot {
        files: paths
            .iter()
            .map(|path| FileSnapshot {
                before: read_state(path),
                path: path.clone(),
            })
            .collect(),
    }
}

pub fn capture_external_agent_diff(
    snapshot: &ExternalAgentDiffSnapshot,
) -> ExternalAgentDiffCapture {
    snapshot
        .files
        .iter()
        .fold(ExternalAgentDiffCapture::default(), |mut capture, file| {
            capture.record(&file.path, &file.before, read_state(&file.path));
            capture
        })
}

impl ExternalAgentDiffCapture {
    pub fn summary(&self) -> String {
        format!(
            "Captured real file diff state: {} modified, {} created, {} deleted, {} unchanged, {} unreadable file(s).",
            self.modified, self.created, self.deleted, self.unchanged, self.unreadable
        )
    }

    fn record(&mut self, path: &Path, before: &FileState, after: FileState) {
        let label = path.display();
        match (before, after) {
            (FileState::Bytes(left), FileState::Bytes(right)) if left != &right => {
                self.modified += 1;
                self.receipts.push(format!("{label}: modified"));
            }
            (FileState::Bytes(_), FileState::Missing) => {
                self.deleted += 1;
                self.receipts.push(format!("{label}: deleted"));
            }
            (FileState::Missing, FileState::Bytes(_)) => {
                self.created += 1;
                self.receipts.push(format!("{label}: created"));
            }
            (FileState::Unreadable(error), _) => {
                self.unreadable += 1;
                self.receipts
                    .push(format!("{label}: unreadable before run ({error})"));
            }
            (_, FileState::Unreadable(error)) => {
                self.unreadable += 1;
                self.receipts
                    .push(format!("{label}: unreadable after run ({error})"));
            }
            _ => {
                self.unchanged += 1;
                self.receipts.push(format!("{label}: unchanged"));
            }
        }
    }
}

impl Default for ExternalAgentDiffCapture {
    fn default() -> Self {
        Self {
            created: 0,
            deleted: 0,
            modified: 0,
            receipts: Vec::new(),
            unchanged: 0,
            unreadable: 0,
        }
    }
}

fn read_state(path: &Path) -> FileState {
    match fs::read(path) {
        Ok(bytes) => FileState::Bytes(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => FileState::Missing,
        Err(error) => FileState::Unreadable(error.to_string()),
    }
}
