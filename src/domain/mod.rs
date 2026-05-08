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

#[allow(dead_code)]
pub mod error_codes {
    pub const FILE_NOT_FOUND: &str = "FILE_NOT_FOUND";
    pub const FILE_OPEN_FAILED: &str = "FILE_OPEN_FAILED";
    pub const UNSUPPORTED_FORMAT: &str = "UNSUPPORTED_FORMAT";
    pub const EPUB_CONTAINER_MISSING: &str = "EPUB_CONTAINER_MISSING";
    pub const EPUB_OPF_MISSING: &str = "EPUB_OPF_MISSING";
    pub const EPUB_TOC_PARSE_FAILED: &str = "EPUB_TOC_PARSE_FAILED";
    pub const EPUB_METADATA_PARSE_FAILED: &str = "EPUB_METADATA_PARSE_FAILED";
    pub const TXT_DECODE_FAILED: &str = "TXT_DECODE_FAILED";
    pub const CONTENT_EMPTY: &str = "CONTENT_EMPTY";
    pub const SETTINGS_LOAD_FAILED: &str = "SETTINGS_LOAD_FAILED";
    pub const SETTINGS_SAVE_FAILED: &str = "SETTINGS_SAVE_FAILED";
    pub const PROGRESS_LOAD_FAILED: &str = "PROGRESS_LOAD_FAILED";
    pub const PROGRESS_SAVE_FAILED: &str = "PROGRESS_SAVE_FAILED";
    pub const BOOKMARK_SAVE_FAILED: &str = "BOOKMARK_SAVE_FAILED";
    pub const RECENT_SAVE_FAILED: &str = "RECENT_SAVE_FAILED";
    pub const TTS_NETWORK_ERROR: &str = "TTS_NETWORK_ERROR";
    pub const TTS_AUTH_ERROR: &str = "TTS_AUTH_ERROR";
    pub const TTS_INVALID_CONFIG: &str = "TTS_INVALID_CONFIG";
    pub const TTS_PROVIDER_ERROR: &str = "TTS_PROVIDER_ERROR";
    pub const TTS_CACHE_ERROR: &str = "TTS_CACHE_ERROR";
}
