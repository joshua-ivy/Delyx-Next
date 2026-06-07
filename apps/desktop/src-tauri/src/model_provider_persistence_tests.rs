#[cfg(test)]
mod tests {
    use crate::model_provider::{ModelRegistry, ModelRole};
    use crate::model_provider_persistence::{load_routes_from_path, save_routes_to_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn model_role_routes_survive_sqlite_reload() {
        let path = temp_path("model-routes");
        let mut registry = ModelRegistry::with_default_local(10);
        registry.save_role_route(ModelRole::Answer, "mock-local", "delyx-mock-reasoner").unwrap();

        save_routes_to_path(&path, registry.routes()).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"SQLite format 3"));

        let routes = load_routes_from_path(&path).unwrap();
        let answer = routes.iter().find(|route| route.role == ModelRole::Answer).unwrap();
        assert_eq!(answer.provider_id, "mock-local");
        assert_eq!(answer.model_id, "delyx-mock-reasoner");
        let _ = fs::remove_file(path);
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("delyx-next-{name}-{stamp}.sqlite3"))
    }
}
