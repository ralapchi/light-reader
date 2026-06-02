use serde::{Deserialize, Serialize};

/// Universal positioning primitive for the reader.
///
/// Identifies a precise location within a book using three fields:
/// - `chapter_id`: stable chapter identifier (e.g. "ch-3")
/// - `block_id`: stable block identifier within the chapter (e.g. "p-5", "img-2")
/// - `char_offset`: character offset within the block's text content
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReaderAnchor {
    pub chapter_id: String,
    pub block_id: String,
    pub char_offset: usize,
}
