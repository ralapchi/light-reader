use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::domain::book_format::BookFormat;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FileHealth {
    Ok,
    Missing,
    Moved,
    ParseWarning,
}

impl fmt::Display for FileHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileHealth::Ok => write!(f, "Ok"),
            FileHealth::Missing => write!(f, "Missing"),
            FileHealth::Moved => write!(f, "Moved"),
            FileHealth::ParseWarning => write!(f, "ParseWarning"),
        }
    }
}

impl FromStr for FileHealth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ok" => Ok(FileHealth::Ok),
            "Missing" => Ok(FileHealth::Missing),
            "Moved" => Ok(FileHealth::Moved),
            "ParseWarning" => Ok(FileHealth::ParseWarning),
            other => Err(format!("Unknown FileHealth: {}", other)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingStatsSnapshot {
    pub total_read_seconds: u64,
    pub last_read_at: Option<String>,
    pub bookmark_count: usize,
    pub last_chapter_index: Option<usize>,
}

impl Default for ReadingStatsSnapshot {
    fn default() -> Self {
        Self {
            total_read_seconds: 0,
            last_read_at: None,
            bookmark_count: 0,
            last_chapter_index: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibraryItem {
    pub book_id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: BookFormat,
    pub source_path: String,
    pub cover_cache_key: Option<String>,
    pub progress_percent: f32,
    pub last_opened_at: Option<String>,
    pub imported_at: String,
    pub chapter_count: usize,
    pub file_health: FileHealth,
    pub stats: ReadingStatsSnapshot,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibraryIndex {
    pub version: u32,
    pub items: Vec<LibraryItem>,
    pub last_selected_book_id: Option<String>,
}

impl Default for LibraryIndex {
    fn default() -> Self {
        Self {
            version: 1,
            items: Vec::new(),
            last_selected_book_id: None,
        }
    }
}
