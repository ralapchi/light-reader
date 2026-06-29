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
        #[cfg(feature = "db-postgres")]
        DatabaseBackendType::Postgres => {
            let conn_str = config
                .connection_string
                .as_ref()
                .ok_or("PostgreSQL requires connection_string")?;
            // TODO: implement PostgresBackend
            Err(format!("PostgreSQL backend not yet implemented: {}", conn_str))
        }
        #[cfg(feature = "db-mysql")]
        DatabaseBackendType::Mysql => {
            let conn_str = config
                .connection_string
                .as_ref()
                .ok_or("MySQL requires connection_string")?;
            // TODO: implement MysqlBackend
            Err(format!("MySQL backend not yet implemented: {}", conn_str))
        }
        #[allow(unreachable_patterns)]
        _ => Err(format!("Unsupported or disabled backend: {:?}", config.backend)),
    }
}
