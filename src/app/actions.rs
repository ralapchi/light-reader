use serde::{Deserialize, Serialize};

use crate::domain::app_error::AppError;
use crate::domain::book::Book;
use crate::domain::enums::LeftPanelTab;
use crate::domain::search_query::SearchQuery;
use crate::domain::theme_kind::ThemeKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    OpenBookSelected(String),
    OpenBookSucceeded(Book),
    OpenBookFailed(AppError),
    CloseBook,
    ToggleSidebar,
    SwitchLeftPanelTab(LeftPanelTab),
    GoToChapter(usize),
    NextChapter,
    PrevChapter,
    UpdateScrollOffset(f32),
    AddBookmarkRequested,
    RemoveBookmark(String),
    JumpToBookmark(String),
    SearchQueryChanged(SearchQuery),
    SearchSubmitted,
    SearchResultSelected(usize),
    ClearSearch,
    ToggleSearchPanel,
    ToggleSettingsPanel,
    ToggleSearchCaseSensitive,
    ThemeChanged(ThemeKind),
    /// Change a single reader setting. Tuple of (setting_key, setting_value).
    /// Keys follow the format `"font_size"`, `"line_height"`, etc.
    /// TODO(Phase-5): Replace key-value strings with typed setting action variants.
    ReaderSettingChanged(String, String),
    RestoreDefaultSettings,
    RecentBookSelected(String),
    RemoveRecentBook(String),
    ClearMissingRecentBooks,
    DismissError,
    StatusMessageTimedOut,
    CloseSearchOrSettings,
}
