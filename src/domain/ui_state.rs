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
        }
    }
}
