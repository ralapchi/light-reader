use serde::{Deserialize, Serialize};

use crate::tts::types::{PlaybackStatus, TtsProviderKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TtsState {
    pub enabled: bool,
    pub provider: Option<TtsProviderKind>,
    pub is_generating: bool,
    pub current_book_id: Option<String>,
    pub current_chapter_index: Option<usize>,
    pub current_segment_index: Option<usize>,
    pub last_error: Option<String>,
    pub last_test_at: Option<String>,
}

impl Default for TtsState {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: None,
            is_generating: false,
            current_book_id: None,
            current_chapter_index: None,
            current_segment_index: None,
            last_error: None,
            last_test_at: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub current_book_id: Option<String>,
    pub current_chapter_index: Option<usize>,
    pub current_segment_index: Option<usize>,
    pub current_paragraph_indices: Vec<usize>,
    pub progress_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            status: PlaybackStatus::Idle,
            current_book_id: None,
            current_chapter_index: None,
            current_segment_index: None,
            current_paragraph_indices: Vec::new(),
            progress_ms: None,
            duration_ms: None,
        }
    }
}
