#[cfg(test)]
mod tests {
    use crate::project::{FileScopeRecord, ProjectRecord, ProjectTrustLevel};
    use crate::project_persistence::{
        delete_project_from_path, ensure_project_to_path, list_projects_from_path,
        load_project_from_path, save_project_to_path,
    };
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }

    #[test]
    fn project_with_scopes_survives_sqlite_reload() {
        let path = temp_path("project-roundtrip");
        let mut project = ProjectRecord::new("Delyx Next", "C:/code/delyx");
        project.trust_level = ProjectTrustLevel::Restricted;
        project.allowed_file_scopes.push(FileScopeRecord {
            path: "C:/code/delyx/src".to_string(),
            recursive: true,
            can_read: true,
            can_write: true,
            reason: "Source tree is writable.".to_string(),
        });

        let saved = save_project_to_path(&path, &project).unwrap();
        assert!(!saved.created_at.is_empty());
        assert!(!saved.updated_at.is_empty());

        let loaded = load_project_from_path(&path, &project.id).unwrap().unwrap();
        assert_eq!(loaded.id, project.id);
        assert_eq!(loaded.trust_level, ProjectTrustLevel::Restricted);
        assert_eq!(loaded.allowed_file_scopes.len(), 2);
        assert_eq!(loaded.allowed_file_scopes[1].path, "C:/code/delyx/src");
        assert!(loaded.allowed_file_scopes[1].can_write);
        // Defaults round-trip through the JSON columns.
        assert_eq!(loaded.approval_policy.mode, "approval-gated");
        assert!(loaded.model_permissions.allow_local);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn saving_replaces_the_scope_set_rather_than_appending() {
        let path = temp_path("project-scope-replace");
        let mut project = ProjectRecord::new("App", "/home/app");
        project.allowed_file_scopes.push(FileScopeRecord {
            path: "/home/app/docs".to_string(),
            recursive: false,
            can_read: true,
            can_write: false,
            reason: "Docs.".to_string(),
        });
        save_project_to_path(&path, &project).unwrap();

        // Re-save with a single scope; the docs scope must be gone, not duplicated.
        project.allowed_file_scopes = vec![FileScopeRecord {
            path: "/home/app".to_string(),
            recursive: true,
            can_read: true,
            can_write: false,
            reason: "Root only.".to_string(),
        }];
        save_project_to_path(&path, &project).unwrap();

        let loaded = load_project_from_path(&path, &project.id).unwrap().unwrap();
        assert_eq!(loaded.allowed_file_scopes.len(), 1);
        assert_eq!(loaded.allowed_file_scopes[0].path, "/home/app");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn list_and_delete_projects() {
        let path = temp_path("project-list-delete");
        let first = ProjectRecord::new("One", "/p/one");
        let second = ProjectRecord::new("Two", "/p/two");
        save_project_to_path(&path, &first).unwrap();
        save_project_to_path(&path, &second).unwrap();

        assert_eq!(list_projects_from_path(&path).unwrap().len(), 2);

        delete_project_from_path(&path, &first.id).unwrap();
        let remaining = list_projects_from_path(&path).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, second.id);
        // Cascade removed the deleted project's scopes too.
        assert!(load_project_from_path(&path, &first.id).unwrap().is_none());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ensure_creates_once_then_returns_existing_without_clobbering() {
        let path = temp_path("project-ensure");
        let created = ensure_project_to_path(&path, "App", "/srv/app").unwrap();

        // Customize the project, then ensure again — the customization must survive.
        let mut customized = created.clone();
        customized.trust_level = ProjectTrustLevel::External;
        customized.allowed_file_scopes.push(FileScopeRecord {
            path: "/srv/app/data".to_string(),
            recursive: true,
            can_read: true,
            can_write: false,
            reason: "Data dir.".to_string(),
        });
        save_project_to_path(&path, &customized).unwrap();

        let ensured = ensure_project_to_path(&path, "App", "/srv/app").unwrap();
        assert_eq!(ensured.id, created.id);
        assert_eq!(ensured.trust_level, ProjectTrustLevel::External);
        assert_eq!(ensured.allowed_file_scopes.len(), 2);
        assert_eq!(list_projects_from_path(&path).unwrap().len(), 1);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn invalid_project_is_rejected_before_persisting() {
        let path = temp_path("project-invalid");
        let mut project = ProjectRecord::new("", "/p/x");
        project.allowed_file_scopes.clear();
        assert!(save_project_to_path(&path, &project).is_err());
        let _ = std::fs::remove_file(path);
    }
}
