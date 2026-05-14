use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::enums::{LeftPanelTab, ScreenKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiState {
    pub screen: ScreenKind,
    pub left_panel_tab: LeftPanelTab,
    pub is_loading: bool,
    pub pending_open_path: Option<PathBuf>,
    pub last_attempted_path: Option<PathBuf>,
    /// LoadingBook 状态的书籍上下文
    pub loading_book_id: Option<String>,
    pub loading_book_title: Option<String>,
    pub loading_book_author: Option<String>,
    pub loading_book_cover_key: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            screen: ScreenKind::EmptyLibrary,
            left_panel_tab: LeftPanelTab::TableOfContents,
            is_loading: false,
            pending_open_path: None,
            last_attempted_path: None,
            loading_book_id: None,
            loading_book_title: None,
            loading_book_author: None,
            loading_book_cover_key: None,
        }
    }
}
