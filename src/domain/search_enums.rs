use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum SearchScope {
    #[default]
    CurrentChapter,
    EntireBook,
}
