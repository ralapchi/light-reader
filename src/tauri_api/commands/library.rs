use crate::domain::book_format::BookFormat;
use crate::services::library_service::LibraryService;
use crate::services::library_service_impl::LibraryServiceImpl;

use super::super::dto::*;
use super::dto_convert::{item_to_dto, read_file_to_data_uri};

#[tauri::command]
pub fn library_list() -> Result<Vec<LibraryBookCardDto>, String> {
    let svc = LibraryServiceImpl::new();
    let items = svc.list_books();
    Ok(items.iter().map(|i| item_to_dto(i)).collect())
}

#[tauri::command]
pub fn library_import(paths: Vec<String>) -> Result<Vec<LibraryBookCardDto>, String> {
    let svc = LibraryServiceImpl::new();
    let items = svc.import_books(paths).map_err(|e| e.to_string())?;
    Ok(items.iter().map(|i| item_to_dto(i)).collect())
}

#[tauri::command]
pub fn library_open(book_id: String) -> Result<(), String> {
    let svc = LibraryServiceImpl::new();
    svc.open_book(&book_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn library_remove(book_id: String, delete_files: bool) -> Result<(), String> {
    let svc = LibraryServiceImpl::new();
    let source = if delete_files {
        let index = LibraryServiceImpl::load_index();
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

    svc.remove_book(&book_id).map_err(|e| e.to_string())?;
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
pub fn library_remove_batch(book_ids: Vec<String>, delete_files: bool) -> Result<(), String> {
    let svc = LibraryServiceImpl::new();
    let index = LibraryServiceImpl::load_index();
    let source_paths: Vec<String> = if delete_files {
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
    };
    let mut failures = Vec::new();
    for id in &book_ids {
        if let Err(e) = svc.remove_book(id) {
            failures.push(format!("{}: {}", id, e));
        }
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
pub fn library_search(query: String) -> Result<Vec<LibraryBookCardDto>, String> {
    let svc = LibraryServiceImpl::new();
    let items = svc.search(&query);
    Ok(items.iter().map(|i| item_to_dto(i)).collect())
}

#[tauri::command]
pub fn library_repair_path(book_id: String, new_path: String) -> Result<(), String> {
    let svc = LibraryServiceImpl::new();
    svc.repair_path(&book_id, &new_path)
        .map_err(|e| e.to_string())
}

/// Get a book's cover image as a data URI by book_id.
#[tauri::command]
pub fn library_cover(book_id: String) -> Result<Option<String>, String> {
    use crate::services::asset_service::AssetService;
    let svc = crate::services::asset_service_impl::AssetServiceImpl::new();
    let path = svc.cover_path(&book_id).or_else(|| {
        // Cache miss: try lightweight EPUB cover extraction
        let index = LibraryServiceImpl::load_index();
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
        Some(p) => read_file_to_data_uri(p.to_str().unwrap_or("")),
        None => Ok(None),
    }
}

/// Read a file by absolute path and return a base64 data URI.
#[tauri::command]
pub fn asset_read_file(path: String) -> Result<Option<String>, String> {
    let requested = std::path::Path::new(&path);
    let canonical = match std::fs::canonicalize(requested) {
        Ok(p) => p,
        Err(_) => return Ok(None),
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
    read_file_to_data_uri(canonical.to_str().unwrap_or(""))
}
