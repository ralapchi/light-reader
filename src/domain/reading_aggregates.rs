use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingAggregates {
    pub total_active_seconds: u64,
    /// {"2026-06-18": 3600, ...}
    pub daily_seconds: HashMap<String, u64>,
    /// {"book-xxx": 7200, ...}
    pub per_book_seconds: HashMap<String, u64>,
    /// {"0": 120, "1": 0, ...} — hour of day -> seconds
    pub hourly_seconds: HashMap<String, u64>,
    /// ["2026-06-18", ...]
    pub active_dates: Vec<String>,
    pub books_completed: u32,
    pub total_nav_events: u64,
    pub computed_at: String,
}

impl Default for ReadingAggregates {
    fn default() -> Self {
        Self {
            total_active_seconds: 0,
            daily_seconds: HashMap::new(),
            per_book_seconds: HashMap::new(),
            hourly_seconds: HashMap::new(),
            active_dates: Vec::new(),
            books_completed: 0,
            total_nav_events: 0,
            computed_at: String::new(),
        }
    }
}
