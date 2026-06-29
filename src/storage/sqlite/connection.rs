use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type SqlitePool = Pool<SqliteConnectionManager>;

/// Create a new SQLite connection pool with WAL mode and foreign keys enabled.
pub fn create_pool(db_path: &Path) -> Result<SqlitePool, String> {
    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=10000;",
        )
    });
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)
        .map_err(|e| e.to_string())?;
    Ok(pool)
}
