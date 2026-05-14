use serde::{Deserialize, Serialize};

/// Index entry for a cached image asset extracted from a book.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookImageAsset {
    pub asset_id: String,
    pub source_href: String,
    /// Resolved full path inside the EPUB zip (e.g. "OEBPS/images/photo.jpg").
    /// Used for on-demand image extraction from EPUB.
    #[serde(default)]
    pub asset_path: String,
    pub media_type: Option<String>,
    pub cache_key: Option<String>,
    pub width_hint: Option<u32>,
    pub height_hint: Option<u32>,
    pub alt_text: Option<String>,
    /// Raw image bytes extracted by the parser. Not serialized to JSON;
    /// consumed by the asset/cache layer to write to disk.
    #[serde(skip)]
    pub image_bytes: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookAssets {
    pub cover_image_bytes: Option<Vec<u8>>,
    pub cover_media_type: Option<String>,
    pub has_images: bool,
    pub embedded_styles_detected: bool,
    pub image_assets: Vec<BookImageAsset>,
}
