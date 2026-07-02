use crate::domain::book_format::BookFormat;
use crate::domain::library_item::LibraryItem;
use crate::services::library_service_impl::LibraryServiceImpl;

use super::super::dto::*;
use super::dto_convert::{item_to_dto, read_file_to_data_uri};

fn cleanup_book_data(
    book_id: &str,
    progress_state: &tauri::State<'_, super::ProgressState>,
    dirty_progress_state: &tauri::State<'_, super::DirtyProgressState>,
    progress_revision_state: &tauri::State<'_, super::ProgressRevisionState>,
) {
    if let Ok(mut map) = progress_state.lock() {
        map.remove(book_id);
    }
    if let Ok(mut dirty) = dirty_progress_state.lock() {
        dirty.remove(book_id);
    }
    if let Ok(mut revs) = progress_revision_state.lock() {
        revs.remove(book_id);
    }
    if let Some(cover_path) = crate::storage::paths::find_cover_by_extensions(book_id) {
        let _ = std::fs::remove_file(&cover_path);
    }
    let img_dir = crate::storage::paths::app_data_dir()
        .join("cache/images")
        .join(book_id);
    if img_dir.exists() {
        let _ = std::fs::remove_dir_all(&img_dir);
    }
    let tts_dir = crate::storage::paths::tts_cache_dir();
    if tts_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&tts_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let book_tts = entry.path().join(book_id);
                    if book_tts.exists() {
                        let _ = std::fs::remove_dir_all(&book_tts);
                    }
                }
            }
        }
    }
}

/// Parse a single book file and extract its cover (heavy I/O, suitable for spawn_blocking).
fn parse_single_book(path: &str, now: &str) -> Option<LibraryItem> {
    let mut item = match LibraryServiceImpl::parse_book_item(path, now) {
        Ok(item) => item,
        Err(e) => {
            log::warn!("导入书籍失败: {} - {}", path, e);
            return None;
        }
    };

    let book_id = item.book_id.clone();
    if item.format == BookFormat::Epub {
        let epub_path = std::path::Path::new(&item.source_path);
        if epub_path.exists() {
            if let Some(cover_path) =
                crate::services::asset_service_impl::extract_and_cache_cover(epub_path, &book_id)
            {
                if let Some(ext) = cover_path.extension().and_then(|e| e.to_str()) {
                    item.cover_cache_key = Some(ext.to_string());
                }
            }
        }
    }

    Some(item)
}

#[tauri::command]
pub async fn library_list(
    index_state: tauri::State<'_, super::LibraryIndexState>,
) -> Result<Vec<LibraryBookCardDto>, String> {
    let items = {
        let index = index_state.lock().map_err(|e| e.to_string())?;
        index.items.clone()
    };
    tauri::async_runtime::spawn_blocking(move || {
        items.iter().map(item_to_dto).collect()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn library_import(
    paths: Vec<String>,
    index_state: tauri::State<'_, super::LibraryIndexState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<Vec<LibraryBookCardDto>, String> {
    let now = chrono::Utc::now().to_rfc3339();

    // Heavy I/O: parse books and extract covers in spawn_blocking
    let parsed: Vec<LibraryItem> = {
        let paths_clone = paths.clone();
        tauri::async_runtime::spawn_blocking(move || {
            paths_clone
                .iter()
                .filter_map(|path| parse_single_book(path, &now))
                .collect::<Vec<_>>()
        })
        .await
        .map_err(|e| e.to_string())?
    };

    // Update index and write to DB (relatively fast)
    let imported = {
        let mut index = index_state.lock().map_err(|e| e.to_string())?;
        let mut imported = Vec::new();
        for item in parsed {
            let book_id = item.book_id.clone();
            if let Some(existing) = index.items.iter_mut().find(|i| i.book_id == book_id) {
                let imported_at = existing.imported_at.clone();
                *existing = item.clone();
                existing.imported_at = imported_at;
            } else {
                index.items.push(item.clone());
            }
            index.last_selected_book_id = Some(book_id.clone());

            if let Some(stored) = index.items.iter().find(|i| i.book_id == book_id) {
                if let Err(e) = db.books().upsert(stored) {
                    log::warn!("导入书籍到数据库失败: {}", e);
                }
            }
            let _ = db.books().set_last_selected(&book_id);

            imported.push(item);
        }
        imported
    };

    if imported.is_empty() && !paths.is_empty() {
        Err("所有书籍导入失败".to_string())
    } else {
        Ok(imported.iter().map(|i| item_to_dto(i)).collect())
    }
}

#[tauri::command]
pub fn library_open(
    book_id: String,
    index_state: tauri::State<'_, super::LibraryIndexState>,
) -> Result<(), String> {
    let index = index_state.lock().map_err(|e| e.to_string())?;
    let item = index
        .items
        .iter()
        .find(|i| i.book_id == book_id)
        .ok_or_else(|| "书籍不在书库中".to_string())?;
    if item.file_health == crate::domain::library_item::FileHealth::Missing {
        return Err("书籍文件缺失".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn library_remove(
    book_id: String,
    delete_files: bool,
    index_state: tauri::State<'_, super::LibraryIndexState>,
    progress_state: tauri::State<'_, super::ProgressState>,
    dirty_progress_state: tauri::State<'_, super::DirtyProgressState>,
    progress_revision_state: tauri::State<'_, super::ProgressRevisionState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    let source = if delete_files {
        let index = index_state.lock().map_err(|e| e.to_string())?;
        Some(
            index
                .items
                .iter()
                .find(|i| i.book_id == book_id)
                .ok_or_else(|| format!("书籍 {} 不在书库中", book_id))?
                .source_path
                .clone(),
        )
    } else {
        None
    };

    {
        let mut index = index_state.lock().map_err(|e| e.to_string())?;
        let before = index.items.len();
        index.items.retain(|i| i.book_id != book_id);
        if index.items.len() == before {
            return Err(format!("书籍 {} 不在书库中", book_id));
        }
    }

    // Delete from database (code-level cascade removes progress, bookmarks, tags, sessions)
    if let Err(e) = db.books().delete(&book_id) {
        log::warn!("从数据库删除书籍失败: {}", e);
    }

    // Clean up in-memory state and cached assets
    cleanup_book_data(&book_id, &progress_state, &dirty_progress_state, &progress_revision_state);

    if delete_files {
        if let Some(source) = source {
            match std::fs::remove_file(&source) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("源文件删除失败: {}", e)),
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn library_remove_batch(
    book_ids: Vec<String>,
    delete_files: bool,
    index_state: tauri::State<'_, super::LibraryIndexState>,
    progress_state: tauri::State<'_, super::ProgressState>,
    dirty_progress_state: tauri::State<'_, super::DirtyProgressState>,
    progress_revision_state: tauri::State<'_, super::ProgressRevisionState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    let source_paths: Vec<String> = {
        let index = index_state.lock().map_err(|e| e.to_string())?;
        if delete_files {
            book_ids
                .iter()
                .filter_map(|id| {
                    index
                        .items
                        .iter()
                        .find(|i| &i.book_id == id)
                        .map(|i| i.source_path.clone())
                })
                .collect()
        } else {
            vec![]
        }
    };

    let mut failures = Vec::new();
    {
        let mut index = index_state.lock().map_err(|e| e.to_string())?;
        for id in &book_ids {
            let before = index.items.len();
            index.items.retain(|i| &i.book_id != id);
            if index.items.len() == before {
                failures.push(format!("{}: 书籍不在书库中", id));
            } else {
                cleanup_book_data(id, &progress_state, &dirty_progress_state, &progress_revision_state);
            }
        }
    }

    // Batch delete from database (CASCADE removes all related data)
    let id_refs: Vec<&str> = book_ids.iter().map(|s| s.as_str()).collect();
    if let Err(e) = db.books().delete_batch(&id_refs) {
        log::warn!("批量删除书籍数据库记录失败: {}", e);
    }

    if delete_files {
        for path in &source_paths {
            match std::fs::remove_file(path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => failures.push(format!("{}: 源文件删除失败: {}", path, e)),
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("部分书籍移除失败: {}", failures.join("; ")))
    }
}

#[tauri::command]
pub async fn library_search(
    query: String,
    index_state: tauri::State<'_, super::LibraryIndexState>,
) -> Result<Vec<LibraryBookCardDto>, String> {
    let (items, q) = {
        let index = index_state.lock().map_err(|e| e.to_string())?;
        (index.items.clone(), query.to_lowercase())
    };
    tauri::async_runtime::spawn_blocking(move || {
        items
            .iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&q)
                    || item
                        .author
                        .as_ref()
                        .map_or(false, |a| a.to_lowercase().contains(&q))
            })
            .map(item_to_dto)
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn library_repair_path(
    book_id: String,
    new_path: String,
    index_state: tauri::State<'_, super::LibraryIndexState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    use crate::parser::ParserFactory;
    let mut index = index_state.lock().map_err(|e| e.to_string())?;
    if !index.items.iter().any(|i| i.book_id == book_id) {
        return Err("书籍不在书库中".to_string());
    }
    if ParserFactory::get_parser(&new_path).is_none() {
        return Err("不支持的文件格式".to_string());
    }
    if !std::path::Path::new(&new_path).exists() {
        return Err("修复路径不存在".to_string());
    }
    LibraryServiceImpl::repair_item_path(&mut index, &book_id, &new_path);
    if let Some(item) = index.items.iter().find(|i| i.book_id == book_id) {
        if let Err(e) = db.books().upsert(item) {
            log::warn!("修复路径后写入数据库失败: {}", e);
        }
    }
    Ok(())
}

/// Get a book's cover image as a data URI by book_id.
#[tauri::command]
pub async fn library_cover(
    book_id: String,
    index_state: tauri::State<'_, super::LibraryIndexState>,
) -> Result<Option<String>, String> {
    let path = crate::services::asset_service_impl::AssetServiceImpl::cover_path(&book_id).or_else(|| {
        let index = index_state.lock().ok()?;
        let item = index.items.iter().find(|i| i.book_id == book_id)?;
        if item.format == BookFormat::Epub {
            let epub_path = std::path::Path::new(&item.source_path);
            if epub_path.exists() {
                crate::services::asset_service_impl::extract_and_cache_cover(epub_path, &book_id)
            } else {
                None
            }
        } else {
            None
        }
    });
    match path {
        Some(p) => {
            let path_str = p.to_str().unwrap_or("").to_string();
            tauri::async_runtime::spawn_blocking(move || read_file_to_data_uri(&path_str))
                .await
                .map_err(|e| e.to_string())?
        }
        None => Ok(None),
    }
}

/// Read a file by absolute path and return a base64 data URI.
/// Blocking I/O runs off the async runtime.
#[tauri::command]
pub async fn asset_read_file(path: String) -> Result<Option<String>, String> {
    // Path validation is fast, do it before spawn_blocking
    let canonical = {
        let requested = std::path::Path::new(&path);
        match std::fs::canonicalize(requested) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        }
    };
    crate::storage::paths::ensure_dirs().map_err(|e| e.to_string())?;
    let app_data = crate::storage::paths::app_data_dir();
    let allowed_root = std::fs::canonicalize(&app_data).map_err(|e| e.to_string())?;
    let canon_comps: Vec<_> = canonical.components().collect();
    let root_comps: Vec<_> = allowed_root.components().collect();
    if canon_comps.len() < root_comps.len()
        || canon_comps[..root_comps.len()] != root_comps[..]
    {
        return Err("不允许读取应用数据目录之外的文件".to_string());
    }

    // File read + base64 encoding is blocking
    let path_str = canonical.to_str().unwrap_or("").to_string();
    tauri::async_runtime::spawn_blocking(move || read_file_to_data_uri(&path_str))
        .await
        .map_err(|e| e.to_string())?
}

/// Persist the in-memory library index to disk and database. Called on navigation back and app exit.
#[tauri::command]
pub fn library_flush_index(
    index_state: tauri::State<'_, super::LibraryIndexState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
) -> Result<(), String> {
    let index = index_state.lock().map_err(|e| e.to_string())?;
    crate::services::library_service_impl::flush_library_to_db(&index, db.as_ref());
    Ok(())
}
