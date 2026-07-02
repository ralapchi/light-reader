use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParagraphKind {
    Title,
    Subtitle,
    Body,
    Quote,
    Separator,
}

impl ParagraphKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Subtitle => "subtitle",
            Self::Body => "body",
            Self::Quote => "quote",
            Self::Separator => "separator",
        }
    }
}
