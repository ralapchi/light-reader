use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParagraphKind {
    Title,
    Subtitle,
    Body,
    Quote,
    Separator,
}
