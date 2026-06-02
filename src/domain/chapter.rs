use serde::{Deserialize, Serialize};

use crate::domain::chapter_block::ChapterBlock;
use crate::domain::paragraph::Paragraph;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub index: usize,
    pub title: String,
    pub raw_title: Option<String>,
    pub content: String,
    pub paragraphs: Vec<Paragraph>,
    /// Mixed blocks (paragraphs + images) for renderer use.
    /// Falls back to `paragraphs`-only if no images are in this chapter.
    pub blocks: Vec<ChapterBlock>,
    pub word_count: usize,
    pub char_count: usize,
    pub source_href: Option<String>,
    pub anchor: Option<String>,
    /// 段内锚点列表 (fragment → paragraph_index)
    pub anchors: Vec<(String, usize)>,
    pub warnings: Vec<String>,
}
