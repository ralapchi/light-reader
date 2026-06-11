use std::path::PathBuf;

use crate::domain::book::Book;
use crate::parser::epub_assets;
use crate::services::asset_service::AssetService;
use crate::storage::paths;

/// Concrete implementation of AssetService.
///
/// Wraps `storage::paths` path computation and file existence checks.
pub struct AssetServiceImpl;

impl AssetServiceImpl {
    pub fn new() -> Self {
        Self
    }

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
        &self,
        book_id: &str,
        asset_id: &str,
        cache_key: &str,
        bytes: &[u8],
    ) {
        let ext = cache_key.rsplit('.').next().unwrap_or("png");
        let cache_path = paths::image_cache_path(book_id, asset_id, ext);
        write_cache_if_changed(&cache_path, bytes);
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
    let opf_path = extract_opf_path(&container_xml)?;

    // 2. Read OPF file
    let opf_content = {
        let mut f = archive.by_name(&opf_path).ok()?;
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut f, &mut buf).ok()?;
        buf
    };

    // 3. Parse manifest and cover reference
    let (_manifest, cover_href) = parse_opf_cover(&opf_content)?;

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

/// Extract OPF path from container.xml content.
fn extract_opf_path(container_xml: &str) -> Option<String> {
    let needle = "application/oebps-package\x2bxml";
    let attr_key = "full-path=\"";
    for line in container_xml.lines() {
        if line.contains(needle) {
            if let Some(start) = line.find(attr_key) {
                let start = start + attr_key.len();
                if let Some(end) = line[start..].find('"') {
                    return Some(line[start..start + end].to_string());
                }
            }
        }
    }
    None
}

/// Parse OPF to find manifest (id -> href) and cover image href.
fn parse_opf_cover(
    opf_content: &str,
) -> Option<(std::collections::HashMap<String, String>, String)> {
    use std::collections::HashMap;

    let mut manifest: HashMap<String, String> = HashMap::new();
    let mut cover_id: Option<String> = None;
    let mut cover_href: Option<String> = None;

    // Simple line-by-line parsing
    let mut in_manifest = false;
    for line in opf_content.lines() {
        let trimmed = line.trim();

        // Detect manifest section
        if trimmed.contains("<manifest") {
            in_manifest = true;
        }
        if trimmed.contains("</manifest") {
            in_manifest = false;
        }

        // Parse manifest items
        if in_manifest && trimmed.contains("<item") {
            let id = extract_attr(trimmed, "id");
            let href = extract_attr(trimmed, "href");
            let properties = extract_attr(trimmed, "properties");

            if let (Some(id), Some(href)) = (id, href) {
                let href = epub_assets::normalize_href(&href);
                manifest.insert(id.clone(), href.clone());

                // EPUB 3: properties="cover-image"
                if properties.as_deref().unwrap_or("").contains("cover-image") {
                    cover_href = Some(href);
                }
            }
        }

        // EPUB 2: <meta name="cover" content="cover-image-id"/>
        if trimmed.contains("<meta") {
            let name = extract_attr(trimmed, "name");
            let content = extract_attr(trimmed, "content");
            if name.as_deref() == Some("cover") {
                if let Some(content) = content {
                    cover_id = Some(content);
                }
            }
        }
    }

    // Resolve EPUB 2 cover ID to href
    if cover_href.is_none() {
        if let Some(ref id) = cover_id {
            // Try exact match first, then prefix match
            cover_href = manifest.get(id).cloned();
            if cover_href.is_none() {
                // Try __id__: prefixed marker
                if let Some(actual_id) = id.strip_prefix("__id__:") {
                    cover_href = manifest.get(actual_id).cloned();
                }
            }
            // Fallback: find any image manifest item with "cover" in id
            if cover_href.is_none() {
                for (mid, mhref) in &manifest {
                    if mid.to_lowercase().contains("cover") && epub_assets::is_image_href(mhref) {
                        cover_href = Some(mhref.clone());
                        break;
                    }
                }
            }
        }
    }

    // Last fallback: find any image with "cover" in path
    if cover_href.is_none() {
        for (_, href) in &manifest {
            if href.to_lowercase().contains("cover") && epub_assets::is_image_href(href) {
                cover_href = Some(href.clone());
                break;
            }
        }
    }

    cover_href.map(|h| (manifest, h))
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    // Try quoted: attr="value"
    let pattern = format!("{}=\"", attr);
    if let Some(start) = tag.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = tag[start..].find('"') {
            return Some(tag[start..start + end].to_string());
        }
    }
    // Try single-quoted: attr='value'
    let pattern = format!("{}='", attr);
    if let Some(start) = tag.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = tag[start..].find('\'') {
            return Some(tag[start..start + end].to_string());
        }
    }
    None
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

impl AssetService for AssetServiceImpl {
    fn cover_path(&self, book_id: &str) -> Option<PathBuf> {
        paths::find_cover_by_extensions(book_id)
    }

    fn image_path(&self, book_id: &str, asset_id: &str) -> Option<PathBuf> {
        let base_dir = paths::app_data_dir().join("cache/images").join(book_id);
        find_with_extension(&base_dir, asset_id)
    }
}
