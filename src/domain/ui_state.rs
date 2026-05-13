use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::enums::{LeftPanelTab, ScreenKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiState {
    pub screen: ScreenKind,
    pub left_panel_tab: LeftPanelTab,
    pub show_settings_panel: bool,
    pub show_search_panel: bool,
    pub is_loading: bool,
    pub pending_open_path: Option<PathBuf>,
    pub last_attempted_path: Option<PathBuf>,
    pub focused_search_input: bool,
    pub sidebar_collapsed: bool,
    /// 悬浮目录显隐
    pub show_floating_toc: bool,
    /// 阅读页顶部工具栏可见（hover reveal）
    pub reader_toolbar_visible: bool,
    /// LoadingBook 状态的书籍上下文
    pub loading_book_id: Option<String>,
    pub loading_book_title: Option<String>,
    pub loading_book_author: Option<String>,
    pub loading_book_cover_key: Option<String>,
    /// 延迟打开：LibraryBookSelected 设置此标志，shell 下一帧才真正 dispatch OpenBookSelected
    pub loading_pending_dispatch: bool,
    /// 最短展示帧数：LoadingBook 至少显示这么多帧再执行打开
    pub loading_min_frames: u8,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            screen: ScreenKind::EmptyLibrary,
            left_panel_tab: LeftPanelTab::TableOfContents,
            show_settings_panel: false,
            show_search_panel: false,
            is_loading: false,
            pending_open_path: None,
            last_attempted_path: None,
            focused_search_input: false,
            sidebar_collapsed: false,
            show_floating_toc: false,
            reader_toolbar_visible: false, // hidden by default, hover to reveal
            loading_book_id: None,
            loading_book_title: None,
            loading_book_author: None,
            loading_book_cover_key: None,
            loading_pending_dispatch: false,
            loading_min_frames: 0,
        }
    }
}
