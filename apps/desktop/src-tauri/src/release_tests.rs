#[cfg(test)]
mod tests {
    use crate::release::{
        check_signing, default_release_profile, export_support_bundle, SigningPolicy, SigningStatus,
    };

    #[test]
    fn windows_dev_release_profile_targets_unsigned_nsis() {
        let profile = default_release_profile();

        assert_eq!(profile.product_name, "Delyx Next");
        assert_eq!(profile.target_platform, "windows");
        assert_eq!(profile.bundle_target, "nsis");
        assert!(!profile.update_metadata.published);
    }

    #[test]
    fn signing_check_is_clear_for_unsigned_dev_builds() {
        let check = check_signing(&SigningPolicy::default());

        assert_eq!(check.status, SigningStatus::UnsignedDev);
        assert!(check.message.contains("Unsigned dev build"));
        assert!(check.message.contains("no certificate"));
    }

    #[test]
    fn signing_check_reports_ready_only_with_required_fields() {
        let policy = SigningPolicy {
            certificate_thumbprint: Some("thumbprint".to_string()),
            digest_algorithm: Some("sha256".to_string()),
            ..SigningPolicy::default()
        };

        let check = check_signing(&policy);

        assert_eq!(check.status, SigningStatus::Ready);
    }

    #[test]
    fn incomplete_signing_policy_is_not_silent() {
        let policy = SigningPolicy {
            certificate_thumbprint: Some("thumbprint".to_string()),
            ..SigningPolicy::default()
        };

        let check = check_signing(&policy);

        assert_eq!(check.status, SigningStatus::Incomplete);
        assert!(check.message.contains("incomplete"));
    }

    #[test]
    fn support_bundle_exports_logs_and_config_without_secrets() {
        let profile = default_release_profile();
        let bundle = export_support_bundle(
            &profile,
            vec![
                ("OPENAI_API_KEY", "sk-test"),
                ("workspace", "C:/Users/geaux/Downloads/Delyx Next"),
            ],
            vec![
                ("runtime", "loaded local config"),
                ("provider", "Authorization: Bearer abc123"),
            ],
            42,
        );

        assert_eq!(bundle.config_summary[0].value, "[redacted]");
        assert_eq!(
            bundle.config_summary[1].value,
            "C:/Users/geaux/Downloads/Delyx Next"
        );
        assert_eq!(bundle.logs[0].line, "loaded local config");
        assert_eq!(bundle.logs[1].line, "[redacted log line]");
        assert!(!format!("{:?}", bundle).contains("sk-test"));
        assert!(!format!("{:?}", bundle).contains("abc123"));
    }

    #[test]
    fn support_bundle_redacts_common_token_shapes() {
        let profile = default_release_profile();
        let bundle = export_support_bundle(
            &profile,
            vec![
                ("github_token", "ghp_example"),
                ("aws_access_key_id", "AKIAIOSFODNN7EXAMPLE"),
                ("public_note", "-----BEGIN PRIVATE KEY-----"),
            ],
            vec![
                ("slack", "xoxb-secret"),
                ("pem", "-----BEGIN OPENSSH PRIVATE KEY-----"),
            ],
            43,
        );

        assert!(bundle
            .config_summary
            .iter()
            .all(|entry| entry.value == "[redacted]"));
        assert!(bundle
            .logs
            .iter()
            .all(|entry| entry.line == "[redacted log line]"));
        assert!(!format!("{:?}", bundle).contains("ghp_example"));
        assert!(!format!("{:?}", bundle).contains("xoxb-secret"));
    }
}
