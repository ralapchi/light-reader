use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingSession {
    pub session_id: String,
    pub book_id: String,
    pub started_at: String,
    pub ended_at: String,
    pub active_seconds: u64,
    pub chapter_start: usize,
    pub chapter_end: usize,
    #[serde(default)]
    pub nav_events: u32,
    pub device_id: Option<String>,
}
