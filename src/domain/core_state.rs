use serde::{Deserialize, Serialize};

use crate::domain::book::Book;
use crate::domain::bookmark::Bookmark;
use crate::domain::library_item::LibraryIndex;
use crate::domain::reader_settings::ReaderSettings;
use crate::domain::reading_progress::ReadingProgress;
use crate::domain::recent_book_item::RecentBookItem;
use crate::tts::config::TtsConfig;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreState {
    pub current_book: Option<Book>,
    pub reader_settings: ReaderSettings,
    pub reading_progress: Option<ReadingProgress>,
    pub recent_books: Vec<RecentBookItem>,
    pub bookmarks: Vec<Bookmark>,
    pub library_index: LibraryIndex,
    pub tts_config: TtsConfig,
}

impl Default for CoreState {
    fn default() -> Self {
        Self {
            current_book: None,
            reader_settings: ReaderSettings::default(),
            reading_progress: None,
            recent_books: Vec::new(),
            bookmarks: Vec::new(),
            library_index: LibraryIndex::default(),
            tts_config: TtsConfig::default(),
        }
    }
}
