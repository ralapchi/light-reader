use super::dto_convert::read_file_to_data_uri;
use super::BookSession;

/// Resolve a chapter image to its on-disk cache path, extracting from EPUB on cache miss.
/// This function performs blocking I/O (zip extraction) and must be called from a blocking context.
fn resolve_chapter_image_cache_path_blocking(
    book_id: &str,
    asset_id: &str,
    epub_path: Option<&std::path::Path>,
    asset_path: Option<&str>,
    cache_key: Option<&str>,
    media_type: Option<&str>,
) -> Result<Option<std::path::PathBuf>, String> {
    use crate::services::asset_service::AssetService;
    let svc = crate::services::asset_service_impl::AssetServiceImpl::new();

    // Check disk cache first
    if let Some(p) = svc.image_path(book_id, asset_id) {
        return Ok(Some(p));
    }

    // Cache miss — extract from EPUB on-demand
    let (epub_path, asset_path) = match (epub_path, asset_path) {
        (Some(ep), Some(ap)) if !ap.is_empty() => (ep, ap),
        _ => return Ok(None),
    };

    if let Some(bytes) =
        crate::services::asset_service_impl::extract_epub_image(epub_path, asset_path)
    {
        let ext = if let Some(key) = cache_key {
            key.rsplit('.').next().unwrap_or("png").to_string()
        } else {
            match media_type {
                Some("image/jpeg") => "jpg".to_string(),
                Some("image/webp") => "webp".to_string(),
                Some("image/gif") => "gif".to_string(),
                Some("image/svg+xml") => "svg".to_string(),
                _ => "png".to_string(),
            }
        };
        let key_owned = cache_key
            .map(|k| k.to_string())
            .unwrap_or_else(|| format!("{}.{}", asset_id, ext));
        svc.cache_chapter_image(book_id, asset_id, &key_owned, &bytes);
        let cache_path = crate::storage::paths::image_cache_path(book_id, asset_id, &ext);
        return Ok(Some(cache_path));
    }

    Ok(None)
}

/// Extract image asset info from book state (short lock).
fn extract_image_asset_info(
    book_id: &str,
    asset_id: &str,
    state: &tauri::State<'_, BookSession>,
) -> Result<Option<(std::path::PathBuf, String, Option<String>, Option<String>)>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    if book.id != book_id {
        return Ok(None);
    }
    Ok(book
        .assets
        .image_assets
        .iter()
        .find(|a| a.asset_id == asset_id)
        .filter(|img| !img.asset_path.is_empty())
        .map(|img| {
            (
                book.source_path.clone(),
                img.asset_path.clone(),
                img.cache_key.clone(),
                img.media_type.clone(),
            )
        }))
}

/// Get a chapter image as a base64 data URI by its asset_id.
/// On cache miss, extracts from EPUB on-demand. Blocking I/O runs off the async runtime.
#[tauri::command]
pub async fn reader_chapter_image(
    book_id: String,
    asset_id: String,
    state: tauri::State<'_, BookSession>,
) -> Result<Option<String>, String> {
    // Fast path: disk cache hit (no state access needed)
    {
        use crate::services::asset_service::AssetService;
        let svc = crate::services::asset_service_impl::AssetServiceImpl::new();
        if let Some(p) = svc.image_path(&book_id, &asset_id) {
            return read_file_to_data_uri(p.to_str().unwrap_or(""));
        }
    }

    // Extract asset info from state (short lock)
    let asset_info = extract_image_asset_info(&book_id, &asset_id, &state)?;

    // Heavy blocking work off the async runtime
    tauri::async_runtime::spawn_blocking(move || {
        let (epub_path, asset_path, cache_key, media_type) = match asset_info {
            Some(info) => info,
            None => return Ok(None),
        };
        let path = resolve_chapter_image_cache_path_blocking(
            &book_id,
            &asset_id,
            Some(epub_path.as_path()),
            Some(&asset_path),
            cache_key.as_deref(),
            media_type.as_deref(),
        )?;
        match path {
            Some(p) => read_file_to_data_uri(p.to_str().unwrap_or("")),
            None => Ok(None),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Get a chapter image file path by its asset_id (for use with convertFileSrc).
/// On cache miss, extracts from EPUB on-demand and caches to disk.
#[tauri::command]
pub async fn reader_chapter_image_path(
    book_id: String,
    asset_id: String,
    state: tauri::State<'_, BookSession>,
) -> Result<Option<String>, String> {
    // Fast path: disk cache hit
    {
        use crate::services::asset_service::AssetService;
        let svc = crate::services::asset_service_impl::AssetServiceImpl::new();
        if let Some(p) = svc.image_path(&book_id, &asset_id) {
            return Ok(p.to_str().map(|s| s.to_string()));
        }
    }

    let asset_info = extract_image_asset_info(&book_id, &asset_id, &state)?;

    tauri::async_runtime::spawn_blocking(move || {
        let (epub_path, asset_path, cache_key, media_type) = match asset_info {
            Some(info) => info,
            None => return Ok(None),
        };
        let path = resolve_chapter_image_cache_path_blocking(
            &book_id,
            &asset_id,
            Some(epub_path.as_path()),
            Some(&asset_path),
            cache_key.as_deref(),
            media_type.as_deref(),
        )?;
        Ok(path.map(|p| p.to_str().map(|s| s.to_string())).unwrap_or(None))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Batch version of reader_chapter_image: returns data URIs for multiple asset IDs in one IPC call.
/// Shares a single EPUB zip open for all cache misses.
#[tauri::command]
pub async fn reader_chapter_images(
    book_id: String,
    asset_ids: Vec<String>,
    state: tauri::State<'_, BookSession>,
) -> Result<std::collections::HashMap<String, String>, String> {
    use crate::services::asset_service::AssetService;
    let svc = crate::services::asset_service_impl::AssetServiceImpl::new();
    let mut result = std::collections::HashMap::new();

    // Separate cached vs uncached
    let mut uncached_ids = Vec::new();
    for aid in &asset_ids {
        if let Some(p) = svc.image_path(&book_id, aid) {
            if let Some(uri) = read_file_to_data_uri(p.to_str().unwrap_or(""))? {
                result.insert(aid.clone(), uri);
            }
        } else {
            uncached_ids.push(aid.clone());
        }
    }

    if uncached_ids.is_empty() {
        return Ok(result);
    }

    // Extract asset info for uncached images (short lock)
    let assets_info: Vec<(String, std::path::PathBuf, String, Option<String>, Option<String>)> = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
        if book.id != book_id {
            return Ok(result);
        }
        uncached_ids
            .iter()
            .filter_map(|aid| {
                book.assets
                    .image_assets
                    .iter()
                    .find(|a| a.asset_id == *aid)
                    .filter(|img| !img.asset_path.is_empty())
                    .map(|img| {
                        (
                            aid.clone(),
                            book.source_path.clone(),
                            img.asset_path.clone(),
                            img.cache_key.clone(),
                            img.media_type.clone(),
                        )
                    })
            })
            .collect()
    };

    // Blocking work: extract from EPUB and convert to data URIs
    let book_id_clone = book_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        for (aid, epub_path, asset_path, cache_key, media_type) in assets_info {
            let path = resolve_chapter_image_cache_path_blocking(
                &book_id_clone,
                &aid,
                Some(epub_path.as_path()),
                Some(&asset_path),
                cache_key.as_deref(),
                media_type.as_deref(),
            )?;
            if let Some(p) = path {
                if let Some(uri) = read_file_to_data_uri(p.to_str().unwrap_or(""))? {
                    result.insert(aid, uri);
                }
            }
        }
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}
