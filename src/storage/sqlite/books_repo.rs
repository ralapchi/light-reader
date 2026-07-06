use std::str::FromStr;

use rusqlite::params;

use crate::domain::book_format::BookFormat;
use crate::domain::library_item::{FileHealth, LibraryItem, ReadingStatsSnapshot};
use crate::storage::traits::BooksRepo;

use super::connection::SqlitePool;

pub struct SqliteBooksRepo {
    pool: SqlitePool,
}

impl SqliteBooksRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn delete_book_cascade(conn: &rusqlite::Connection, book_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM reading_progress WHERE book_id = ?1", params![book_id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM bookmarks WHERE book_id = ?1", params![book_id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM book_tags WHERE book_id = ?1", params![book_id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM reading_sessions WHERE book_id = ?1", params![book_id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM books WHERE book_id = ?1", params![book_id]).map_err(|e| e.to_string())?;
    Ok(())
}

fn row_to_library_item(row: &rusqlite::Row) -> rusqlite::Result<LibraryItem> {
    let format_str: String = row.get("format")?;
    let health_str: String = row.get("file_health")?;
    let total_read_seconds: u64 = row.get::<_, u64>("total_read_seconds")?;
    let last_read_at: Option<String> = row.get("last_read_at")?;
    let bookmark_count: usize = row.get::<_, usize>("bookmark_count")?;
    let last_chapter_index: Option<usize> = row.get("last_chapter_index")?;

    Ok(LibraryItem {
        book_id: row.get("book_id")?,
        title: row.get("title")?,
        author: row.get("author")?,
        format: BookFormat::from_str(&format_str).map_err(|e| rusqlite::Error::InvalidParameterName(e))?,
        source_path: row.get("source_path")?,
        cover_cache_key: row.get("cover_ext")?,
        progress_percent: row.get("progress_percent")?,
        last_opened_at: row.get("last_opened_at")?,
        imported_at: row.get("imported_at")?,
        chapter_count: row.get::<_, usize>("chapter_count")?,
        file_health: FileHealth::from_str(&health_str)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e))?,
        stats: ReadingStatsSnapshot {
            total_read_seconds,
            last_read_at,
            bookmark_count,
            last_chapter_index,
        },
    })
}

impl BooksRepo for SqliteBooksRepo {
    fn upsert(&self, item: &LibraryItem) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let format_str = item.format.to_string();
        let health_str = item.file_health.to_string();
        conn.execute(
            "INSERT INTO books (
                book_id, title, author, format, source_path, cover_ext,
                chapter_count, file_health, total_read_seconds, last_read_at,
                bookmark_count, last_chapter_index, progress_percent,
                imported_at, last_opened_at, updated_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
            ON CONFLICT(book_id) DO UPDATE SET
                title=excluded.title, author=excluded.author, format=excluded.format,
                source_path=excluded.source_path, cover_ext=excluded.cover_ext,
                chapter_count=excluded.chapter_count, file_health=excluded.file_health,
                total_read_seconds=excluded.total_read_seconds, last_read_at=excluded.last_read_at,
                bookmark_count=excluded.bookmark_count, last_chapter_index=excluded.last_chapter_index,
                progress_percent=excluded.progress_percent, imported_at=excluded.imported_at,
                last_opened_at=excluded.last_opened_at, updated_at=excluded.updated_at",
            params![
                item.book_id,
                item.title,
                item.author,
                format_str,
                item.source_path,
                item.cover_cache_key,
                item.chapter_count,
                health_str,
                item.stats.total_read_seconds,
                item.stats.last_read_at,
                item.stats.bookmark_count,
                item.stats.last_chapter_index,
                item.progress_percent,
                item.imported_at,
                item.last_opened_at,
                now,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn delete(&self, book_id: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        delete_book_cascade(&tx, book_id)?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn delete_batch(&self, book_ids: &[&str]) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for id in book_ids {
            delete_book_cascade(&tx, id)?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn list_all(&self) -> Result<Vec<LibraryItem>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT * FROM books ORDER BY last_opened_at DESC")
            .map_err(|e| e.to_string())?;
        let items = stmt
            .query_map([], row_to_library_item)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(items)
    }

    fn get_last_selected(&self) -> Result<Option<String>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let result: String = conn
            .query_row(
                "SELECT value FROM app_meta WHERE key = 'last_selected_book_id'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    fn set_last_selected(&self, book_id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE app_meta SET value = ?1 WHERE key = 'last_selected_book_id'",
            params![book_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
