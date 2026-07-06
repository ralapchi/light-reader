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
            // Ensure tag exists in tags table (default group)
            tx.execute(
                "INSERT OR IGNORE INTO tags (tag, group_id) VALUES (?1, 'default')",
                params![tag],
            )
            .map_err(|e| e.to_string())?;
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

    fn get_tags_with_groups(&self, book_id: &str) -> Result<Vec<(String, Option<String>)>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT bt.tag, COALESCE(t.group_id, 'default')
                 FROM book_tags bt
                 LEFT JOIN tags t ON bt.tag = t.tag
                 WHERE bt.book_id = ?1
                 ORDER BY bt.tag",
            )
            .map_err(|e| e.to_string())?;
        let tags = stmt
            .query_map(params![book_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(tags)
    }
}
