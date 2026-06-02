use serde::{Deserialize, Serialize};

use crate::domain::ParagraphKind;

/// A link span within paragraph text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextLink {
    pub start: usize,
    pub end: usize,
    pub href: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub index: usize,
    pub text: String,
    pub kind: ParagraphKind,
    pub indent_level: u8,
    pub source_line_hint: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<TextLink>,
}
