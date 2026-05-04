use serde::{Deserialize, Serialize};

use crate::domain::app_error::AppError;
use crate::domain::book::Book;
use crate::domain::bookmark::Bookmark;
use crate::domain::reader_settings::ReaderSettings;
use crate::domain::reading_progress::ReadingProgress;
use crate::domain::recent_book_item::RecentBookItem;
use crate::domain::search_state::SearchState;
use crate::domain::ui_state::UiState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub current_book: Option<Book>,
    pub reader_settings: ReaderSettings,
    pub reading_progress: Option<ReadingProgress>,
    pub recent_books: Vec<RecentBookItem>,
    pub bookmarks: Vec<Bookmark>,
    pub search_state: SearchState,
    pub ui_state: UiState,
    pub status_message: String,
    pub status_message_set_at: Option<String>,
    pub last_error: Option<AppError>,
    pub window_size: Option<(f32, f32)>,
    pub window_pos: Option<(f32, f32)>,
    pub session_started_at: Option<String>,
    pub total_read_seconds_at_session_start: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_book: None,
            reader_settings: ReaderSettings::default(),
            reading_progress: None,
            recent_books: Vec::new(),
            bookmarks: Vec::new(),
            search_state: SearchState::default(),
            ui_state: UiState::default(),
            status_message: "就绪".to_string(),
            status_message_set_at: None,
            last_error: None,
            window_size: None,
            window_pos: None,
            session_started_at: None,
            total_read_seconds_at_session_start: 0,
        }
    }
}
