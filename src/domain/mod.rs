pub mod app_error;
pub mod app_state;
pub mod book;
pub mod tts_state;
pub mod book_assets;
pub mod book_format;
pub mod book_load_info;
pub mod book_metadata;
pub mod bookmark;
pub mod chapter;
pub mod chapter_block;
pub mod chapter_builder;
pub mod enums;
pub mod library_item;
pub mod library_view_state;
pub mod paragraph;
pub mod paragraph_kind;
pub mod reader_settings;
pub mod reading_mode;
pub mod reading_progress;
pub mod recent_book_item;
pub mod search_enums;
pub mod search_query;
pub mod search_result;
pub mod search_state;
pub mod theme_kind;
pub mod toc_item;
pub mod ui_state;

pub use paragraph_kind::ParagraphKind;
pub use theme_kind::ThemeKind;

pub mod error_codes {
    pub const FILE_OPEN_FAILED: &str = "FILE_OPEN_FAILED";
    pub const UNSUPPORTED_FORMAT: &str = "UNSUPPORTED_FORMAT";
}
