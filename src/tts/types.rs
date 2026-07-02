use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TtsProviderKind {
    Xiaomi,
    #[cfg(feature = "tts-aliyun")]
    Aliyun,
}

impl TtsProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Xiaomi => "xiaomi",
            #[cfg(feature = "tts-aliyun")]
            Self::Aliyun => "aliyun",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Idle,
    Buffering,
    Playing,
    Paused,
    Finished,
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TtsRequest {
    pub book_id: String,
    pub chapter_index: usize,
    pub segment_index: usize,
    pub paragraph_indices: Vec<usize>,
    pub text: String,
    pub voice_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TtsResponse {
    pub audio_bytes: Vec<u8>,
    pub media_type: String,
    pub duration_ms: Option<u64>,
}
