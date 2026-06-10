//! Native Delyx project domain model.
//!
//! A project is the workbench's durable local trust boundary: a root path, the
//! file scopes Delyx is allowed to read/write, and the default approval/model/
//! tool/memory policies. This is product state Delyx owns directly — MCP and
//! other connectors attach to a project but never define one.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectTrustLevel {
    /// Local code the user owns; the default.
    Local,
    /// Treat reads/writes with extra caution (e.g. shared or sensitive trees).
    Restricted,
    /// Externally sourced content; nothing is trusted without approval.
    External,
}

impl ProjectTrustLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            ProjectTrustLevel::Local => "local",
            ProjectTrustLevel::Restricted => "restricted",
            ProjectTrustLevel::External => "external",
        }
    }

    pub fn from_str(value: &str) -> ProjectTrustLevel {
        match value {
            "restricted" => ProjectTrustLevel::Restricted,
            "external" => ProjectTrustLevel::External,
            _ => ProjectTrustLevel::Local,
        }
    }
}

/// A single read/write permission over a path within (or attached to) a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileScopeRecord {
    pub path: String,
    pub recursive: bool,
    pub can_read: bool,
    pub can_write: bool,
    pub reason: String,
}

/// Default thresholds that drive when an attachment/import needs an approval.
/// Concrete enforcement lands in later PRs; this is the persisted policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalPolicyRecord {
    /// "approval-gated" (default) or "trusted".
    pub mode: String,
    /// Files larger than this need approval before reading.
    pub large_file_bytes: u64,
    /// Folder imports with more files than this need approval.
    pub folder_file_count: u32,
    /// Reading paths outside an allowed read scope always needs approval.
    pub require_approval_outside_scope: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPermissionsRecord {
    pub allow_local: bool,
    pub allow_cli: bool,
    pub allow_cloud: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPermissionsRecord {
    pub allow_file_write: bool,
    pub allow_terminal: bool,
    pub allow_mcp_tools: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryScopeRecord {
    /// "project" (default), "global", or "off".
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub trust_level: ProjectTrustLevel,
    pub allowed_file_scopes: Vec<FileScopeRecord>,
    pub approval_policy: ApprovalPolicyRecord,
    pub model_permissions: ModelPermissionsRecord,
    pub tool_permissions: ToolPermissionsRecord,
    pub memory_scope: MemoryScopeRecord,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for ApprovalPolicyRecord {
    fn default() -> Self {
        Self {
            mode: "approval-gated".to_string(),
            large_file_bytes: 2_000_000,
            folder_file_count: 25,
            require_approval_outside_scope: true,
        }
    }
}

impl Default for ModelPermissionsRecord {
    fn default() -> Self {
        Self {
            allow_local: true,
            allow_cli: true,
            allow_cloud: false,
        }
    }
}

impl Default for ToolPermissionsRecord {
    fn default() -> Self {
        Self {
            allow_file_write: false,
            allow_terminal: false,
            allow_mcp_tools: false,
        }
    }
}

impl Default for MemoryScopeRecord {
    fn default() -> Self {
        Self {
            mode: "project".to_string(),
        }
    }
}

impl ProjectRecord {
    /// A new project rooted at `root_path` with safe defaults: trusted-local,
    /// one read-only recursive scope over the root, approval-gated policy.
    pub fn new(name: &str, root_path: &str) -> ProjectRecord {
        let root_path = root_path.trim().to_string();
        ProjectRecord {
            id: stable_project_id(&root_path),
            name: name.trim().to_string(),
            root_path: root_path.clone(),
            trust_level: ProjectTrustLevel::Local,
            allowed_file_scopes: vec![FileScopeRecord {
                path: root_path,
                recursive: true,
                can_read: true,
                can_write: false,
                reason: "Project root (read-only by default).".to_string(),
            }],
            approval_policy: ApprovalPolicyRecord::default(),
            model_permissions: ModelPermissionsRecord::default(),
            tool_permissions: ToolPermissionsRecord::default(),
            memory_scope: MemoryScopeRecord::default(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    /// Reject obviously invalid projects before persisting.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Project name is required.".to_string());
        }
        if self.root_path.trim().is_empty() {
            return Err("Project root path is required.".to_string());
        }
        if self.allowed_file_scopes.is_empty() {
            return Err("A project needs at least one allowed file scope.".to_string());
        }
        for scope in &self.allowed_file_scopes {
            if scope.path.trim().is_empty() {
                return Err("File scope path cannot be empty.".to_string());
            }
            if scope.reason.trim().is_empty() {
                return Err(format!("File scope `{}` needs a reason.", scope.path));
            }
        }
        Ok(())
    }

    /// Whether `path` falls under an allowed *read* scope. Used by the attachment
    /// pipeline to decide if a read needs a fresh approval.
    pub fn can_read_path(&self, path: &str) -> bool {
        self.allowed_file_scopes
            .iter()
            .any(|scope| scope.can_read && scope_contains(scope, path))
    }

    /// Whether `path` falls under an allowed *write* scope.
    pub fn can_write_path(&self, path: &str) -> bool {
        self.allowed_file_scopes
            .iter()
            .any(|scope| scope.can_write && scope_contains(scope, path))
    }
}

fn scope_contains(scope: &FileScopeRecord, path: &str) -> bool {
    let scope_path = normalize(&scope.path);
    let target = normalize(path);
    if scope.recursive {
        target == scope_path || target.starts_with(&format!("{scope_path}/"))
    } else {
        // Non-recursive: the path must sit directly inside the scope directory,
        // or be the scope itself.
        target == scope_path || Path::new(&target).parent().map(normalize_path) == Some(scope_path)
    }
}

fn normalize(path: &str) -> String {
    normalize_path(Path::new(path))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string()
}

/// Deterministic id derived from the root path so re-opening the same folder
/// resolves to the same project.
pub fn stable_project_id(root_path: &str) -> String {
    let mut hash: u64 = 1469598103934665603; // FNV-1a offset basis
    for byte in normalize(root_path).to_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("project-{hash:016x}")
}
