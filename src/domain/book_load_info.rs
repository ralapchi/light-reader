use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookLoadInfo {
    pub parser_name: String,
    pub parse_warnings: Vec<String>,
    pub chapter_count: usize,
    pub loaded_at: String,
    pub source_file_size: u64,
    pub load_duration_ms: u64,
}
