use std::path::PathBuf;

use crate::domain::book::Book;
use crate::parser::epub_assets;
use crate::parser::opf_utils;
use crate::storage::paths;

/// Asset service: cover/image path resolution and caching.
///
/// Wraps `storage::paths` path computation and file existence checks.
pub struct AssetServiceImpl;

impl AssetServiceImpl {
    /// Returns `true` if the book has a cover that is not yet cached on disk.
    pub fn needs_cover_caching(book_id: &str, book: &Book) -> bool {
        if let Some(ref bytes) = book.assets.cover_image_bytes {
            let ext = epub_assets::ext_from_media_type(book.assets.cover_media_type.as_deref());
            let cache_path = paths::cover_cache_path(book_id, ext);
            return !is_cached(&cache_path, bytes.len());
        }
        false
    }

    /// Cache only the cover image. Called during book open.
    pub fn cache_cover_only(book_id: &str, book: &Book) {
        if let Some(ref bytes) = book.assets.cover_image_bytes {
            let ext = epub_assets::ext_from_media_type(book.assets.cover_media_type.as_deref());
            let cache_path = paths::cover_cache_path(book_id, ext);
            write_cache_if_changed(&cache_path, bytes);
        }
    }

    /// Cache a single chapter image. Called on-demand.
    pub fn cache_chapter_image(
        book_id: &str,
        asset_id: &str,
        cache_key: &str,
        bytes: &[u8],
    ) {
        let ext = cache_key.rsplit('.').next().unwrap_or("png");
        let cache_path = paths::image_cache_path(book_id, asset_id, ext);
        write_cache_if_changed(&cache_path, bytes);
    }

    /// Get the cached cover image path for a book.
    pub fn cover_path(book_id: &str) -> Option<PathBuf> {
        paths::find_cover_by_extensions(book_id)
    }

    /// Get the cached inline image path for a specific asset.
    pub fn image_path(book_id: &str, asset_id: &str) -> Option<PathBuf> {
        let base_dir = paths::app_data_dir().join("cache/images").join(book_id);
        find_with_extension(&base_dir, asset_id)
    }
}

/// Extract a single image from an EPUB zip file by its internal path.
pub fn extract_epub_image(epub_path: &std::path::Path, image_path: &str) -> Option<Vec<u8>> {
    let file = std::fs::File::open(epub_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    epub_assets::read_zip_entry(&mut archive, image_path)
}

/// Lightweight cover extraction from an EPUB file.
///
/// Opens the EPUB zip, finds the cover image via OPF metadata, and caches it.
/// Returns the cache path if successful.
pub fn extract_and_cache_cover(
    epub_path: &std::path::Path,
    book_id: &str,
) -> Option<std::path::PathBuf> {
    use zip::ZipArchive;

    let file = std::fs::File::open(epub_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;

    // 1. Read META-INF/container.xml to find OPF path
    let container_xml = {
        let mut f = archive.by_name("META-INF/container.xml").ok()?;
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut f, &mut buf).ok()?;
        buf
    };
    let opf_path = opf_utils::extract_opf_path(&container_xml)?;

    // 2. Read OPF file
    let opf_content = {
        let mut f = archive.by_name(&opf_path).ok()?;
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut f, &mut buf).ok()?;
        buf
    };

    // 3. Parse manifest and cover reference
    let (_manifest, cover_href) = opf_utils::parse_opf_cover(&opf_content)?;

    // 4. Resolve cover path relative to OPF base
    let opf_base = opf_path
        .rfind('/')
        .map(|i| &opf_path[..i + 1])
        .unwrap_or("");
    let cover_full_path = epub_assets::resolve_path(opf_base.trim_end_matches('/'), &cover_href);

    // 5. Extract image bytes
    let (bytes, ext) = {
        let buf = epub_assets::read_zip_entry(&mut archive, &cover_full_path)?;
        (buf, epub_assets::ext_from_href(&cover_href))
    };

    // 6. Cache to disk
    let cache_path = paths::cover_cache_path(book_id, ext);
    write_cache_if_changed(&cache_path, &bytes);
    Some(cache_path)
}

/// Check if a cached file exists with the expected size.
fn is_cached(path: &std::path::Path, expected_len: usize) -> bool {
    if path.exists() {
        if let Ok(meta) = std::fs::metadata(path) {
            return meta.len() == expected_len as u64;
        }
    }
    false
}

/// Best-effort cache write that avoids rewriting unchanged files.
fn write_cache_if_changed(path: &std::path::Path, bytes: &[u8]) {
    if is_cached(path, bytes.len()) {
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, bytes);
}

/// Check if a file exists at any of the common image extensions.
fn find_with_extension(base_dir: &std::path::Path, stem: &str) -> Option<PathBuf> {
    for ext in &["png", "jpg", "jpeg", "webp", "gif", "svg"] {
        let candidate = base_dir.join(format!("{}.{}", stem, ext));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
