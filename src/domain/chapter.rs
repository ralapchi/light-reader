use serde::{Deserialize, Serialize};

use crate::domain::chapter_block::ChapterBlock;
use crate::domain::paragraph::Paragraph;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub index: usize,
    pub title: String,
    pub raw_title: Option<String>,
    /// Mixed blocks (paragraphs + images) for renderer use.
    pub blocks: Vec<ChapterBlock>,
    pub word_count: usize,
    pub char_count: usize,
    pub source_href: Option<String>,
    pub anchor: Option<String>,
    /// 段内锚点列表 (fragment → paragraph_index)
    pub anchors: Vec<(String, usize)>,
    pub warnings: Vec<String>,
}

impl Chapter {
    /// Returns an iterator over all text-type Paragraph references in blocks.
    pub fn text_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.blocks.iter().filter_map(|b| match b {
            ChapterBlock::Paragraph(p)
            | ChapterBlock::Heading(p)
            | ChapterBlock::Quote(p) => Some(p),
            _ => None,
        })
    }
}
