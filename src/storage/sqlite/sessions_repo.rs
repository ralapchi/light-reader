use rusqlite::params;

use crate::domain::reading_session::ReadingSession;
use crate::storage::traits::SessionsRepo;

use super::connection::SqlitePool;

pub struct SqliteSessionsRepo {
    pool: SqlitePool,
}

impl SqliteSessionsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn row_to_session(row: &rusqlite::Row) -> rusqlite::Result<ReadingSession> {
    Ok(ReadingSession {
        session_id: row.get("session_id")?,
        book_id: row.get("book_id")?,
        started_at: row.get("started_at")?,
        ended_at: row.get("ended_at")?,
        active_seconds: row.get("active_seconds")?,
        chapter_start: row.get::<_, usize>("chapter_start")?,
        chapter_end: row.get::<_, usize>("chapter_end")?,
        nav_events: row.get("nav_events")?,
        device_id: row.get("device_id")?,
    })
}

impl SessionsRepo for SqliteSessionsRepo {
    fn save(&self, session: &ReadingSession) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO reading_sessions (
                session_id, book_id, started_at, ended_at, active_seconds,
                chapter_start, chapter_end, nav_events, device_id
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
            ON CONFLICT(session_id) DO UPDATE SET
                book_id=excluded.book_id, started_at=excluded.started_at, ended_at=excluded.ended_at,
                active_seconds=excluded.active_seconds, chapter_start=excluded.chapter_start,
                chapter_end=excluded.chapter_end, nav_events=excluded.nav_events, device_id=excluded.device_id",
            params![
                session.session_id,
                session.book_id,
                session.started_at,
                session.ended_at,
                session.active_seconds,
                session.chapter_start,
                session.chapter_end,
                session.nav_events,
                session.device_id,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn load_all(&self) -> Result<Vec<ReadingSession>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT * FROM reading_sessions ORDER BY ended_at DESC")
            .map_err(|e| e.to_string())?;
        let items = stmt
            .query_map([], row_to_session)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(items)
    }

    fn load_since(&self, date: &str) -> Result<Vec<ReadingSession>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT * FROM reading_sessions WHERE ended_at >= ?1 ORDER BY ended_at DESC")
            .map_err(|e| e.to_string())?;
        let items = stmt
            .query_map(params![date], row_to_session)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(items)
    }

    fn load_for_book(&self, book_id: &str) -> Result<Vec<ReadingSession>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT * FROM reading_sessions WHERE book_id = ?1 ORDER BY ended_at DESC")
            .map_err(|e| e.to_string())?;
        let items = stmt
            .query_map(params![book_id], row_to_session)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(items)
    }
}
