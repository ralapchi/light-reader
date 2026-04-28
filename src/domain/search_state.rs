use serde::{Deserialize, Serialize};

use crate::domain::search_query::SearchQuery;
use crate::domain::search_result::SearchResult;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchState {
    pub current_query: Option<SearchQuery>,
    pub results: Vec<SearchResult>,
    pub selected_result_index: Option<usize>,
    pub is_searching: bool,
    pub last_search_at: Option<String>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            current_query: None,
            results: Vec::new(),
            selected_result_index: None,
            is_searching: false,
            last_search_at: None,
        }
    }
}
