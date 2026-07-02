use rusqlite::params;

use crate::domain::reader_anchor::ReaderAnchor;
use crate::domain::reading_progress::ReadingProgress;
use crate::storage::traits::ProgressRepo;

use super::connection::SqlitePool;

pub struct SqliteProgressRepo {
    pool: SqlitePool,
}

impl SqliteProgressRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn row_to_progress(row: &rusqlite::Row) -> rusqlite::Result<ReadingProgress> {
    let anchor_chapter_id: Option<String> = row.get("anchor_chapter_id")?;
    let anchor = anchor_chapter_id.map(|ch| ReaderAnchor {
        chapter_id: ch,
        block_id: row.get::<_, Option<String>>("anchor_block_id").unwrap_or_default().unwrap_or_default(),
        char_offset: row.get::<_, Option<usize>>("anchor_char_offset").unwrap_or_default().unwrap_or_default(),
    });

    Ok(ReadingProgress {
        book_id: row.get("book_id")?,
        chapter_index: row.get::<_, usize>("chapter_index")?,
        paragraph_index: row.get("paragraph_index")?,
        scroll_offset: row.get("scroll_offset")?,
        progress_percent: row.get("progress_percent")?,
        last_read_at: row.get("last_read_at")?,
        session_read_seconds: row.get("session_read_seconds")?,
        total_read_seconds: row.get("total_read_seconds")?,
        anchor,
    })
}

fn upsert_progress(conn: &rusqlite::Connection, book_id: &str, p: &ReadingProgress, dirty: i32, revision: i64) -> Result<(), String> {
    let (ach, abk, acf) = match &p.anchor {
        Some(a) => (Some(a.chapter_id.clone()), Some(a.block_id.clone()), Some(a.char_offset as i64)),
        None => (None, None, None),
    };
    conn.execute(
        "INSERT INTO reading_progress (
            book_id, chapter_index, paragraph_index, scroll_offset, progress_percent,
            last_read_at, anchor_chapter_id, anchor_block_id, anchor_char_offset,
            session_read_seconds, total_read_seconds, revision, dirty
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
        ON CONFLICT(book_id) DO UPDATE SET
            chapter_index=excluded.chapter_index, paragraph_index=excluded.paragraph_index,
            scroll_offset=excluded.scroll_offset, progress_percent=excluded.progress_percent,
            last_read_at=excluded.last_read_at, anchor_chapter_id=excluded.anchor_chapter_id,
            anchor_block_id=excluded.anchor_block_id, anchor_char_offset=excluded.anchor_char_offset,
            session_read_seconds=excluded.session_read_seconds, total_read_seconds=excluded.total_read_seconds,
            revision=excluded.revision, dirty=excluded.dirty",
        params![
            book_id,
            p.chapter_index,
            p.paragraph_index,
            p.scroll_offset,
            p.progress_percent,
            p.last_read_at,
            ach,
            abk,
            acf,
            p.session_read_seconds,
            p.total_read_seconds,
            revision,
            dirty,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

impl ProgressRepo for SqliteProgressRepo {
    fn load(&self, book_id: &str) -> Result<Option<ReadingProgress>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT * FROM reading_progress WHERE book_id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![book_id], row_to_progress)
            .map_err(|e| e.to_string())?;
        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    fn save_batch(&self, entries: &[(String, ReadingProgress)]) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for (book_id, progress) in entries {
            upsert_progress(&tx, book_id, progress, 0, 0)?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
}
