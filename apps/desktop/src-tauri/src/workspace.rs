use crate::workspace_git::detect_git;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub approved_roots: Vec<PathBuf>,
    pub git: GitState,
    pub rules_files: Vec<ProjectRulesFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitState {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub uncommitted_changes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRulesFile {
    pub path: PathBuf,
    pub kind: RulesFileKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesFileKind {
    Agents,
    Delyx,
    Claude,
    DelyxRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIndexEntry {
    pub path: PathBuf,
    pub relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    ProjectPathMissing,
    ProjectNotFound,
    OutsideApprovedRoot,
    Io(String),
}

#[derive(Debug, Default)]
pub struct WorkspaceManager {
    projects: Vec<Project>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_project(&mut self, path: impl AsRef<Path>) -> Result<Project, WorkspaceError> {
        let path = canonical_dir(path.as_ref())?;
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("workspace")
            .to_string();
        let project = Project {
            id: stable_project_id(&path),
            name,
            approved_roots: vec![path.clone()],
            git: detect_git(&path),
            rules_files: detect_rules_files(&path),
            path,
        };

        self.projects.retain(|existing| existing.id != project.id);
        self.projects.push(project.clone());
        Ok(project)
    }

    pub fn remove_project(&mut self, project_id: &str) -> bool {
        let before = self.projects.len();
        self.projects.retain(|project| project.id != project_id);
        before != self.projects.len()
    }

    pub fn list_projects(&self) -> &[Project] {
        &self.projects
    }

    pub fn read_file(
        &self,
        project_id: &str,
        path: impl AsRef<Path>,
    ) -> Result<String, WorkspaceError> {
        let project = self.project(project_id)?;
        let path = canonical_file(path.as_ref())?;
        ensure_inside_roots(project, &path)?;
        fs::read_to_string(path).map_err(|error| WorkspaceError::Io(error.to_string()))
    }

    pub fn index_files(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<FileIndexEntry>, WorkspaceError> {
        let project = self.project(project_id)?;
        let mut entries = Vec::new();
        collect_files(&project.path, &project.path, limit, &mut entries)?;
        Ok(entries)
    }

    pub fn search_files(
        &self,
        project_id: &str,
        query: &str,
    ) -> Result<Vec<FileIndexEntry>, WorkspaceError> {
        let query = query.to_lowercase();
        let matches = self
            .index_files(project_id, 500)?
            .into_iter()
            .filter(|entry| entry.relative_path.to_lowercase().contains(&query))
            .collect();
        Ok(matches)
    }

    fn project(&self, project_id: &str) -> Result<&Project, WorkspaceError> {
        self.projects
            .iter()
            .find(|project| project.id == project_id)
            .ok_or(WorkspaceError::ProjectNotFound)
    }
}

fn canonical_dir(path: &Path) -> Result<PathBuf, WorkspaceError> {
    let path = fs::canonicalize(path).map_err(|_| WorkspaceError::ProjectPathMissing)?;
    if path.is_dir() {
        Ok(path)
    } else {
        Err(WorkspaceError::ProjectPathMissing)
    }
}

fn canonical_file(path: &Path) -> Result<PathBuf, WorkspaceError> {
    fs::canonicalize(path).map_err(|error| WorkspaceError::Io(error.to_string()))
}

fn ensure_inside_roots(project: &Project, path: &Path) -> Result<(), WorkspaceError> {
    if project
        .approved_roots
        .iter()
        .any(|root| path.starts_with(root))
    {
        Ok(())
    } else {
        Err(WorkspaceError::OutsideApprovedRoot)
    }
}

fn collect_files(
    root: &Path,
    current: &Path,
    limit: usize,
    entries: &mut Vec<FileIndexEntry>,
) -> Result<(), WorkspaceError> {
    if entries.len() >= limit {
        return Ok(());
    }

    for entry in fs::read_dir(current).map_err(|error| WorkspaceError::Io(error.to_string()))? {
        let entry = entry.map_err(|error| WorkspaceError::Io(error.to_string()))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let file_type = entry
            .file_type()
            .map_err(|error| WorkspaceError::Io(error.to_string()))?;

        if should_skip(&name) || file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            collect_files(root, &path, limit, entries)?;
        } else if file_type.is_file() {
            let relative_path = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            entries.push(FileIndexEntry {
                path,
                relative_path,
            });
        }

        if entries.len() >= limit {
            break;
        }
    }

    Ok(())
}

fn should_skip(name: &str) -> bool {
    matches!(name, ".git" | ".tools" | "node_modules" | "target" | "dist")
}

fn detect_rules_files(root: &Path) -> Vec<ProjectRulesFile> {
    let mut files = Vec::new();
    for (name, kind) in [
        ("AGENTS.md", RulesFileKind::Agents),
        ("DELYX.md", RulesFileKind::Delyx),
        ("CLAUDE.md", RulesFileKind::Claude),
    ] {
        let path = root.join(name);
        if is_plain_file(&path) {
            files.push(ProjectRulesFile { path, kind });
        }
    }

    let rules_dir = root.join(".delyx").join("rules");
    if let Ok(entries) = fs::read_dir(rules_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_file = entry.file_type().is_ok_and(|file_type| file_type.is_file());
            if is_file && path.extension().and_then(|value| value.to_str()) == Some("md") {
                files.push(ProjectRulesFile {
                    path,
                    kind: RulesFileKind::DelyxRule,
                });
            }
        }
    }

    files
}

fn is_plain_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_file())
}

fn stable_project_id(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() {
                value.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}
