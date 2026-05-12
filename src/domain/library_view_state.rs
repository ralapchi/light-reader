use serde::{Deserialize, Serialize};

use crate::domain::enums::LibraryNavSection;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LibrarySortMode {
    LastOpenedDesc,
    ImportedDesc,
    TitleAsc,
    AuthorAsc,
    ProgressDesc,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LibraryFilterMode {
    All,
    EpubOnly,
    TxtOnly,
    InProgress,
    Finished,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibraryViewState {
    pub search_query: String,
    pub sort_mode: LibrarySortMode,
    pub filter_mode: LibraryFilterMode,
    pub selected_book_id: Option<String>,
    pub selected_nav: LibraryNavSection,
    pub sidebar_search_query: String,
}

impl Default for LibraryViewState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            sort_mode: LibrarySortMode::LastOpenedDesc,
            filter_mode: LibraryFilterMode::All,
            selected_book_id: None,
            selected_nav: LibraryNavSection::Home,
            sidebar_search_query: String::new(),
        }
    }
}
