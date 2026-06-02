use serde::{Deserialize, Serialize};

use crate::domain::paragraph::Paragraph;

/// An inline image block extracted from EPUB HTML.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InlineImageBlock {
    pub index: usize,
    pub asset_id: String,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub source_href: Option<String>,
}

/// A content block within a chapter — either a text paragraph or an image.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChapterBlock {
    Paragraph(Paragraph),
    Heading(Paragraph),
    Quote(Paragraph),
    Image(InlineImageBlock),
    Separator,
}
