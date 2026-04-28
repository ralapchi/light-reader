use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub book_id: String,
    pub chapter_index: usize,
    pub paragraph_index: Option<usize>,
    pub scroll_offset: f32,
    pub progress_percent: f32,
    pub last_read_at: String,
    pub session_read_seconds: u64,
    pub total_read_seconds: u64,
}
