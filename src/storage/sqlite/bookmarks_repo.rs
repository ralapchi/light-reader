use rusqlite::params;

use crate::domain::bookmark::Bookmark;
use crate::storage::traits::BookmarksRepo;

use super::connection::SqlitePool;

pub struct SqliteBookmarksRepo {
    pool: SqlitePool,
}

impl SqliteBookmarksRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn row_to_bookmark(row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
    Ok(Bookmark {
        id: row.get("id")?,
        book_id: row.get("book_id")?,
        chapter_index: row.get::<_, usize>("chapter_index")?,
        paragraph_index: row.get("paragraph_index")?,
        title: row.get("title")?,
        snippet: row.get("snippet")?,
        created_at: row.get("created_at")?,
        note: row.get("note")?,
    })
}

impl BookmarksRepo for SqliteBookmarksRepo {
    fn list(&self, book_id: &str) -> Result<Vec<Bookmark>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT * FROM bookmarks WHERE book_id = ?1 ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let items = stmt
            .query_map(params![book_id], row_to_bookmark)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(items)
    }

    fn list_all(&self) -> Result<Vec<Bookmark>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT * FROM bookmarks ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let items = stmt
            .query_map([], row_to_bookmark)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(items)
    }

    fn add(&self, bookmark: &Bookmark) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO bookmarks (id, book_id, chapter_index, paragraph_index, title, snippet, created_at, note)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                bookmark.id,
                bookmark.book_id,
                bookmark.chapter_index,
                bookmark.paragraph_index,
                bookmark.title,
                bookmark.snippet,
                bookmark.created_at,
                bookmark.note,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn remove(&self, book_id: &str, bookmark_id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM bookmarks WHERE book_id = ?1 AND id = ?2",
            params![book_id, bookmark_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
