#[cfg(test)]
mod tests {
    use crate::project::{stable_project_id, FileScopeRecord, ProjectRecord};

    #[test]
    fn new_project_defaults_to_a_read_only_recursive_root_scope() {
        let project = ProjectRecord::new("My App", "C:/code/app");
        assert_eq!(project.allowed_file_scopes.len(), 1);
        let scope = &project.allowed_file_scopes[0];
        assert!(scope.recursive);
        assert!(scope.can_read);
        assert!(!scope.can_write);
        assert_eq!(project.approval_policy.mode, "approval-gated");
        project.validate().unwrap();
    }

    #[test]
    fn stable_id_is_path_derived_and_case_slash_insensitive() {
        assert_eq!(
            stable_project_id("C:/code/app"),
            stable_project_id("C:\\code\\app\\"),
        );
        assert_eq!(
            stable_project_id("C:/code/app"),
            stable_project_id("c:/code/app")
        );
        assert_ne!(
            stable_project_id("C:/code/app"),
            stable_project_id("C:/code/other")
        );
    }

    #[test]
    fn recursive_scope_contains_nested_paths_but_not_siblings() {
        let project = ProjectRecord::new("App", "/home/app");
        assert!(project.can_read_path("/home/app"));
        assert!(project.can_read_path("/home/app/src/main.rs"));
        assert!(!project.can_read_path("/home/other/secret.txt"));
        // Read-only root: writes are not allowed by default.
        assert!(!project.can_write_path("/home/app/src/main.rs"));
    }

    #[test]
    fn non_recursive_scope_only_covers_direct_children() {
        let mut project = ProjectRecord::new("App", "/home/app");
        project.allowed_file_scopes = vec![FileScopeRecord {
            path: "/home/app/config".to_string(),
            recursive: false,
            can_read: true,
            can_write: false,
            reason: "Config dir.".to_string(),
        }];
        assert!(project.can_read_path("/home/app/config/app.toml"));
        assert!(!project.can_read_path("/home/app/config/nested/deep.toml"));
    }

    #[test]
    fn validate_rejects_empty_scope_reason() {
        let mut project = ProjectRecord::new("App", "/home/app");
        project.allowed_file_scopes[0].reason = "  ".to_string();
        assert!(project.validate().is_err());
    }
}
