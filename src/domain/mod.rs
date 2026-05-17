pub mod app_error;
pub mod book;
pub mod book_assets;
pub mod book_format;
pub mod book_load_info;
pub mod book_metadata;
pub mod bookmark;
pub mod chapter;
pub mod chapter_block;
pub mod chapter_builder;
pub mod core_state;
pub mod library_item;
pub mod paragraph;
pub mod paragraph_kind;
pub mod reader_settings;
pub mod reading_mode;
pub mod reading_progress;
pub mod recent_book_item;
pub mod session_state;
pub mod theme_kind;
pub mod toc_item;
pub mod tts_state;

pub use paragraph_kind::ParagraphKind;

pub mod error_codes {
    pub const FILE_OPEN_FAILED: &str = "FILE_OPEN_FAILED";
    pub const UNSUPPORTED_FORMAT: &str = "UNSUPPORTED_FORMAT";
}
