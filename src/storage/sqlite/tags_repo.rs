use rusqlite::params;

use crate::storage::traits::TagsRepo;

use super::connection::SqlitePool;

pub struct SqliteTagsRepo {
    pool: SqlitePool,
}

impl SqliteTagsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl TagsRepo for SqliteTagsRepo {
    fn get_tags(&self, book_id: &str) -> Result<Vec<String>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT tag FROM book_tags WHERE book_id = ?1 ORDER BY tag")
            .map_err(|e| e.to_string())?;
        let tags = stmt
            .query_map(params![book_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(tags)
    }

    fn set_tags(&self, book_id: &str, tags: &[String]) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM book_tags WHERE book_id = ?1", params![book_id])
            .map_err(|e| e.to_string())?;
        for tag in tags {
            tx.execute(
                "INSERT INTO book_tags (book_id, tag) VALUES (?1, ?2)",
                params![book_id, tag],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn all_tags(&self) -> Result<Vec<(String, u32)>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT tag, COUNT(*) as cnt FROM book_tags GROUP BY tag ORDER BY cnt DESC")
            .map_err(|e| e.to_string())?;
        let tags = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(tags)
    }
}
