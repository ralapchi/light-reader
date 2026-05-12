use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScreenKind {
    EmptyLibrary,
    LoadingBook,
    Reader,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LeftPanelTab {
    TableOfContents,
    Bookmarks,
    Recent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LibraryNavSection {
    Home,
    AllBooks,
    InProgress,
    Finished,
    TtsConfig,
}
