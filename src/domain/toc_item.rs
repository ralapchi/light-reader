use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TocItem {
    pub id: String,
    pub title: String,
    pub chapter_index: Option<usize>,
    pub href: Option<String>,
    pub depth: u8,
    pub children: Vec<TocItem>,
    pub is_generated: bool,
}
