use serde::{Deserialize, Serialize};

// ── Book Opening Events ─────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookOpeningStarted {
    pub book_id: String,
    pub title: String,
    pub author: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookOpeningProgress {
    pub book_id: String,
    pub stage: String,
    pub progress_text: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookOpeningFinished {
    pub book_id: String,
    pub chapter_count: usize,
    pub load_duration_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookOpeningFailed {
    pub book_id: Option<String>,
    pub error_code: String,
    pub error_message: String,
    pub recoverable: bool,
}

// ── TTS Events ──────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsBuffering {
    pub book_id: String,
    pub chapter_index: usize,
    pub segment_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsPlaying {
    pub book_id: String,
    pub chapter_index: usize,
    pub segment_index: usize,
    pub total_segments: usize,
    pub paragraph_indices: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsFinished {
    pub book_id: String,
    pub chapter_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsPaused {
    pub book_id: String,
    pub segment_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsStopped {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsError {
    pub book_id: Option<String>,
    pub error_message: String,
}
