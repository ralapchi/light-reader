use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub book_id: String,
    pub chapter_index: usize,
    pub paragraph_index: usize,
    pub match_start: usize,
    pub match_end: usize,
    pub chapter_title: String,
    pub snippet: String,
    pub score: f32,
}
