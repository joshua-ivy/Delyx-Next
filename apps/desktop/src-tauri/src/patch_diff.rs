#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchFileChangeKind {
    Create,
    Modify,
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

pub(crate) fn build_diff(before: &str, after: &str) -> Vec<DiffLine> {
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
