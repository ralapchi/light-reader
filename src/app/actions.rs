use serde::{Deserialize, Serialize};

use crate::domain::app_error::AppError;
use crate::domain::book::Book;
use crate::domain::enums::LeftPanelTab;
use crate::domain::search_query::SearchQuery;
use crate::domain::theme_kind::ThemeKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReaderSettingUpdate {
    SetFontSize(f32),
    SetLineHeight(f32),
    SetParagraphSpacing(f32),
    SetContentWidth(f32),
    SetSideMargin(f32),
    SetTocWidth(f32),
    SetWindowPadding(f32),
    SetFontFamily(String),
    SetShowToc(bool),
    SetShowStatusBar(bool),
    SetShowChapterProgress(bool),
    SetAutoSaveProgress(bool),
    SetSmoothScroll(bool),
    SetOpenLastBookOnStartup(bool),
    SetRestoreLastPosition(bool),
    SetAutoPageTurn(bool),
}

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
    /// 优先使用 UpdateReaderSetting 替代
    ReaderSettingChanged(ReaderSettingUpdate),
    UpdateReaderSetting(ReaderSettingUpdate),
    RestoreDefaultSettings,
    RecentBookSelected(String),
    RemoveRecentBook(String),
    ClearMissingRecentBooks,
    DismissError,
    StatusMessageTimedOut,
    CloseSearchOrSettings,
}
