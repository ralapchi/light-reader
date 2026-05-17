use serde::{Deserialize, Serialize};

use crate::domain::app_error::AppError;
use crate::domain::tts_state::{PlaybackState, TtsState};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    pub tts_state: TtsState,
    pub playback_state: PlaybackState,
    pub status_message: String,
    pub status_message_set_at: Option<String>,
    pub last_error: Option<AppError>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            tts_state: TtsState::default(),
            playback_state: PlaybackState::default(),
            status_message: "就绪".to_string(),
            status_message_set_at: None,
            last_error: None,
        }
    }
}
