use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecentBookItem {
    pub book_id: String,
    pub title: String,
    pub author: Option<String>,
    pub source_path: String,
    pub format: String,
    pub last_opened_at: String,
    pub last_progress_percent: f32,
    pub cover_cached: bool,
    pub is_missing: bool,
}
