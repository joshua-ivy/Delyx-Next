use crate::approval::{ApprovalEngine, ApprovalError, RiskyAction};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchProposal {
    pub id: String,
    pub run_id: String,
    pub approval_id: String,
    pub files: Vec<PatchFile>,
    pub status: PatchStatus,
    pub checkpoint_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchFile {
    pub path: PathBuf,
    pub before: String,
    pub after: String,
    pub diff: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchStatus {
    Proposed,
    Applied,
    Restored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    pub id: String,
    pub proposal_id: String,
    pub approval_id: String,
    pub files: Vec<CheckpointFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointFile {
    pub path: PathBuf,
    pub contents: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchError {
    AlreadyApplied,
    AlreadyRestored,
    Approval(ApprovalError),
    CheckpointNotFound,
    DuplicatePath,
    EmptyPatch,
    Io(String),
    OutsideApprovedRoot,
    ProposalNotFound,
}

#[derive(Debug)]
pub struct PatchEngine {
    approved_roots: Vec<PathBuf>,
    checkpoints: Vec<Checkpoint>,
    next_checkpoint_id: usize,
    next_proposal_id: usize,
    proposals: Vec<PatchProposal>,
}

impl PatchEngine {
    pub fn new(approved_roots: Vec<PathBuf>) -> Result<Self, PatchError> {
        let roots = approved_roots
            .iter()
            .map(|root| fs::canonicalize(root).map_err(io_error))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            approved_roots: roots,
            checkpoints: Vec::new(),
            next_checkpoint_id: 0,
            next_proposal_id: 0,
            proposals: Vec::new(),
        })
    }

    pub fn propose_patch(&mut self, input: PatchInput) -> Result<PatchProposal, PatchError> {
        if input.files.is_empty() {
            return Err(PatchError::EmptyPatch);
        }

        let mut files = Vec::new();
        let mut seen_paths = HashSet::new();
        for file in input.files {
            let path = self.checked_path(&file.path)?;
            if !seen_paths.insert(path.clone()) {
                return Err(PatchError::DuplicatePath);
            }
            let before = read_text(&path)?;
            let diff = build_diff(&before, &file.after);
            files.push(PatchFile {
                path,
                before,
                after: file.after,
                diff,
            });
        }

        self.next_proposal_id += 1;
        let proposal = PatchProposal {
            id: format!("patch-{}", self.next_proposal_id),
            run_id: input.run_id,
            approval_id: input.approval_id,
            files,
            status: PatchStatus::Proposed,
            checkpoint_id: None,
        };
        self.proposals.push(proposal.clone());
        Ok(proposal)
    }

    pub fn list_proposals(&self, run_id: &str) -> Vec<&PatchProposal> {
        self.proposals
            .iter()
            .filter(|proposal| proposal.run_id == run_id)
            .collect()
    }

    pub fn apply_approved_patch(
        &mut self,
        proposal_id: &str,
        now: u64,
        approvals: &ApprovalEngine,
    ) -> Result<Checkpoint, PatchError> {
        let index = self.proposal_index(proposal_id)?;
        let proposal = self.proposals[index].clone();
        if proposal.status != PatchStatus::Proposed {
            return Err(PatchError::AlreadyApplied);
        }
        approvals
            .assert_can_execute_action_for_run(
                &proposal.approval_id,
                now,
                RiskyAction::FileWrite,
                &proposal.run_id,
            )
            .map_err(PatchError::Approval)?;

        let checkpoint = self.create_checkpoint(&proposal)?;
        for file in &proposal.files {
            fs::write(&file.path, &file.after).map_err(io_error)?;
        }
        self.proposals[index].status = PatchStatus::Applied;
        self.proposals[index].checkpoint_id = Some(checkpoint.id.clone());
        self.checkpoints.push(checkpoint.clone());
        Ok(checkpoint)
    }

    pub fn restore_checkpoint(
        &mut self,
        checkpoint_id: &str,
        now: u64,
        approvals: &ApprovalEngine,
    ) -> Result<(), PatchError> {
        let checkpoint = self
            .checkpoints
            .iter()
            .find(|item| item.id == checkpoint_id)
            .cloned()
            .ok_or(PatchError::CheckpointNotFound)?;
        let index = self.proposal_index(&checkpoint.proposal_id)?;
        if self.proposals[index].status != PatchStatus::Applied {
            return Err(PatchError::AlreadyRestored);
        }
        approvals
            .assert_can_execute_action_for_run(
                &checkpoint.approval_id,
                now,
                RiskyAction::FileWrite,
                &self.proposals[index].run_id,
            )
            .map_err(PatchError::Approval)?;

        for file in &checkpoint.files {
            match &file.contents {
                Some(contents) => fs::write(&file.path, contents).map_err(io_error)?,
                None if file.path.exists() => fs::remove_file(&file.path).map_err(io_error)?,
                None => {}
            }
        }
        self.proposals[index].status = PatchStatus::Restored;
        Ok(())
    }

    fn checked_path(&self, path: &Path) -> Result<PathBuf, PatchError> {
        let normalized = normalized_path(path)?;
        self.approved_roots
            .iter()
            .any(|root| normalized.starts_with(root))
            .then_some(normalized)
            .ok_or(PatchError::OutsideApprovedRoot)
    }

    fn create_checkpoint(&mut self, proposal: &PatchProposal) -> Result<Checkpoint, PatchError> {
        self.next_checkpoint_id += 1;
        let files = proposal
            .files
            .iter()
            .map(|file| checkpoint_file(&file.path))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Checkpoint {
            id: format!("checkpoint-{}", self.next_checkpoint_id),
            proposal_id: proposal.id.clone(),
            approval_id: proposal.approval_id.clone(),
            files,
        })
    }

    fn proposal_index(&self, proposal_id: &str) -> Result<usize, PatchError> {
        self.proposals
            .iter()
            .position(|proposal| proposal.id == proposal_id)
            .ok_or(PatchError::ProposalNotFound)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchInput {
    pub run_id: String,
    pub approval_id: String,
    pub files: Vec<PatchFileInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchFileInput {
    pub path: PathBuf,
    pub after: String,
}

fn normalized_path(path: &Path) -> Result<PathBuf, PatchError> {
    if path.exists() {
        return fs::canonicalize(path).map_err(io_error);
    }
    let parent = path.parent().ok_or(PatchError::OutsideApprovedRoot)?;
    let name = path.file_name().ok_or(PatchError::OutsideApprovedRoot)?;
    Ok(fs::canonicalize(parent).map_err(io_error)?.join(name))
}

fn read_text(path: &Path) -> Result<String, PatchError> {
    if path.exists() {
        fs::read_to_string(path).map_err(io_error)
    } else {
        Ok(String::new())
    }
}

fn checkpoint_file(path: &Path) -> Result<CheckpointFile, PatchError> {
    let contents = path
        .exists()
        .then(|| fs::read_to_string(path))
        .transpose()
        .map_err(io_error)?;
    Ok(CheckpointFile {
        path: path.to_path_buf(),
        contents,
    })
}

fn build_diff(before: &str, after: &str) -> Vec<DiffLine> {
    if before == after {
        return vec![DiffLine {
            kind: DiffLineKind::Context,
            text: "No text changes.".to_string(),
        }];
    }
    before
        .lines()
        .map(|line| DiffLine {
            kind: DiffLineKind::Removed,
            text: line.to_string(),
        })
        .chain(after.lines().map(|line| DiffLine {
            kind: DiffLineKind::Added,
            text: line.to_string(),
        }))
        .collect()
}

fn io_error(error: std::io::Error) -> PatchError {
    PatchError::Io(error.to_string())
}
