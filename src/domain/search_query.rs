use serde::{Deserialize, Serialize};

use crate::domain::search_enums::SearchScope;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub keyword: String,
    pub case_sensitive: bool,
    pub scope: SearchScope,
}
