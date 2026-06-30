use crate::services::asset_service_impl::AssetServiceImpl;
use crate::services::reader_service_impl::ReaderServiceImpl;

use super::super::dto::*;
use super::dto_convert::build_reader_book_dto;
use super::BookSession;

#[tauri::command]
pub fn reader_get_book(
    state: tauri::State<'_, BookSession>,
) -> Result<Option<ReaderBookDto>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    match guard.book.as_ref() {
        Some(book) => Ok(Some(build_reader_book_dto(&book))),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn reader_open_book(
    book_id: String,
    state: tauri::State<'_, BookSession>,
    index_state: tauri::State<'_, super::LibraryIndexState>,
    progress_state: tauri::State<'_, super::ProgressState>,
    db: tauri::State<'_, Box<dyn crate::storage::traits::DatabaseBackend>>,
    app: tauri::AppHandle,
) -> Result<ReaderBookDto, String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::{
        BookOpeningFailed, BookOpeningFinished, BookOpeningProgress, BookOpeningStarted,
    };
    use std::time::Instant;
    let emitter = EventEmitter::new(&app);
    let start = Instant::now();

    // Validate book exists and extract source_path from cached index, then drop guard.
    let source_path = {
        let index = index_state.lock().map_err(|e| e.to_string())?;
        let item = index
            .items
            .iter()
            .find(|i| i.book_id == book_id)
            .ok_or_else(|| {
                let msg = "书籍不在书库中".to_string();
                emitter.book_opening_failed(&BookOpeningFailed {
                    book_id: Some(book_id.clone()),
                    error_code: "not_found".to_string(),
                    error_message: msg.clone(),
                    recoverable: false,
                });
                msg
            })?;
        if item.file_health == crate::domain::library_item::FileHealth::Missing {
            let msg = "书籍文件缺失".to_string();
            emitter.book_opening_failed(&BookOpeningFailed {
                book_id: Some(book_id.clone()),
                error_code: "validation".to_string(),
                error_message: msg.clone(),
                recoverable: true,
            });
            return Err(msg);
        }

        emitter.book_opening_started(&BookOpeningStarted {
            book_id: book_id.clone(),
            title: item.title.clone(),
            author: item.author.clone(),
        });
        item.source_path.clone()
    };

    emitter.book_opening_progress(&BookOpeningProgress {
        book_id: book_id.clone(),
        stage: "parsing".to_string(),
        progress_text: Some("正在打开...".to_string()),
    });

    let mut book =
        tauri::async_runtime::spawn_blocking(move || ReaderServiceImpl::load_book(&source_path))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| {
                emitter.book_opening_failed(&BookOpeningFailed {
                    book_id: Some(book_id.clone()),
                    error_code: "parse_error".to_string(),
                    error_message: e.to_string(),
                    recoverable: true,
                });
                e.to_string()
            })?;

    // Cache cover only if not already cached; chapter images are cached on-demand
    if AssetServiceImpl::needs_cover_caching(&book_id, &book) {
        emitter.book_opening_progress(&BookOpeningProgress {
            book_id: book_id.clone(),
            stage: "caching".to_string(),
            progress_text: Some("缓存封面...".to_string()),
        });
    }
    AssetServiceImpl::cache_cover_only(&book_id, &book);

    // Strip image bytes from session to free memory
    book.assets.cover_image_bytes = None;

    let dto = build_reader_book_dto(&book);

    // Store the book in session state.
    state.lock().map_err(|e| e.to_string())?.book = Some(book);

    // Load persisted progress into memory before the reader page restores position.
    {
        let mut progress_map = progress_state.lock().map_err(|e| e.to_string())?;
        if !progress_map.contains_key(&book_id) {
            if let Ok(Some(progress)) = db.progress().load(&book_id) {
                progress_map.insert(book_id.clone(), progress);
            }
        }
    }

    // Update last_opened_at in cached library index and persist to DB.
    {
        let mut index = index_state.lock().map_err(|e| e.to_string())?;
        if let Some(item) = index.items.iter_mut().find(|i| i.book_id == book_id) {
            item.last_opened_at = Some(chrono::Utc::now().to_rfc3339());
            if let Err(e) = db.books().upsert(item) {
                log::warn!("打开书籍时写入数据库失败: {}", e);
            }
        }
    }

    emitter.book_opening_finished(&BookOpeningFinished {
        book_id: book_id.clone(),
        chapter_count: dto.chapter_count,
        load_duration_ms: start.elapsed().as_millis() as u64,
    });

    Ok(dto)
}
