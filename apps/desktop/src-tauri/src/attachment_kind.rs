//! Attachment source/kind taxonomy plus extension-based kind inference.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentSourceKind {
    LocalFile,
    LocalFolder,
    ProjectFile,
    Clipboard,
    Url,
    Screenshot,
    Connector,
    McpResource,
}

impl AttachmentSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AttachmentSourceKind::LocalFile => "local_file",
            AttachmentSourceKind::LocalFolder => "local_folder",
            AttachmentSourceKind::ProjectFile => "project_file",
            AttachmentSourceKind::Clipboard => "clipboard",
            AttachmentSourceKind::Url => "url",
            AttachmentSourceKind::Screenshot => "screenshot",
            AttachmentSourceKind::Connector => "connector",
            AttachmentSourceKind::McpResource => "mcp_resource",
        }
    }

    pub fn from_str(value: &str) -> Option<AttachmentSourceKind> {
        Some(match value {
            "local_file" => AttachmentSourceKind::LocalFile,
            "local_folder" => AttachmentSourceKind::LocalFolder,
            "project_file" => AttachmentSourceKind::ProjectFile,
            "clipboard" => AttachmentSourceKind::Clipboard,
            "url" => AttachmentSourceKind::Url,
            "screenshot" => AttachmentSourceKind::Screenshot,
            "connector" => AttachmentSourceKind::Connector,
            "mcp_resource" => AttachmentSourceKind::McpResource,
            _ => return None,
        })
    }

    /// External sources carry data from outside the project trust boundary.
    pub fn is_external(self) -> bool {
        matches!(
            self,
            AttachmentSourceKind::Url
                | AttachmentSourceKind::Connector
                | AttachmentSourceKind::McpResource
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentKind {
    Text,
    Code,
    Markdown,
    Pdf,
    Image,
    Archive,
    Binary,
    Folder,
    Url,
    Unknown,
}

impl AttachmentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AttachmentKind::Text => "text",
            AttachmentKind::Code => "code",
            AttachmentKind::Markdown => "markdown",
            AttachmentKind::Pdf => "pdf",
            AttachmentKind::Image => "image",
            AttachmentKind::Archive => "archive",
            AttachmentKind::Binary => "binary",
            AttachmentKind::Folder => "folder",
            AttachmentKind::Url => "url",
            AttachmentKind::Unknown => "unknown",
        }
    }

    pub fn from_str(value: &str) -> AttachmentKind {
        match value {
            "text" => AttachmentKind::Text,
            "code" => AttachmentKind::Code,
            "markdown" => AttachmentKind::Markdown,
            "pdf" => AttachmentKind::Pdf,
            "image" => AttachmentKind::Image,
            "archive" => AttachmentKind::Archive,
            "binary" => AttachmentKind::Binary,
            "folder" => AttachmentKind::Folder,
            "url" => AttachmentKind::Url,
            _ => AttachmentKind::Unknown,
        }
    }
}

/// Infer a kind from a locator's file extension. Conservative: unknown stays
/// unknown rather than guessing text.
pub fn infer_kind(locator: &str) -> AttachmentKind {
    let lower = locator.to_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or("");
    match ext {
        "md" | "markdown" => AttachmentKind::Markdown,
        "txt" | "log" | "csv" | "json" | "yaml" | "yml" | "toml" => AttachmentKind::Text,
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "c" | "h" | "cpp" | "hpp"
        | "cs" | "rb" | "php" | "swift" | "kt" | "sql" | "sh" => AttachmentKind::Code,
        "pdf" => AttachmentKind::Pdf,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" => AttachmentKind::Image,
        "zip" | "tar" | "gz" | "tgz" | "rar" | "7z" => AttachmentKind::Archive,
        "exe" | "dll" | "bin" | "so" | "dylib" | "o" => AttachmentKind::Binary,
        _ => AttachmentKind::Unknown,
    }
}
