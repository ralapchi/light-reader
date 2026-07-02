use std::path::Path;

use crate::domain::database_config::{DatabaseBackendType, DatabaseConfig};
use crate::storage::traits::DatabaseBackend;

/// Create a database backend instance based on the provided configuration.
pub fn create_backend(
    config: &DatabaseConfig,
    data_dir: &Path,
) -> Result<Box<dyn DatabaseBackend>, String> {
    match config.backend {
        DatabaseBackendType::Sqlite => {
            let db_path = config
                .path
                .as_ref()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| data_dir.join("reader.db"));
            let backend = crate::storage::sqlite::SqliteBackend::open(&db_path)?;
            Ok(Box::new(backend))
        }
    }
}
