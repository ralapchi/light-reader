use std::io::Read;

/// Normalize an EPUB href for zip lookups.
///
/// EPUB references can include fragments or query strings even though zip entries
/// cannot. Callers that resolve relative paths should pass the normalized href
/// through `resolve_path`.
pub fn normalize_href(href: &str) -> String {
    href.split(['?', '#'])
        .next()
        .unwrap_or(href)
        .trim()
        .to_string()
}

/// Resolve an EPUB-internal href relative to an EPUB-internal base directory.
pub fn resolve_path(base_dir: &str, href: &str) -> String {
    let href = normalize_href(href);
    if href.starts_with('/') {
        return href.trim_start_matches('/').to_string();
    }

    let mut parts: Vec<&str> = if base_dir.is_empty() {
        Vec::new()
    } else {
        base_dir.split('/').collect()
    };
    for seg in href.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(seg),
        }
    }
    parts.join("/")
}

pub fn media_type_from_href(href: &str) -> &'static str {
    let lower = normalize_href(href).to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "image/png"
    }
}

pub fn ext_from_media_type(mime: Option<&str>) -> &'static str {
    match mime {
        Some("image/jpeg") => "jpg",
        Some("image/png") => "png",
        Some("image/webp") => "webp",
        Some("image/gif") => "gif",
        Some("image/svg+xml") => "svg",
        _ => "png",
    }
}

pub fn ext_from_href(href: &str) -> &'static str {
    ext_from_media_type(Some(media_type_from_href(href)))
}

pub fn is_image_href(href: &str) -> bool {
    let lower = normalize_href(href).to_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".webp")
        || lower.ends_with(".gif")
        || lower.ends_with(".svg")
}

pub fn read_zip_entry<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    path: &str,
) -> Option<Vec<u8>> {
    let path = normalize_href(path).trim_start_matches('/').to_string();
    let mut file = archive.by_name(&path).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    if buf.is_empty() { None } else { Some(buf) }
}
