use serde::{Deserialize, Serialize};

use crate::domain::paragraph::Paragraph;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub index: usize,
    pub title: String,
    pub raw_title: Option<String>,
    pub content: String,
    pub paragraphs: Vec<Paragraph>,
    pub word_count: usize,
    pub char_count: usize,
    pub source_href: Option<String>,
    pub anchor: Option<String>,
    pub warnings: Vec<String>,
}
