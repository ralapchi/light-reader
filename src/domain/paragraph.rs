use serde::{Deserialize, Serialize};

use crate::domain::ParagraphKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub index: usize,
    pub text: String,
    pub kind: ParagraphKind,
    pub indent_level: u8,
    pub source_line_hint: Option<usize>,
}
