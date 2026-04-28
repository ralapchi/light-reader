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
    pub last_error: Option<AppError>,
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
            status_message: String::new(),
            last_error: None,
        }
    }
}
