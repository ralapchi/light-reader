use crate::services::asset_service_impl::AssetServiceImpl;
use crate::services::library_service_impl::LibraryServiceImpl;
use crate::services::reader_service_impl::ReaderServiceImpl;

use super::super::dto::*;
use super::dto_convert::{block_to_dto, build_reader_book_dto, read_file_to_data_uri};
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
    app: tauri::AppHandle,
) -> Result<ReaderBookDto, String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::{
        BookOpeningFailed, BookOpeningFinished, BookOpeningProgress, BookOpeningStarted,
    };
    use std::time::Instant;
    let emitter = EventEmitter::new(&app);
    let start = Instant::now();

    // Load index once and validate book exists / file is present.
    let mut index = LibraryServiceImpl::load_index();
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

    emitter.book_opening_progress(&BookOpeningProgress {
        book_id: book_id.clone(),
        stage: "parsing".to_string(),
        progress_text: Some("正在打开...".to_string()),
    });

    let source_path = item.source_path.clone();
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

    // Update last_opened_at in library index.
    if let Some(item) = index.items.iter_mut().find(|i| i.book_id == book_id) {
        item.last_opened_at = Some(chrono::Utc::now().to_rfc3339());
        LibraryServiceImpl::save_index(&index);
    }

    emitter.book_opening_finished(&BookOpeningFinished {
        book_id: book_id.clone(),
        chapter_count: dto.chapter_count,
        load_duration_ms: start.elapsed().as_millis() as u64,
    });

    Ok(dto)
}

#[tauri::command]
pub fn reader_get_chapter(
    chapter_index: usize,
    state: tauri::State<'_, BookSession>,
) -> Result<ReaderChapterDto, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    let chapter = book.chapters.get(chapter_index).ok_or_else(|| {
        format!(
            "章节 {} 不存在 (共 {} 章)",
            chapter_index,
            book.chapters.len()
        )
    })?;

    Ok(ReaderChapterDto {
        chapter_index,
        title: chapter.title.clone(),
        blocks: chapter
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| block_to_dto(b, i))
            .collect(),
        char_count: chapter.char_count,
    })
}

/// Resolve an EPUB href (optionally with fragment) to chapter_index and paragraph_index.
#[tauri::command]
pub fn reader_resolve_href(
    href: String,
    from_chapter_index: Option<usize>,
    state: tauri::State<'_, BookSession>,
) -> Result<Option<ReaderResolvedLinkDto>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    let (file_part, fragment) = match href.split_once('#') {
        Some((f, frag)) => (f.to_string(), Some(frag.to_string())),
        None => (href.clone(), None),
    };

    if file_part.is_empty() {
        let chapter_index = from_chapter_index.unwrap_or(0);
        if chapter_index >= book.chapters.len() {
            return Ok(None);
        }
        let paragraph_index = fragment.as_ref().and_then(|fragment| {
            book.chapters[chapter_index]
                .anchors
                .iter()
                .find(|(id, _)| id == fragment)
                .map(|(_, pi)| *pi)
        });
        return Ok(Some(ReaderResolvedLinkDto {
            chapter_index,
            paragraph_index,
            block_index: paragraph_index,
            scroll_offset: None,
        }));
    }

    // Normalize file part: take just the filename for matching
    let target_file = file_part.rsplit('/').next().unwrap_or(&file_part);
    // Guard against bare "/" or empty file paths
    if target_file.is_empty() {
        return Ok(None);
    }

    // 1. Find the chapter by matching source_href ending
    let chapter_index = book.chapters.iter().position(|ch| {
        ch.source_href
            .as_ref()
            .map(|h| {
                h == &file_part || h.ends_with(&format!("/{}", target_file)) || h == target_file
            })
            .unwrap_or(false)
    });

    let chapter_index = match chapter_index {
        Some(ci) => ci,
        None => {
            // Fallback: match by href filename only (for EPUBs where paths are inconsistent)
            match book.chapters.iter().position(|ch| {
                ch.source_href
                    .as_ref()
                    .map(|h| {
                        let ch_file = h.rsplit('/').next().unwrap_or(h);
                        ch_file == target_file
                    })
                    .unwrap_or(false)
            }) {
                Some(ci) => ci,
                None => {
                    // Try from_chapter_index as base for fragment-only hrefs
                    if file_part.is_empty() || href.starts_with('#') {
                        from_chapter_index.unwrap_or(0)
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    };

    // 2. If no fragment, just return chapter_index
    let fragment = match fragment {
        Some(f) => f,
        None => {
            return Ok(Some(ReaderResolvedLinkDto {
                chapter_index,
                paragraph_index: None,
                block_index: None,
                scroll_offset: None,
            }));
        }
    };

    // 3. Look up the anchor within the chapter
    let chapter = &book.chapters[chapter_index];
    let paragraph_index = chapter
        .anchors
        .iter()
        .find(|(id, _)| id == &fragment)
        .map(|(_, pi)| *pi);

    Ok(Some(ReaderResolvedLinkDto {
        chapter_index,
        paragraph_index,
        block_index: paragraph_index, // blocks are 1:1 with paragraphs in current model
        scroll_offset: None,
    }))
}

/// Resolve a chapter image to its on-disk cache path, extracting from EPUB on cache miss.
fn resolve_chapter_image_cache_path(
    book_id: &str,
    asset_id: &str,
    state: &tauri::State<'_, BookSession>,
) -> Result<Option<std::path::PathBuf>, String> {
    use crate::services::asset_service::AssetService;
    let svc = crate::services::asset_service_impl::AssetServiceImpl::new();

    // Check disk cache first
    if let Some(p) = svc.image_path(book_id, asset_id) {
        return Ok(Some(p));
    }

    // Cache miss — extract from EPUB on-demand
    let (epub_path, asset_path, cache_key, media_type) = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
        match book
            .assets
            .image_assets
            .iter()
            .find(|a| a.asset_id == asset_id)
        {
            Some(img) if !img.asset_path.is_empty() => (
                book.source_path.clone(),
                img.asset_path.clone(),
                img.cache_key.clone(),
                img.media_type.clone(),
            ),
            _ => return Ok(None),
        }
    };

    if let Some(bytes) =
        crate::services::asset_service_impl::extract_epub_image(&epub_path, &asset_path)
    {
        let ext = if let Some(ref key) = cache_key {
            key.rsplit('.').next().unwrap_or("png").to_string()
        } else {
            match media_type.as_deref() {
                Some("image/jpeg") => "jpg".to_string(),
                Some("image/webp") => "webp".to_string(),
                Some("image/gif") => "gif".to_string(),
                Some("image/svg+xml") => "svg".to_string(),
                _ => "png".to_string(),
            }
        };
        let key = cache_key.unwrap_or_else(|| format!("{}.{}", asset_id, ext));
        svc.cache_chapter_image(book_id, asset_id, &key, &bytes);
        let cache_path = crate::storage::paths::image_cache_path(book_id, asset_id, &ext);
        return Ok(Some(cache_path));
    }

    Ok(None)
}

/// Get a chapter image as a base64 data URI by its asset_id.
/// On cache miss, extracts from EPUB on-demand.
#[tauri::command]
pub fn reader_chapter_image(
    book_id: String,
    asset_id: String,
    state: tauri::State<'_, BookSession>,
) -> Result<Option<String>, String> {
    match resolve_chapter_image_cache_path(&book_id, &asset_id, &state)? {
        Some(p) => read_file_to_data_uri(p.to_str().unwrap_or("")),
        None => Ok(None),
    }
}

/// Get a chapter image file path by its asset_id (for use with convertFileSrc).
/// On cache miss, extracts from EPUB on-demand and caches to disk.
#[tauri::command]
pub fn reader_chapter_image_path(
    book_id: String,
    asset_id: String,
    state: tauri::State<'_, BookSession>,
) -> Result<Option<String>, String> {
    match resolve_chapter_image_cache_path(&book_id, &asset_id, &state)? {
        Some(p) => Ok(p.to_str().map(|s| s.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn reader_go_to_chapter(
    chapter_index: usize,
    state: tauri::State<'_, BookSession>,
) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_mut().ok_or("没有打开的书籍")?;
    if chapter_index >= book.chapters.len() {
        return Err(format!(
            "章节 {} 不存在 (共 {} 章)",
            chapter_index,
            book.chapters.len()
        ));
    }
    // Session state chapter index is managed by the frontend; the backend just validates.
    Ok(())
}

#[tauri::command]
pub fn reader_save_progress(mut progress: SaveProgressDto) -> Result<(), String> {
    progress.progress_percent = progress.progress_percent.clamp(0.0, 1.0);
    log::info!(
        "保存进度: book={}, ch={}, pct={:.0}%",
        progress.book_id,
        progress.chapter_index,
        progress.progress_percent * 100.0
    );
    use crate::domain::reading_progress::ReadingProgress;
    let existing = crate::storage::progress_store::load(&progress.book_id);
    let same_chapter_existing = existing
        .as_ref()
        .filter(|p| p.chapter_index == progress.chapter_index);
    let clear = progress.clear_position.unwrap_or(false);
    let no_position_supplied =
        !clear && progress.paragraph_index.is_none() && progress.scroll_offset.is_none();
    let rp = ReadingProgress {
        book_id: progress.book_id.clone(),
        chapter_index: progress.chapter_index,
        paragraph_index: if clear {
            None
        } else if no_position_supplied {
            same_chapter_existing.and_then(|p| p.paragraph_index)
        } else {
            progress.paragraph_index
        },
        scroll_offset: if clear {
            0.0
        } else {
            progress
                .scroll_offset
                .or_else(|| same_chapter_existing.map(|p| p.scroll_offset))
                .unwrap_or(0.0)
        },
        progress_percent: progress.progress_percent,
        last_read_at: chrono::Utc::now().to_rfc3339(),
        session_read_seconds: existing
            .as_ref()
            .map(|p| p.session_read_seconds)
            .unwrap_or(0),
        total_read_seconds: existing.as_ref().map(|p| p.total_read_seconds).unwrap_or(0),
        anchor: if clear {
            None
        } else {
            progress.anchor.map(|a| crate::domain::reader_anchor::ReaderAnchor {
                chapter_id: a.chapter_id,
                block_id: a.block_id,
                char_offset: a.char_offset,
            })
        },
    };
    ReaderServiceImpl::persist_progress(&rp.book_id, &rp, None, 0);

    // Also update the library index so "继续阅读" works.
    let mut index = LibraryServiceImpl::load_index();
    if let Some(item) = index
        .items
        .iter_mut()
        .find(|i| i.book_id == progress.book_id)
    {
        item.progress_percent = progress.progress_percent;
        item.last_opened_at = Some(chrono::Utc::now().to_rfc3339());
        LibraryServiceImpl::save_index(&index);
    }

    Ok(())
}

/// Load saved reading progress for a book (chapter + scroll offset).
#[tauri::command]
pub fn reader_get_progress(book_id: String) -> Option<SaveProgressDto> {
    crate::storage::progress_store::load(&book_id).map(|rp| SaveProgressDto {
        book_id: rp.book_id,
        chapter_index: rp.chapter_index,
        progress_percent: rp.progress_percent,
        paragraph_index: rp.paragraph_index,
        scroll_offset: Some(rp.scroll_offset),
        anchor: rp.anchor.map(|a| ReaderAnchorDto {
            chapter_id: a.chapter_id,
            block_id: a.block_id,
            char_offset: a.char_offset,
        }),
        clear_position: None,
    })
}
