//! Tauri command functions.
//!
//! Each function is annotated with `#[tauri::command]` and delegates to
//! the corresponding service implementation. Return types use DTOs
//! defined in `dto.rs` for safe serde serialization.

use crate::domain::book_format::BookFormat;
use crate::services::asset_service_impl::AssetServiceImpl;
use crate::services::library_service::LibraryService;
use crate::services::library_service_impl::LibraryServiceImpl;
use crate::services::reader_service_impl::ReaderServiceImpl;
use crate::services::settings_service::SettingsService;
use crate::services::settings_service_impl::SettingsServiceImpl;
use crate::tts::cache::TtsCache;
use crate::tts::config::TtsConfig;
use crate::tts::segmenter::Segment;
use crate::tts::synthesis_service::TtsSynthesisService;
use crate::tts::types::{PlaybackStatus, TtsProviderKind};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use super::dto::*;

/// Commands sent to the dedicated audio playback thread.
pub enum PlaybackCmd {
    Play(Vec<u8>, String),
    Pause,
    Resume,
    Stop,
}

/// Shared session state for Tauri commands.
///
/// Note: The audio player lives on a dedicated thread (rodio's OutputStream is !Send).
/// Playback control goes through `playback_tx`, and status is polled via `is_playing_flag`.
pub struct TtsSession {
    pub book: Option<crate::domain::book::Book>,
    pub tts_config: TtsConfig,
    pub cache: Arc<TtsCache>,
    pub playback_state: crate::domain::tts_state::PlaybackState,
    pub segments: Vec<Segment>,
    pub stop_flag: Arc<AtomicBool>,
    pub is_playing_flag: Arc<AtomicBool>,
    pub playback_tx: Option<mpsc::Sender<PlaybackCmd>>,
}

impl TtsSession {
    pub fn new() -> Self {
        let _ = crate::storage::paths::ensure_dirs();
        let settings_file = crate::storage::settings_store::load();

        let mut tts_config = settings_file.tts_config.unwrap_or_default();
        // Migration: clear old invalid voice IDs
        if let Some(ref vid) = tts_config.voice_id {
            if vid.contains('-') && vid.len() > 30 {
                tts_config.voice_id = None;
            }
        }

        let cache = Arc::new(TtsCache::new(crate::storage::paths::tts_cache_dir()));

        Self {
            book: None,
            tts_config,
            cache,
            playback_state: Default::default(),
            segments: Vec::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            is_playing_flag: Arc::new(AtomicBool::new(false)),
            playback_tx: None,
        }
    }
}

/// Back-compat type alias — commands use `State<'_, ReaderSession>`.
pub type ReaderSession = Mutex<TtsSession>;

// ── Helpers ──────────────────────────────────────────────────

fn item_to_dto(item: &crate::domain::library_item::LibraryItem) -> LibraryBookCardDto {
    use crate::services::asset_service::AssetService;
    let svc = crate::services::asset_service_impl::AssetServiceImpl::new();
    let cover_url = svc
        .cover_path(&item.book_id)
        .and_then(|p| p.to_str().map(|s| s.to_string()));
    LibraryBookCardDto {
        book_id: item.book_id.clone(),
        title: item.title.clone(),
        author: item.author.clone(),
        format: match item.format {
            BookFormat::Epub => "epub".to_string(),
            BookFormat::Txt => "txt".to_string(),
        },
        cover_url,
        progress_percent: item.progress_percent,
        chapter_count: item.chapter_count,
        last_opened_at: item.last_opened_at.clone(),
        imported_at: item.imported_at.clone(),
        file_ok: item.file_health == crate::domain::library_item::FileHealth::Ok,
    }
}

fn toc_to_dto(toc: &crate::domain::toc_item::TocItem) -> TocItemDto {
    TocItemDto {
        id: toc.id.clone(),
        title: toc.title.clone(),
        chapter_index: toc.chapter_index,
        href: toc.href.clone(),
        depth: toc.depth as usize,
        children: toc.children.iter().map(toc_to_dto).collect(),
    }
}

fn block_to_dto(
    block: &crate::domain::chapter_block::ChapterBlock,
    block_index: usize,
) -> ReaderBlockDto {
    match block {
        crate::domain::chapter_block::ChapterBlock::Paragraph(p) => ReaderBlockDto::Paragraph {
            index: p.index,
            block_id: format!("p-{}", p.index),
            text: p.text.clone(),
            kind: format!("{:?}", p.kind).to_lowercase(),
            links: p
                .links
                .iter()
                .map(|l| ReaderTextLinkDto {
                    start: l.start,
                    end: l.end,
                    href: l.href.clone(),
                    title: l.title.clone(),
                })
                .collect(),
        },
        crate::domain::chapter_block::ChapterBlock::Heading(p) => ReaderBlockDto::Heading {
            index: p.index,
            block_id: format!("h-{}", p.index),
            text: p.text.clone(),
            kind: format!("{:?}", p.kind).to_lowercase(),
            links: p
                .links
                .iter()
                .map(|l| ReaderTextLinkDto {
                    start: l.start,
                    end: l.end,
                    href: l.href.clone(),
                    title: l.title.clone(),
                })
                .collect(),
        },
        crate::domain::chapter_block::ChapterBlock::Quote(p) => ReaderBlockDto::Quote {
            index: p.index,
            block_id: format!("q-{}", p.index),
            text: p.text.clone(),
            links: p
                .links
                .iter()
                .map(|l| ReaderTextLinkDto {
                    start: l.start,
                    end: l.end,
                    href: l.href.clone(),
                    title: l.title.clone(),
                })
                .collect(),
        },
        crate::domain::chapter_block::ChapterBlock::Image(img) => ReaderBlockDto::Image {
            index: img.index,
            block_id: format!("img-{}", img.index),
            asset_id: img.asset_id.clone(),
            alt_text: img.alt_text.clone(),
            caption: img.caption.clone(),
        },
        crate::domain::chapter_block::ChapterBlock::Separator => ReaderBlockDto::Separator {
            block_id: format!("sep-{}", block_index),
        },
    }
}

fn build_reader_book_dto(book: &crate::domain::book::Book) -> ReaderBookDto {
    ReaderBookDto {
        book_id: book.id.clone(),
        title: book.metadata.title.clone(),
        author: book.metadata.author.clone(),
        format: match book.format {
            BookFormat::Epub => "epub".to_string(),
            BookFormat::Txt => "txt".to_string(),
        },
        chapter_count: book.chapters.len(),
        toc: book.toc.iter().map(toc_to_dto).collect(),
    }
}

fn tts_config_to_dto(config: &TtsConfig) -> TtsConfigDto {
    TtsConfigDto {
        enabled: config.enabled,
        provider: format!("{:?}", config.provider).to_lowercase(),
        has_api_key: config.api_key.is_some(),
        api_key: None,
        base_url: config.base_url.clone(),
        model: config.model.clone(),
        voice_id: config.voice_id.clone(),
    }
}

fn dto_to_tts_config(dto: &TtsConfigDto, api_key: Option<String>) -> TtsConfig {
    let provider = match dto.provider.as_str() {
        #[cfg(feature = "tts-aliyun")]
        "aliyun" => TtsProviderKind::Aliyun,
        _ => TtsProviderKind::Xiaomi,
    };
    TtsConfig {
        enabled: dto.enabled,
        provider,
        api_key: dto.api_key.clone().or(api_key),
        base_url: dto.base_url.clone(),
        model: dto.model.clone(),
        voice_id: dto.voice_id.clone(),
    }
}

// ── Library Commands ────────────────────────────────────────

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

// ── Reader Commands ─────────────────────────────────────────

#[tauri::command]
pub fn reader_get_book(
    state: tauri::State<'_, ReaderSession>,
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
    state: tauri::State<'_, ReaderSession>,
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

/// Round `idx` down to the nearest valid UTF-8 char boundary in `s`.
fn snap_to_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[tauri::command]
pub fn reader_get_chapter(
    chapter_index: usize,
    state: tauri::State<'_, ReaderSession>,
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
    state: tauri::State<'_, ReaderSession>,
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

/// Get a chapter image as a base64 data URI by its asset_id.
/// On cache miss, extracts from EPUB on-demand.
#[tauri::command]
pub fn reader_chapter_image(
    book_id: String,
    asset_id: String,
    state: tauri::State<'_, ReaderSession>,
) -> Result<Option<String>, String> {
    use crate::services::asset_service::AssetService;
    let svc = crate::services::asset_service_impl::AssetServiceImpl::new();

    // Check disk cache first
    if let Some(p) = svc.image_path(&book_id, &asset_id) {
        return read_file_to_data_uri(p.to_str().unwrap_or(""));
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
        svc.cache_chapter_image(&book_id, &asset_id, &key, &bytes);
        let cache_path = crate::storage::paths::image_cache_path(&book_id, &asset_id, &ext);
        return read_file_to_data_uri(cache_path.to_str().unwrap_or(""));
    }

    Ok(None)
}

/// Get a chapter image file path by its asset_id (for use with convertFileSrc).
/// On cache miss, extracts from EPUB on-demand and caches to disk.
#[tauri::command]
pub fn reader_chapter_image_path(
    book_id: String,
    asset_id: String,
    state: tauri::State<'_, ReaderSession>,
) -> Result<Option<String>, String> {
    use crate::services::asset_service::AssetService;
    let svc = crate::services::asset_service_impl::AssetServiceImpl::new();

    // Check disk cache first
    if let Some(p) = svc.image_path(&book_id, &asset_id) {
        return Ok(p.to_str().map(|s| s.to_string()));
    }

    // Cache miss — extract from EPUB on-demand
    let (epub_path, asset_path, cache_key) = {
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
            "png".to_string()
        };
        let key = cache_key.unwrap_or_else(|| format!("{}.{}", asset_id, ext));
        svc.cache_chapter_image(&book_id, &asset_id, &key, &bytes);
        let cache_path = crate::storage::paths::image_cache_path(&book_id, &asset_id, &ext);
        return Ok(cache_path.to_str().map(|s| s.to_string()));
    }

    Ok(None)
}

#[tauri::command]
pub fn reader_go_to_chapter(
    chapter_index: usize,
    state: tauri::State<'_, ReaderSession>,
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
    let no_position_supplied =
        progress.paragraph_index.is_none() && progress.scroll_offset.is_none();
    let rp = ReadingProgress {
        book_id: progress.book_id.clone(),
        chapter_index: progress.chapter_index,
        paragraph_index: if no_position_supplied {
            same_chapter_existing.and_then(|p| p.paragraph_index)
        } else {
            progress.paragraph_index
        },
        scroll_offset: progress
            .scroll_offset
            .or_else(|| same_chapter_existing.map(|p| p.scroll_offset))
            .unwrap_or(0.0),
        progress_percent: progress.progress_percent,
        last_read_at: chrono::Utc::now().to_rfc3339(),
        session_read_seconds: existing
            .as_ref()
            .map(|p| p.session_read_seconds)
            .unwrap_or(0),
        total_read_seconds: existing.as_ref().map(|p| p.total_read_seconds).unwrap_or(0),
        anchor: progress.anchor.map(|a| crate::domain::reader_anchor::ReaderAnchor {
            chapter_id: a.chapter_id,
            block_id: a.block_id,
            char_offset: a.char_offset,
        }),
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
    })
}

// ── Search / Bookmark Commands ──────────────────────────────

#[tauri::command]
pub fn search_in_book(
    query: String,
    state: tauri::State<'_, ReaderSession>,
) -> Result<Vec<SearchHitDto>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    let _total_chapters = book.chapters.len().max(1);
    let mut hits = Vec::new();

    for chapter in &book.chapters {
        for para in &chapter.paragraphs {
            if let Some(pos) = para.text.find(&query) {
                let raw_start = pos.saturating_sub(30);
                let start = snap_to_char_boundary(&para.text, raw_start);
                let raw_end = (pos + query.len() + 30).min(para.text.len());
                let end = snap_to_char_boundary(&para.text, raw_end);
                let mut context = String::new();
                if start > 0 {
                    context.push_str("...");
                }
                context.push_str(&para.text[start..end]);
                if end < para.text.len() {
                    context.push_str("...");
                }

                let progress_hint = format!(
                    "约 {}% 处",
                    ((para.index as f32 / chapter.paragraphs.len().max(1) as f32) * 100.0) as u32
                );

                hits.push(SearchHitDto {
                    chapter_index: chapter.index,
                    chapter_title: chapter.title.clone(),
                    context,
                    progress_hint,
                    paragraph_index: para.index,
                });
                if hits.len() >= 50 {
                    return Ok(hits);
                }
            }
        }
    }
    Ok(hits)
}

#[tauri::command]
pub fn bookmark_list(book_id: String) -> Result<Vec<BookmarkDto>, String> {
    let items = crate::storage::bookmark_store::load(&book_id);
    Ok(items
        .into_iter()
        .map(|b| BookmarkDto {
            id: b.id,
            book_id: b.book_id,
            chapter_index: b.chapter_index,
            paragraph_index: b.paragraph_index,
            title: b.title,
            snippet: b.snippet,
            created_at: b.created_at,
            note: b.note,
        })
        .collect())
}

#[tauri::command]
pub fn bookmark_list_all() -> Result<Vec<BookmarkDto>, String> {
    let items = crate::storage::bookmark_store::load_all();
    Ok(items
        .into_iter()
        .map(|b| BookmarkDto {
            id: b.id,
            book_id: b.book_id,
            chapter_index: b.chapter_index,
            paragraph_index: b.paragraph_index,
            title: b.title,
            snippet: b.snippet,
            created_at: b.created_at,
            note: b.note,
        })
        .collect())
}

#[tauri::command]
pub fn bookmark_add(
    book_id: String,
    chapter_index: usize,
    paragraph_index: Option<usize>,
    note: Option<String>,
    state: tauri::State<'_, ReaderSession>,
) -> Result<BookmarkDto, String> {
    use crate::domain::bookmark::Bookmark;

    let guard = state.lock().map_err(|e| e.to_string())?;
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;

    let chapter = book
        .chapters
        .get(chapter_index)
        .ok_or_else(|| format!("章节 {} 不存在", chapter_index))?;

    let snippet = match paragraph_index {
        Some(pi) => chapter
            .paragraphs
            .get(pi)
            .map(|p| {
                let s = &p.text;
                if s.len() > 80 {
                    let end = snap_to_char_boundary(s, 80);
                    s[..end].to_string()
                } else {
                    s.clone()
                }
            })
            .unwrap_or_default(),
        None => chapter.title.clone(),
    };

    let bm = Bookmark {
        id: uuid::Uuid::new_v4().to_string(),
        book_id: book_id.clone(),
        chapter_index,
        paragraph_index,
        title: chapter.title.clone(),
        snippet: snippet.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        note: note.clone(),
    };

    let mut items = crate::storage::bookmark_store::load(&book_id);
    items.push(bm.clone());
    crate::storage::bookmark_store::save(&book_id, &items).map_err(|e| e.to_string())?;

    Ok(BookmarkDto {
        id: bm.id,
        book_id: bm.book_id,
        chapter_index: bm.chapter_index,
        paragraph_index: bm.paragraph_index,
        title: bm.title,
        snippet: bm.snippet,
        created_at: bm.created_at,
        note: bm.note,
    })
}

#[tauri::command]
pub fn bookmark_remove(book_id: String, bookmark_id: String) -> Result<(), String> {
    let mut items = crate::storage::bookmark_store::load(&book_id);
    let original_len = items.len();
    items.retain(|b| b.id != bookmark_id);
    if items.len() == original_len {
        return Err(format!("书签 {} 不存在", bookmark_id));
    }
    crate::storage::bookmark_store::save(&book_id, &items).map_err(|e| e.to_string())
}

// ── Settings Commands ───────────────────────────────────────

#[tauri::command]
pub fn settings_load() -> Result<serde_json::Value, String> {
    let svc = SettingsServiceImpl::new();
    let settings = svc.load_settings();
    serde_json::to_value(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn settings_save(settings: serde_json::Value) -> Result<(), String> {
    let parsed: crate::domain::reader_settings::ReaderSettings =
        serde_json::from_value(settings).map_err(|e| e.to_string())?;
    let svc = SettingsServiceImpl::new();
    svc.save_settings(&parsed).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn tts_config_load() -> Result<TtsConfigDto, String> {
    let svc = SettingsServiceImpl::new();
    let config = svc.load_tts_config();
    Ok(tts_config_to_dto(&config))
}

#[tauri::command]
pub fn tts_config_save(config: TtsConfigDto) -> Result<(), String> {
    let existing = SettingsServiceImpl::new().load_tts_config();
    let full = dto_to_tts_config(&config, existing.api_key);
    let svc = SettingsServiceImpl::new();
    svc.save_tts_config(&full).map_err(|e| e.to_string())
}

// ── Asset Commands ──────────────────────────────────────────

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

fn read_file_to_data_uri(path: &str) -> Result<Option<String>, String> {
    use base64::Engine;
    let p = std::path::Path::new(path);
    if !p.exists() {
        return Ok(None);
    }
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    };
    let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(Some(format!("data:{};base64,{}", mime, b64)))
}

// ── TTS Commands ────────────────────────────────────────────

/// Spawn a dedicated audio playback thread (rodio OutputStream is !Send).
fn spawn_playback_thread() -> (mpsc::Sender<PlaybackCmd>, Arc<AtomicBool>) {
    let (tx, rx) = mpsc::channel::<PlaybackCmd>();
    let is_playing = Arc::new(AtomicBool::new(false));

    let playing_flag = Arc::clone(&is_playing);
    std::thread::spawn(move || {
        use crate::tts::player::AudioPlayer;
        let mut player = match AudioPlayer::new() {
            Ok(p) => Some(p),
            Err(e) => {
                log::warn!("音频播放器初始化失败: {}", e);
                // Drain commands without panicking
                for cmd in rx {
                    if matches!(cmd, PlaybackCmd::Stop) {
                        break;
                    }
                }
                return;
            }
        };

        loop {
            let p = match player.as_mut() {
                Some(p) => p,
                None => break,
            };

            // Detect when sink finishes playing naturally
            if playing_flag.load(Ordering::Relaxed) && p.is_empty() && !p.is_paused() {
                playing_flag.store(false, Ordering::Relaxed);
            }

            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(cmd) => match cmd {
                    PlaybackCmd::Play(data, media_type) => {
                        let res = if media_type == "audio/pcm16" {
                            p.append_pcm16(data)
                        } else {
                            p.append(data)
                        };
                        if let Err(e) = res {
                            log::warn!("TTS 音频播放失败: {}", e);
                        }
                        playing_flag.store(true, Ordering::Relaxed);
                    }
                    PlaybackCmd::Pause => {
                        p.pause();
                    }
                    PlaybackCmd::Resume => {
                        p.play();
                        playing_flag.store(true, Ordering::Relaxed);
                    }
                    PlaybackCmd::Stop => {
                        p.stop();
                        playing_flag.store(false, Ordering::Relaxed);
                        break;
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    (tx, is_playing)
}

/// Get voice_id from config, with default fallback.
fn tts_voice_id(config: &TtsConfig) -> String {
    config
        .voice_id
        .clone()
        .unwrap_or_else(|| "default_zh".to_string())
}

/// Get the max text length from a provider for the given config.
/// TODO: query provider dynamically when multiple providers are supported.
fn tts_max_text_length(_config: &TtsConfig) -> usize {
    200 // Xiaomi provider limit
}

fn synthesize_and_play(
    segment: &Segment,
    book_id: &str,
    chapter_index: usize,
    config: &TtsConfig,
    cache: &Arc<TtsCache>,
    playback_tx: &mpsc::Sender<PlaybackCmd>,
    app: &tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::{TtsBuffering, TtsError};
    let emitter = EventEmitter::new(app);

    emitter.tts_buffering(&TtsBuffering {
        book_id: book_id.to_string(),
        chapter_index,
        segment_index: segment.segment_index,
    });

    let voice_id = tts_voice_id(config);
    let request = crate::tts::types::TtsRequest {
        book_id: book_id.to_string(),
        chapter_index,
        segment_index: segment.segment_index,
        paragraph_indices: segment.paragraph_indices.clone(),
        text: segment.text.clone(),
        voice_id: Some(voice_id.clone()),
    };

    // Check cache first
    let segment_path = cache.segment_path(
        &format!("{:?}", config.provider).to_lowercase(),
        book_id,
        chapter_index,
        segment.segment_index,
        &voice_id,
        "pcm16",
    );
    let audio_result = if segment_path.exists() {
        std::fs::read(&segment_path)
            .ok()
            .map(|bytes| (bytes, "audio/pcm16".to_string()))
    } else {
        None
    };

    let (audio_bytes, media_type) = match audio_result {
        Some(r) => r,
        None => {
            // Synthesize on current thread (blocking)
            match TtsSynthesisService::synthesize_blocking(&request, config, &voice_id, cache) {
                Ok(resp) => (resp.audio_bytes, resp.media_type),
                Err(e) => {
                    let msg = format!("TTS 合成失败: {}", e);
                    log::error!("{}", msg);
                    emitter.tts_error(&TtsError {
                        book_id: Some(book_id.to_string()),
                        error_message: msg.clone(),
                    });
                    return Err(msg);
                }
            }
        }
    };

    // Send to playback thread
    playback_tx
        .send(PlaybackCmd::Play(audio_bytes, media_type))
        .map_err(|e| {
            let msg = format!("TTS 播放线程不可用: {}", e);
            emitter.tts_error(&TtsError {
                book_id: Some(book_id.to_string()),
                error_message: msg.clone(),
            });
            msg
        })?;
    Ok(())
}

#[tauri::command]
pub fn tts_test_connection(
    config: TtsConfigDto,
    state: tauri::State<'_, ReaderSession>,
) -> Result<bool, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let full_config = dto_to_tts_config(&config, guard.tts_config.api_key.clone());
    let mut svc = TtsSynthesisService::new(crate::storage::paths::tts_cache_dir());
    svc.register_provider(Box::new(
        crate::tts::xiaomi_provider::XiaomiTtsProvider::new(),
    ));
    match svc.test_connection(&full_config) {
        Ok(()) => Ok(true),
        Err(e) => Err(format!("{}", e)),
    }
}

#[tauri::command]
pub fn tts_start(
    chapter_index: usize,
    state: tauri::State<'_, ReaderSession>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::TtsPlaying;
    let emitter = EventEmitter::new(&app);

    let mut guard = state.lock().map_err(|e| e.to_string())?;

    // Stop any existing playback first
    if let Some(tx) = &guard.playback_tx {
        let _ = tx.send(PlaybackCmd::Stop);
    }
    guard.stop_flag.store(true, Ordering::Relaxed);
    guard.playback_state = Default::default();
    guard.playback_state.current_chapter_index = Some(chapter_index);

    // Reset stop flag for new session
    guard.stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&guard.stop_flag);

    // Get current chapter
    let book = guard.book.as_ref().ok_or("没有打开的书籍")?;
    let book_id = book.id.clone();
    let chapter = book
        .chapters
        .get(chapter_index)
        .ok_or_else(|| format!("章节 {} 不存在", chapter_index))?;
    let paragraphs = chapter.paragraphs.clone();

    if paragraphs.is_empty() {
        return Err("当前章节没有内容".to_string());
    }

    let config = guard.tts_config.clone();
    let cache = Arc::clone(&guard.cache);

    // Segment the chapter
    let max_chars = tts_max_text_length(&config);
    let segments = crate::tts::segmenter::segment_chapter(chapter_index, &paragraphs, max_chars);
    if segments.is_empty() {
        return Err("章节分割结果为空".to_string());
    }

    let total_segments = segments.len();
    let segments_for_poll = segments.clone();
    guard.segments = segments.clone();

    // Spawn playback thread
    let (playback_tx, is_playing_flag) = spawn_playback_thread();
    guard.playback_tx = Some(playback_tx.clone());
    guard.is_playing_flag = Arc::clone(&is_playing_flag);

    // Synthesize and play segment 0
    let segment = segments[0].clone();
    let paragraph_indices = segment.paragraph_indices.clone();

    guard.playback_state.status = PlaybackStatus::Buffering;
    guard.playback_state.current_book_id = Some(book_id.clone());
    guard.playback_state.current_chapter_index = Some(chapter_index);
    guard.playback_state.total_segments = total_segments;
    drop(guard);

    if let Err(e) = synthesize_and_play(
        &segment,
        &book_id,
        chapter_index,
        &config,
        &cache,
        &playback_tx,
        &app,
    ) {
        let mut guard = state.lock().map_err(|lock_err| lock_err.to_string())?;
        guard.stop_flag.store(true, Ordering::Relaxed);
        guard.playback_state.status = PlaybackStatus::Error(e.clone());
        guard.playback_tx = None;
        guard.is_playing_flag.store(false, Ordering::Relaxed);
        return Err(e);
    }

    let mut guard = state.lock().map_err(|e| e.to_string())?;

    // Update playback state
    guard.playback_state.status = PlaybackStatus::Playing;
    guard.playback_state.current_book_id = Some(book_id.clone());
    guard.playback_state.current_chapter_index = Some(chapter_index);
    guard.playback_state.current_segment_index = Some(0);
    guard.playback_state.current_paragraph_indices = paragraph_indices.clone();
    guard.playback_state.total_segments = total_segments;

    emitter.tts_playing(&TtsPlaying {
        book_id: book_id.clone(),
        chapter_index,
        segment_index: 0,
        total_segments,
        paragraph_indices: paragraph_indices.clone(),
    });
    drop(guard);

    // Pre-fetch segment 1
    if segments.len() > 1 {
        let next = segments[1].clone();
        let cfg = config.clone();
        let c = Arc::clone(&cache);
        let bid = book_id.clone();
        let vid = tts_voice_id(&config);
        std::thread::spawn(move || {
            let req = crate::tts::types::TtsRequest {
                book_id: bid,
                chapter_index,
                segment_index: next.segment_index,
                paragraph_indices: next.paragraph_indices,
                text: next.text,
                voice_id: Some(vid.clone()),
            };
            let _ = TtsSynthesisService::synthesize_blocking(&req, &cfg, &vid, &c);
        });
    }

    // Start polling thread for auto-advance
    let poll_playing = Arc::clone(&is_playing_flag);
    let poll_stop = Arc::clone(&stop_flag);
    let poll_tx = playback_tx.clone();
    let poll_app = app.clone();
    let poll_segments = segments_for_poll;
    let poll_config = config.clone();
    let poll_cache = Arc::clone(&cache);
    std::thread::spawn(move || {
        let emitter = EventEmitter::new(&poll_app);
        let mut current_seg_idx: usize = 0;
        let mut was_playing = true;

        loop {
            if poll_stop.load(Ordering::Relaxed) {
                break;
            }

            // Auto-advance when playback finishes
            let currently_playing = poll_playing.load(Ordering::Relaxed);
            if was_playing && !currently_playing && !poll_stop.load(Ordering::Relaxed) {
                was_playing = false;
                current_seg_idx += 1;
                if current_seg_idx >= total_segments {
                    emitter.tts_finished(&crate::tauri_api::events::TtsFinished {
                        book_id: book_id.clone(),
                        chapter_index,
                    });
                    break;
                }
                if let Some(next_seg) = poll_segments.get(current_seg_idx) {
                    if synthesize_and_play(
                        next_seg,
                        &book_id,
                        chapter_index,
                        &poll_config,
                        &poll_cache,
                        &poll_tx,
                        &poll_app,
                    )
                    .is_err()
                    {
                        break;
                    }
                    emitter.tts_playing(&crate::tauri_api::events::TtsPlaying {
                        book_id: book_id.clone(),
                        chapter_index,
                        segment_index: current_seg_idx,
                        total_segments,
                        paragraph_indices: next_seg.paragraph_indices.clone(),
                    });
                    was_playing = true;
                }
            }

            if poll_stop.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });

    Ok(())
}

#[tauri::command]
pub fn tts_pause(
    state: tauri::State<'_, ReaderSession>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::TtsPaused;
    let emitter = EventEmitter::new(&app);
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    if let Some(tx) = &guard.playback_tx {
        tx.send(PlaybackCmd::Pause).ok();
    }
    guard.playback_state.status = PlaybackStatus::Paused;

    let book_id = guard
        .playback_state
        .current_book_id
        .clone()
        .unwrap_or_default();
    let seg_idx = guard.playback_state.current_segment_index.unwrap_or(0);
    emitter.tts_paused(&TtsPaused {
        book_id,
        segment_index: seg_idx,
    });
    Ok(())
}

#[tauri::command]
pub fn tts_resume(
    state: tauri::State<'_, ReaderSession>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::TtsPlaying;
    let emitter = EventEmitter::new(&app);
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    if let Some(tx) = &guard.playback_tx {
        tx.send(PlaybackCmd::Resume).ok();
    }
    guard.playback_state.status = PlaybackStatus::Playing;
    guard.is_playing_flag.store(true, Ordering::Relaxed);

    let book_id = guard
        .playback_state
        .current_book_id
        .clone()
        .unwrap_or_default();
    let chapter_index = guard.playback_state.current_chapter_index.unwrap_or(0);
    let seg_idx = guard.playback_state.current_segment_index.unwrap_or(0);
    let total = guard.playback_state.total_segments;
    let para = guard.playback_state.current_paragraph_indices.clone();
    emitter.tts_playing(&TtsPlaying {
        book_id,
        chapter_index,
        segment_index: seg_idx,
        total_segments: total,
        paragraph_indices: para,
    });
    Ok(())
}

#[tauri::command]
pub fn tts_stop(
    state: tauri::State<'_, ReaderSession>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use crate::tauri_api::emitter::EventEmitter;
    use crate::tauri_api::events::TtsStopped;
    let emitter = EventEmitter::new(&app);
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    guard.stop_flag.store(true, Ordering::Relaxed);
    if let Some(tx) = &guard.playback_tx {
        let _ = tx.send(PlaybackCmd::Stop);
    }
    guard.playback_state = Default::default();
    guard.segments.clear();
    guard.playback_tx = None;
    guard.is_playing_flag.store(false, Ordering::Relaxed);

    emitter.tts_stopped(&TtsStopped {});
    Ok(())
}

#[tauri::command]
pub fn tts_clear_cache(state: tauri::State<'_, ReaderSession>) -> Result<(), String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    guard.cache.clear_all().map_err(|e| e.to_string())
}
