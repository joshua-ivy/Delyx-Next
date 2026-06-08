use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseProfile {
    pub product_name: String,
    pub version: String,
    pub target_platform: String,
    pub bundle_target: String,
    pub signing: SigningPolicy,
    pub update_metadata: UpdateMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SigningPolicy {
    pub certificate_thumbprint: Option<String>,
    pub digest_algorithm: Option<String>,
    pub timestamp_url: Option<String>,
    pub sign_command: Option<String>,
    pub tsp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMetadata {
    pub channel: String,
    pub published: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningStatus {
    UnsignedDev,
    Ready,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningCheck {
    pub status: SigningStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportBundle {
    pub app_name: String,
    pub version: String,
    pub created_at: u64,
    pub config_summary: Vec<SupportConfigEntry>,
    pub logs: Vec<SupportLogEntry>,
    pub secret_policy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseSmokeStatus {
    Failed,
    Passed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSmokeRecord {
    pub status: ReleaseSmokeStatus,
    pub installer_path: String,
    pub command: String,
    pub captured_at: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SupportConfigEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SupportLogEntry {
    pub source: String,
    pub line: String,
}

pub fn default_release_profile() -> ReleaseProfile {
    ReleaseProfile {
        product_name: "Delyx Next".to_string(),
        version: "0.0.0".to_string(),
        target_platform: "windows".to_string(),
        bundle_target: "nsis".to_string(),
        signing: SigningPolicy::default(),
        update_metadata: UpdateMetadata {
            channel: "dev-local".to_string(),
            published: false,
        },
    }
}

pub fn check_signing(policy: &SigningPolicy) -> SigningCheck {
    let has_certificate = policy.certificate_thumbprint.is_some();
    let has_digest = policy.digest_algorithm.is_some();
    let has_sign_command = policy.sign_command.is_some();

    if !has_certificate
        && !has_digest
        && !has_sign_command
        && policy.timestamp_url.is_none()
        && !policy.tsp
    {
        return SigningCheck {
            status: SigningStatus::UnsignedDev,
            message:
                "Unsigned dev build: no certificate, digest, timestamp, or sign command configured."
                    .to_string(),
        };
    }
    if has_certificate && has_digest {
        return SigningCheck {
            status: SigningStatus::Ready,
            message: "Windows signing inputs are configured.".to_string(),
        };
    }
    SigningCheck {
        status: SigningStatus::Incomplete,
        message:
            "Signing configuration is incomplete; certificate and digest are required together."
                .to_string(),
    }
}

pub fn export_support_bundle(
    profile: &ReleaseProfile,
    config: Vec<(&str, &str)>,
    logs: Vec<(&str, &str)>,
    created_at: u64,
) -> SupportBundle {
    SupportBundle {
        app_name: profile.product_name.clone(),
        version: profile.version.clone(),
        created_at,
        config_summary: config.into_iter().map(config_entry).collect(),
        logs: logs.into_iter().map(log_entry).collect(),
        secret_policy: "logs/config summary redacted before export".to_string(),
    }
}

fn config_entry((key, value): (&str, &str)) -> SupportConfigEntry {
    SupportConfigEntry {
        key: key.to_string(),
        value: if secret_key(key) || secret_value(value) {
            "[redacted]".to_string()
        } else {
            value.to_string()
        },
    }
}

fn log_entry((source, line): (&str, &str)) -> SupportLogEntry {
    SupportLogEntry {
        source: source.to_string(),
        line: redact_line(line),
    }
}

fn secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    [
        "access_key",
        "api_key",
        "apikey",
        "authorization",
        "client_secret",
        "credential",
        "password",
        "private_key",
        "secret",
        "token",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn secret_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let trimmed = value.trim_start();
    lower.starts_with("sk-")
        || lower.contains("bearer ")
        || lower.contains("api_key=")
        || lower.contains("token=")
        || lower.contains("password=")
        || value.contains("AKIA")
        || value.contains("ghp_")
        || value.contains("github_pat_")
        || value.contains("xoxb-")
        || trimmed.starts_with("-----BEGIN ")
}

fn redact_line(line: &str) -> String {
    if secret_value(line) || secret_key(line) {
        return "[redacted log line]".to_string();
    }
    line.to_string()
}
