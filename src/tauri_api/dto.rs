use serde::{Deserialize, Serialize};

// ── Library DTOs ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryBookCardDto {
    pub book_id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub cover_url: Option<String>,
    pub progress_percent: f32,
    pub chapter_count: usize,
    pub last_opened_at: Option<String>,
    pub imported_at: String,
    pub file_ok: bool,
}

// ── Reader DTOs ─────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReaderBookDto {
    pub book_id: String,
    pub title: String,
    pub author: Option<String>,
    pub format: String,
    pub chapter_count: usize,
    pub toc: Vec<TocItemDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TocItemDto {
    pub id: String,
    pub title: String,
    pub chapter_index: Option<usize>,
    pub href: Option<String>,
    pub depth: usize,
    pub children: Vec<TocItemDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReaderChapterDto {
    pub chapter_index: usize,
    pub title: String,
    pub blocks: Vec<ReaderBlockDto>,
    pub char_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReaderAnchorDto {
    pub chapter_id: String,
    pub block_id: String,
    pub char_offset: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveProgressDto {
    pub book_id: String,
    pub chapter_index: usize,
    pub progress_percent: f32,
    pub paragraph_index: Option<usize>,
    pub scroll_offset: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<ReaderAnchorDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchHitDto {
    pub chapter_index: usize,
    pub chapter_title: String,
    pub context: String,
    pub progress_hint: String,
    pub paragraph_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReaderTextLinkDto {
    pub start: usize,
    pub end: usize,
    pub href: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ReaderBlockDto {
    #[serde(rename = "paragraph")]
    Paragraph {
        index: usize,
        block_id: String,
        text: String,
        kind: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        links: Vec<ReaderTextLinkDto>,
    },
    #[serde(rename = "heading")]
    Heading {
        index: usize,
        block_id: String,
        text: String,
        kind: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        links: Vec<ReaderTextLinkDto>,
    },
    #[serde(rename = "quote")]
    Quote {
        index: usize,
        block_id: String,
        text: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        links: Vec<ReaderTextLinkDto>,
    },
    #[serde(rename = "image")]
    Image {
        index: usize,
        block_id: String,
        asset_id: String,
        alt_text: Option<String>,
        caption: Option<String>,
    },
    #[serde(rename = "separator")]
    Separator {
        block_id: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReaderResolvedLinkDto {
    pub chapter_index: usize,
    pub paragraph_index: Option<usize>,
    pub block_index: Option<usize>,
    pub scroll_offset: Option<f32>,
}

// ── TTS DTOs ────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsConfigDto {
    pub enabled: bool,
    pub provider: String,
    pub has_api_key: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub voice_id: Option<String>,
}

// ── Bookmark DTOs ───────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookmarkDto {
    pub id: String,
    pub book_id: String,
    pub chapter_index: usize,
    pub paragraph_index: Option<usize>,
    pub title: String,
    pub snippet: String,
    pub created_at: String,
    pub note: Option<String>,
}
