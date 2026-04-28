use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookAssets {
    pub cover_image_bytes: Option<Vec<u8>>,
    pub cover_media_type: Option<String>,
    pub has_images: bool,
    pub embedded_styles_detected: bool,
}
