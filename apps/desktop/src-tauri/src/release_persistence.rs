use crate::release::{ReleaseProfile, SigningPolicy, SupportBundle, UpdateMetadata};
use rusqlite::{params, OptionalExtension};
use std::path::Path;

const PROFILE_ID: &str = "default";
const SUPPORT_BUNDLE_ID: &str = "latest";

pub fn save_profile_to_path(profile: &ReleaseProfile, path: &Path) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "INSERT INTO release_profiles
             (id, product_name, version, target_platform, bundle_target, certificate_thumbprint, digest_algorithm,
              timestamp_url, sign_command, tsp, update_channel, update_published)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
               product_name = excluded.product_name,
               version = excluded.version,
               target_platform = excluded.target_platform,
               bundle_target = excluded.bundle_target,
               certificate_thumbprint = excluded.certificate_thumbprint,
               digest_algorithm = excluded.digest_algorithm,
               timestamp_url = excluded.timestamp_url,
               sign_command = excluded.sign_command,
               tsp = excluded.tsp,
               update_channel = excluded.update_channel,
               update_published = excluded.update_published",
            params![
                PROFILE_ID,
                &profile.product_name,
                &profile.version,
                &profile.target_platform,
                &profile.bundle_target,
                profile.signing.certificate_thumbprint.as_deref(),
                profile.signing.digest_algorithm.as_deref(),
                profile.signing.timestamp_url.as_deref(),
                profile.signing.sign_command.as_deref(),
                profile.signing.tsp as i64,
                &profile.update_metadata.channel,
                profile.update_metadata.published as i64,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

pub fn load_profile_from_path(path: &Path) -> Result<Option<ReleaseProfile>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .query_row(
            "SELECT product_name, version, target_platform, bundle_target, certificate_thumbprint, digest_algorithm,
                    timestamp_url, sign_command, tsp, update_channel, update_published
             FROM release_profiles WHERE id = ?1",
            [PROFILE_ID],
            |row| {
                Ok(ReleaseProfile {
                    product_name: row.get(0)?,
                    version: row.get(1)?,
                    target_platform: row.get(2)?,
                    bundle_target: row.get(3)?,
                    signing: SigningPolicy {
                        certificate_thumbprint: row.get(4)?,
                        digest_algorithm: row.get(5)?,
                        timestamp_url: row.get(6)?,
                        sign_command: row.get(7)?,
                        tsp: row.get::<_, i64>(8)? != 0,
                    },
                    update_metadata: UpdateMetadata {
                        channel: row.get(9)?,
                        published: row.get::<_, i64>(10)? != 0,
                    },
                })
            },
        )
        .optional()
        .map_err(sql_string)
}

pub fn save_support_bundle_to_path(bundle: &SupportBundle, path: &Path) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    let config = serde_json::to_string(&bundle.config_summary).map_err(|error| error.to_string())?;
    let logs = serde_json::to_string(&bundle.logs).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT INTO support_bundles
             (id, app_name, version, created_at, config_summary_json, logs_json, secret_policy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
               app_name = excluded.app_name,
               version = excluded.version,
               created_at = excluded.created_at,
               config_summary_json = excluded.config_summary_json,
               logs_json = excluded.logs_json,
               secret_policy = excluded.secret_policy",
            params![
                SUPPORT_BUNDLE_ID,
                &bundle.app_name,
                &bundle.version,
                bundle.created_at as i64,
                config,
                logs,
                &bundle.secret_policy,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

pub fn load_support_bundle_from_path(path: &Path) -> Result<Option<SupportBundle>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .query_row(
            "SELECT app_name, version, created_at, config_summary_json, logs_json, secret_policy
             FROM support_bundles WHERE id = ?1",
            [SUPPORT_BUNDLE_ID],
            |row| {
                let config: String = row.get(3)?;
                let logs: String = row.get(4)?;
                Ok((row.get(0)?, row.get(1)?, row.get::<_, i64>(2)?, config, logs, row.get(5)?))
            },
        )
        .optional()
        .map_err(sql_string)?
        .map(|(app_name, version, created_at, config, logs, secret_policy)| {
            Ok(SupportBundle {
                app_name,
                version,
                created_at: created_at as u64,
                config_summary: serde_json::from_str(&config).map_err(|error| error.to_string())?,
                logs: serde_json::from_str(&logs).map_err(|error| error.to_string())?,
                secret_policy,
            })
        })
        .transpose()
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
