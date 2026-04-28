use serde::{Deserialize, Serialize};

use crate::domain::reading_mode::ReadingMode;
use crate::domain::theme_kind::ThemeKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReaderSettings {
    pub theme: ThemeKind,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub paragraph_spacing: f32,
    pub content_width: f32,
    pub side_margin: f32,
    pub show_toc: bool,
    pub toc_width: f32,
    pub reading_mode: ReadingMode,
    pub auto_save_progress: bool,
    pub show_status_bar: bool,
    pub show_chapter_progress: bool,
    pub smooth_scroll: bool,
    pub open_last_book_on_startup: bool,
    pub restore_last_position: bool,
    pub window_padding: f32,
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            theme: ThemeKind::Light,
            font_family: "sans-serif".to_string(),
            font_size: 16.0,
            line_height: 1.6,
            paragraph_spacing: 8.0,
            content_width: 720.0,
            side_margin: 32.0,
            show_toc: true,
            toc_width: 280.0,
            reading_mode: ReadingMode::ChapterScroll,
            auto_save_progress: true,
            show_status_bar: true,
            show_chapter_progress: true,
            smooth_scroll: true,
            open_last_book_on_startup: true,
            restore_last_position: true,
            window_padding: 8.0,
        }
    }
}
