use std::path::PathBuf;

/// Service trait for accessing cached book assets (covers, images, audio).
///
/// Provides stable URLs or file paths that the frontend can consume
/// without knowing cache directory layout.
pub trait AssetService {
    /// Get the cached cover image path for a book.
    fn cover_path(&self, book_id: &str) -> Option<PathBuf>;

    /// Get the cached inline image path for a specific asset.
    fn image_path(&self, book_id: &str, asset_id: &str) -> Option<PathBuf>;
}
