use std::collections::HashMap;

use rusqlite::params;

use crate::domain::reading_aggregates::ReadingAggregates;
use crate::storage::traits::AggregatesRepo;

use super::connection::SqlitePool;

pub struct SqliteAggregatesRepo {
    pool: SqlitePool,
}

impl SqliteAggregatesRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl AggregatesRepo for SqliteAggregatesRepo {
    fn load(&self) -> Result<Option<ReadingAggregates>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT * FROM stats_aggregates WHERE id = 'default'")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map([], |row| {
                let daily_json: String = row.get("daily_seconds")?;
                let per_book_json: String = row.get("per_book_seconds")?;
                let hourly_json: String = row.get("hourly_seconds")?;
                let active_json: String = row.get("active_dates")?;

                let daily_seconds: HashMap<String, u64> =
                    serde_json::from_str(&daily_json).unwrap_or_default();
                let per_book_seconds: HashMap<String, u64> =
                    serde_json::from_str(&per_book_json).unwrap_or_default();
                let hourly_seconds: HashMap<String, u64> =
                    serde_json::from_str(&hourly_json).unwrap_or_default();
                let active_dates: Vec<String> =
                    serde_json::from_str(&active_json).unwrap_or_default();

                Ok(ReadingAggregates {
                    total_active_seconds: row.get("total_active_seconds")?,
                    daily_seconds,
                    per_book_seconds,
                    hourly_seconds,
                    active_dates,
                    books_completed: row.get("books_completed")?,
                    total_nav_events: row.get("total_nav_events")?,
                    computed_at: row.get("computed_at")?,
                })
            })
            .map_err(|e| e.to_string())?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    fn save(&self, agg: &ReadingAggregates) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let daily_json = serde_json::to_string(&agg.daily_seconds).map_err(|e| e.to_string())?;
        let per_book_json =
            serde_json::to_string(&agg.per_book_seconds).map_err(|e| e.to_string())?;
        let hourly_json =
            serde_json::to_string(&agg.hourly_seconds).map_err(|e| e.to_string())?;
        let active_json = serde_json::to_string(&agg.active_dates).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO stats_aggregates (
                id, total_active_seconds, daily_seconds, per_book_seconds,
                hourly_seconds, active_dates, books_completed, total_nav_events, computed_at
            ) VALUES ('default',?1,?2,?3,?4,?5,?6,?7,?8)
            ON CONFLICT(id) DO UPDATE SET
                total_active_seconds=excluded.total_active_seconds,
                daily_seconds=excluded.daily_seconds, per_book_seconds=excluded.per_book_seconds,
                hourly_seconds=excluded.hourly_seconds, active_dates=excluded.active_dates,
                books_completed=excluded.books_completed, total_nav_events=excluded.total_nav_events,
                computed_at=excluded.computed_at",
            params![
                agg.total_active_seconds,
                daily_json,
                per_book_json,
                hourly_json,
                active_json,
                agg.books_completed,
                agg.total_nav_events,
                agg.computed_at,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
