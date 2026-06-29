//! Tauri command functions.
//!
//! Each function is annotated with `#[tauri::command]` and delegates to
//! the corresponding service implementation. Return types use DTOs
//! defined in `dto.rs` for safe serde serialization.

mod bookmark;
mod dto_convert;
mod library;
mod reader;
mod settings;
mod tts;

use crate::tts::cache::TtsCache;
use crate::tts::config::TtsConfig;
use crate::tts::segmenter::Segment;

use std::sync::atomic::AtomicBool;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Commands sent to the dedicated audio playback thread.
pub enum PlaybackCmd {
    Play(Vec<u8>, String),
    Pause,
    Resume,
    Stop,
}

/// Book state for Reader commands (chapter browsing, search, bookmarks, images).
pub struct ReaderState {
    pub book: Option<crate::domain::book::Book>,
}

impl ReaderState {
    pub fn new() -> Self {
        Self { book: None }
    }
}

/// TTS-only session state (playback control, synthesis cache, segments).
///
/// Note: The audio player lives on a dedicated thread (rodio's OutputStream is !Send).
/// Playback control goes through `playback_tx`, and status is polled via `is_playing_flag`.
pub struct TtsSession {
    pub tts_config: TtsConfig,
    pub cache: Arc<TtsCache>,
    pub playback_state: crate::domain::tts_state::PlaybackState,
    pub segments: Vec<Segment>,
    pub stop_flag: Arc<AtomicBool>,
    pub is_playing_flag: Arc<AtomicBool>,
    pub playback_tx: Option<mpsc::Sender<PlaybackCmd>>,
}

impl TtsSession {
    pub fn new() -> Self {
        let _ = crate::storage::paths::ensure_dirs();
        let settings_file = crate::storage::settings_store::load();

        let mut tts_config = settings_file.tts_config.unwrap_or_default();
        // Migration: clear old invalid voice IDs
        if let Some(ref vid) = tts_config.voice_id {
            if vid.contains('-') && vid.len() > 30 {
                tts_config.voice_id = None;
            }
        }

        let cache = Arc::new(TtsCache::new(crate::storage::paths::tts_cache_dir()));

        Self {
            tts_config,
            cache,
            playback_state: Default::default(),
            segments: Vec::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            is_playing_flag: Arc::new(AtomicBool::new(false)),
            playback_tx: None,
        }
    }
}

/// Type aliases for Tauri state management.
pub type BookSession = Mutex<ReaderState>;
pub type TtsSessionLock = Mutex<TtsSession>;
pub type LibraryIndexState = Mutex<crate::domain::library_item::LibraryIndex>;
pub type ProgressState = Mutex<HashMap<String, crate::domain::reading_progress::ReadingProgress>>;
pub type DirtyProgressState = Mutex<HashSet<String>>;
pub type ProgressRevisionState = Mutex<HashMap<String, u64>>;

// Re-export all command functions so `use tauri_api::commands::*` still works.
pub use bookmark::*;
pub use library::*;
pub use reader::*;
pub use settings::*;
pub use tts::*;
