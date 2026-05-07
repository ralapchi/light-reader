use serde::{Deserialize, Serialize};

/// Index entry for a cached image asset extracted from a book.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookImageAsset {
    pub asset_id: String,
    pub source_href: String,
    pub media_type: Option<String>,
    pub cache_key: Option<String>,
    pub width_hint: Option<u32>,
    pub height_hint: Option<u32>,
    pub alt_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookAssets {
    pub cover_image_bytes: Option<Vec<u8>>,
    pub cover_media_type: Option<String>,
    pub has_images: bool,
    pub embedded_styles_detected: bool,
    pub image_assets: Vec<BookImageAsset>,
}
