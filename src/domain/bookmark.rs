use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub book_id: String,
    pub chapter_index: usize,
    pub paragraph_index: Option<usize>,
    pub title: String,
    pub snippet: String,
    pub created_at: String,
    pub note: Option<String>,
}
