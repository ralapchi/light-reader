use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use chrono::Utc;

use crate::app::Action;
use crate::app::compat::CompatAdapter;
use crate::app::reducer;
use crate::domain::enums::ScreenKind;
use crate::domain::library_item::{FileHealth, LibraryItem, ReadingStatsSnapshot};
use crate::parser::ParserFactory;
use crate::storage;

pub fn dispatch(adapter: &mut CompatAdapter, action: Action) {
    match action {
        Action::OpenBookSelected(path) => {
            {
                let state = adapter.state_mut();
                state.status_message = format!("正在打开文件: {}", path);
                state.status_message_set_at = Some(Utc::now().to_rfc3339());
                state.last_error = None;
                state.ui_state.is_loading = true;
                state.ui_state.screen = ScreenKind::LoadingBook;
                state.ui_state.pending_open_path = Some(PathBuf::from(&path));
                state.ui_state.last_attempted_path = Some(PathBuf::from(&path));
            }

            match adapter.try_load_book(&path) {
                Ok(book) => {
                    reducer::reduce(adapter.state_mut(), Action::OpenBookSucceeded(book));
                    after_book_opened(adapter);
                }
                Err(err) => reducer::reduce(adapter.state_mut(), Action::OpenBookFailed(err)),
            }
        }
        Action::GoToChapter(_)
        | Action::NextChapter
        | Action::PrevChapter
        | Action::JumpToBookmark(_) => {
            reducer::reduce(adapter.state_mut(), action);
            save_progress(adapter);
        }
        Action::AddBookmarkRequested | Action::RemoveBookmark(_) => {
            reducer::reduce(adapter.state_mut(), action);
            save_bookmarks(adapter);
        }
        Action::RecentBookSelected(_) => {
            reducer::reduce(adapter.state_mut(), action);
            let path = adapter
                .state()
                .ui_state
                .pending_open_path
                .as_ref()
                .filter(|p| p.exists())
                .cloned();
            adapter.state_mut().ui_state.pending_open_path = None;
            if let Some(path) = path {
                if let Some(path_str) = path.to_str() {
                    dispatch(adapter, Action::OpenBookSelected(path_str.to_string()));
                }
            }
        }
        Action::RemoveRecentBook(_) => {
            reducer::reduce(adapter.state_mut(), action);
            save_recent(adapter);
        }
        Action::ClearMissingRecentBooks => {
            reducer::reduce(adapter.state_mut(), action);
            save_recent(adapter);
        }
        Action::CloseBook => {
            save_progress(adapter);
            save_bookmarks(adapter);
            // Update library index before CloseBook clears the state
            let close_data = adapter.state().current_book.as_ref().map(|book| {
                let progress_pct = adapter
                    .state()
                    .reading_progress
                    .as_ref()
                    .map(|p| p.progress_percent)
                    .unwrap_or(0.0);
                (book.id.clone(), progress_pct)
            });
            reducer::reduce(adapter.state_mut(), action);
            if let Some((book_id, progress_pct)) = close_data {
                let state = adapter.state_mut();
                if let Some(item) = state
                    .library_index
                    .items
                    .iter_mut()
                    .find(|i| i.book_id == *book_id)
                {
                    item.progress_percent = progress_pct;
                }
                save_library_index(adapter);
            }
        }
        Action::OpenLibraryHome => {
            // Load library index from storage
            let library_index = storage::library_store::load();
            adapter.state_mut().library_index = library_index;
            reducer::reduce(adapter.state_mut(), action);
        }
        Action::ImportBooksSelected(paths) => {
            let now = Utc::now().to_rfc3339();
            let mut imported_ids: Vec<String> = Vec::new();
            for path in &paths {
                match import_single_book(path, &now) {
                    Ok(item) => {
                        let book_id = item.book_id.clone();
                        {
                            let state = adapter.state_mut();
                            if let Some(existing) =
                                state.library_index.items.iter_mut().find(|i| i.book_id == book_id)
                            {
                                // Update existing item (re-import)
                                let imported_at = existing.imported_at.clone();
                                *existing = item;
                                existing.imported_at = imported_at;
                            } else {
                                state.library_index.items.push(item);
                            }
                            state.library_index.last_selected_book_id = Some(book_id.clone());
                        }
                        imported_ids.push(book_id);
                    }
                    Err(err) => {
                        log::warn!("导入书籍失败: {} - {}", path, err);
                        reducer::reduce(
                            adapter.state_mut(),
                            Action::ImportBookFailed(path.clone(), err),
                        );
                    }
                }
            }
            save_library_index(adapter);
            for book_id in imported_ids {
                let found = adapter
                    .state()
                    .library_index
                    .items
                    .iter()
                    .find(|i| i.book_id == *book_id)
                    .cloned()
                    .unwrap();
                reducer::reduce(adapter.state_mut(), Action::ImportBookSucceeded(found));
            }
        }
        Action::LibraryBookSelected(_) => {
            let book_id = {
                reducer::reduce(adapter.state_mut(), action);
                adapter.state().library_view_state.selected_book_id.clone()
            };
            // Set loading context from library item
            if let Some(ref book_id) = book_id {
                let ctx = adapter.state().library_index.items.iter()
                    .find(|i| i.book_id == *book_id)
                    .map(|item| (item.title.clone(), item.author.clone(), item.cover_cache_key.clone()));
                if let Some((title, author, cover_key)) = ctx {
                    let state = adapter.state_mut();
                    state.ui_state.loading_book_title = Some(title);
                    state.ui_state.loading_book_author = author;
                    state.ui_state.loading_book_cover_key = cover_key;
                }
            }
            let path = adapter
                .state()
                .ui_state
                .pending_open_path
                .as_ref()
                .filter(|p| p.exists())
                .cloned();
            adapter.state_mut().ui_state.pending_open_path = None;
            if let Some(path) = path {
                if let Some(path_str) = path.to_str() {
                    dispatch(adapter, Action::OpenBookSelected(path_str.to_string()));
                }
            }
        }
        Action::RemoveFromLibrary(_) => {
            reducer::reduce(adapter.state_mut(), action);
            save_library_index(adapter);
        }
        Action::RefreshLibraryItem(_) => {
            reducer::reduce(adapter.state_mut(), action);
            save_library_index(adapter);
        }
        Action::RescanMissingBooks => {
            reducer::reduce(adapter.state_mut(), action);
            save_library_index(adapter);
        }
        Action::RepairLibraryPath { ref book_id, ref new_path } => {
            {
                let state = adapter.state_mut();
                if let Some(item) = state
                    .library_index
                    .items
                    .iter_mut()
                    .find(|i| i.book_id == *book_id)
                {
                    item.source_path = new_path.clone();
                    item.file_health = if std::path::Path::new(&new_path).exists() {
                        FileHealth::Ok
                    } else {
                        FileHealth::Missing
                    };
                }
            }
            save_library_index(adapter);
            reducer::reduce(adapter.state_mut(), action);
        }
        Action::ImportBookSucceeded(_) | Action::ImportBookFailed(_, _) => {
            reducer::reduce(adapter.state_mut(), action);
        }
        Action::ThemeChanged(_)
        | Action::ReaderSettingChanged(_)
        | Action::UpdateReaderSetting(_)
        | Action::RestoreDefaultSettings => {
            reducer::reduce(adapter.state_mut(), action);
            save_settings(adapter);
        }
        other => reducer::reduce(adapter.state_mut(), other),
    }
}

fn after_book_opened(adapter: &mut CompatAdapter) {
    let state = adapter.state();
    let book_id = match &state.current_book {
        Some(book) => book.id.clone(),
        None => return,
    };
    let chapter_count = state.current_book.as_ref().map(|b| b.chapters.len()).unwrap_or(0);

    if let Some(progress) = storage::progress_store::load(&book_id) {
        adapter.state_mut().total_read_seconds_at_session_start = progress.total_read_seconds;
        adapter.state_mut().reading_progress = Some(progress);
    } else {
        // T13: 进度文件损坏，回退到章节开头并提示用户
        if chapter_count > 0 {
            let state = adapter.state_mut();
            let is_initial_message = state.status_message_set_at.is_none();
            if state.status_message.is_empty() || is_initial_message {
                state.status_message = "上次阅读进度无法恢复，已从开头开始阅读".to_string();
                state.status_message_set_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }
    }

    let bookmarks = storage::bookmark_store::load(&book_id);
    if !bookmarks.is_empty() {
        adapter.state_mut().bookmarks = bookmarks;
    }

    // Update library index and save cover to cache
    let library_info = {
        let state = adapter.state();
        state.current_book.as_ref().map(|book| {
            let progress_pct = state.reading_progress.as_ref()
                .map(|p| p.progress_percent).unwrap_or(0.0);
            (
                book.id.clone(),
                book.metadata.title.clone(),
                book.metadata.author.clone(),
                book.format.clone(),
                book.source_path.to_string_lossy().to_string(),
                book.chapters.len(),
                progress_pct,
                book.assets.cover_image_bytes.clone(),
                book.assets.cover_media_type.clone(),
                book.assets.image_assets.clone(),
            )
        })
    };

    if let Some((book_id, title, author, format, source_path, chapter_count, progress_pct, cover_bytes, cover_mime, image_assets)) = library_info {
        // Save cover to cache with real extension
        let cover_key: Option<String> = cover_bytes.and_then(|bytes| {
            let ext = media_type_to_ext(cover_mime.as_deref());
            let cache_path = storage::paths::cover_cache_path(&book_id, ext);
            let _ = std::fs::create_dir_all(cache_path.parent()?);
            std::fs::write(&cache_path, &bytes).ok()?;
            Some(format!("{}.{}", book_id, ext))
        });

        // Save image assets to cache
        for img_asset in &image_assets {
            if let Some(cache_key) = &img_asset.cache_key {
                // The actual bytes were extracted during parsing and written by the parser
                // Here we just need to ensure the cache keys are set on library items
                let _ = cache_key;
            }
        }

        upsert_library_item(
            adapter, &book_id, title, author, format,
            source_path, chapter_count, progress_pct,
        );

        // Set cover_cache_key if cover was saved
        if let Some(ref key) = cover_key {
            let state = adapter.state_mut();
            if let Some(item) = state.library_index.items.iter_mut().find(|i| i.book_id == book_id) {
                item.cover_cache_key = Some(key.clone());
            }
        }
    }
    save_library_index(adapter);

    save_recent(adapter);
}

fn save_progress(adapter: &mut CompatAdapter) {
    let (book_id, progress_opt, session_start, total_at_start) = {
        let state = adapter.state();
        (
            state.current_book.as_ref().map(|b| b.id.clone()),
            state.reading_progress.clone(),
            state.session_started_at.clone(),
            state.total_read_seconds_at_session_start,
        )
    };

    if let (Some(book_id), Some(progress)) = (book_id, progress_opt) {
        if let Some(ref started_at) = session_start {
            if let Ok(start) = chrono::DateTime::parse_from_rfc3339(started_at) {
                let elapsed = Utc::now().signed_duration_since(start).num_seconds().max(0) as u64;
                let mut progress = progress;
                progress.session_read_seconds = elapsed;
                progress.total_read_seconds = total_at_start + elapsed;
                let _ = storage::progress_store::save(&book_id, &progress);
                sync_library_stats(adapter, elapsed, progress.chapter_index);
                return;
            }
        }
        let chapter_idx = progress.chapter_index;
        let _ = storage::progress_store::save(&book_id, &progress);
        sync_library_stats(adapter, 0, chapter_idx);
    }
}

fn save_bookmarks(adapter: &mut CompatAdapter) {
    let state = adapter.state();
    if let Some(book) = &state.current_book {
        if let Err(e) = storage::bookmark_store::save(&book.id, &state.bookmarks) {
            log::warn!("保存书签失败: {}", e);
        }
        // Sync bookmark count to library index
        let bookmark_count = state.bookmarks.len();
        let book_id = book.id.clone();
        let _ = state;
        let state_mut = adapter.state_mut();
        if let Some(item) = state_mut.library_index.items.iter_mut().find(|i| i.book_id == book_id) {
            item.stats.bookmark_count = bookmark_count;
        }
    }
}

/// Sync reading stats from current session to library index item.
fn sync_library_stats(adapter: &mut CompatAdapter, elapsed_seconds: u64, chapter_index: usize) {
    let state = adapter.state();
    let book_id = match &state.current_book {
        Some(book) => book.id.clone(),
        None => return,
    };
    let total_seconds = state.total_read_seconds_at_session_start + elapsed_seconds;
    let _ = state;
    let state_mut = adapter.state_mut();
    if let Some(item) = state_mut.library_index.items.iter_mut().find(|i| i.book_id == book_id) {
        item.stats.total_read_seconds = total_seconds;
        item.stats.last_chapter_index = Some(chapter_index);
        item.stats.last_read_at = Some(Utc::now().to_rfc3339());
    }
}

fn save_recent(adapter: &CompatAdapter) {
    let state = adapter.state();
    if let Err(e) = storage::recent_store::save(&state.recent_books) {
        log::warn!("保存最近阅读失败: {}", e);
    }
}

fn save_settings(adapter: &CompatAdapter) {
    let state = adapter.state();
    let settings_file = storage::settings_store::SettingsFile::from_reader_settings(
        &state.reader_settings,
        state.current_book.as_ref().map(|b| b.id.clone()),
    );
    if let Err(e) = storage::settings_store::save(&settings_file) {
        log::warn!("保存设置失败: {}", e);
    }
}

fn save_library_index(adapter: &CompatAdapter) {
    if let Err(e) = storage::library_store::save(&adapter.state().library_index) {
        log::warn!("保存书库索引失败: {}", e);
    }
}

/// Shared helper: construct or update a LibraryItem in the index from a Book.
/// Used by both `after_book_opened` (full Book available) and `import_single_book` (parser result).
fn upsert_library_item(
    adapter: &mut CompatAdapter,
    book_id: &str,
    title: String,
    author: Option<String>,
    format: crate::domain::book_format::BookFormat,
    source_path: String,
    chapter_count: usize,
    progress_percent: f32,
) {
    let now = Utc::now().to_rfc3339();
    let file_health = if std::path::Path::new(&source_path).exists() {
        FileHealth::Ok
    } else {
        FileHealth::Missing
    };

    let state = adapter.state_mut();
    if let Some(existing) = state.library_index.items.iter_mut().find(|i| i.book_id == book_id) {
        existing.title = title;
        existing.author = author;
        existing.format = format;
        existing.source_path = source_path;
        existing.chapter_count = chapter_count;
        existing.progress_percent = progress_percent;
        existing.last_opened_at = Some(now.clone());
        existing.file_health = file_health;
        existing.stats.last_read_at = Some(now);
        existing.stats.last_chapter_index = state
            .reading_progress
            .as_ref()
            .map(|p| p.chapter_index);
    } else {
        state.library_index.items.push(LibraryItem {
            book_id: book_id.to_string(),
            title,
            author,
            format,
            source_path,
            cover_cache_key: None,
            progress_percent,
            last_opened_at: Some(now.clone()),
            imported_at: now.clone(),
            chapter_count,
            file_health,
            stats: ReadingStatsSnapshot {
                last_read_at: Some(now),
                last_chapter_index: state
                    .reading_progress
                    .as_ref()
                    .map(|p| p.chapter_index),
                ..Default::default()
            },
        });
    }
}

fn media_type_to_ext(mime: Option<&str>) -> &'static str {
    match mime {
        Some("image/jpeg") => "jpg",
        Some("image/png") => "png",
        Some("image/webp") => "webp",
        Some("image/gif") => "gif",
        Some("image/svg+xml") => "svg",
        _ => "png",
    }
}

fn stable_book_id(path: &str) -> String {
    let normalized = std::fs::canonicalize(path)
        .ok()
        .and_then(|resolved| resolved.to_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| path.to_string());
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    format!("book-{:016x}", hasher.finish())
}

fn import_single_book(path: &str, now: &str) -> Result<LibraryItem, crate::domain::app_error::AppError> {
    let format = if path.ends_with(".epub") {
        crate::domain::book_format::BookFormat::Epub
    } else {
        crate::domain::book_format::BookFormat::Txt
    };

    let parser = ParserFactory::get_parser(path).ok_or_else(|| {
        let mut err = crate::domain::app_error::AppError::new(
            crate::domain::error_codes::UNSUPPORTED_FORMAT,
            "不支持的文件格式",
        );
        err.recoverable = true;
        err
    })?;

    let result = parser.parse(path).map_err(|e| {
        let mut err = crate::domain::app_error::AppError::with_detail(
            crate::domain::error_codes::FILE_OPEN_FAILED,
            "解析失败",
            e,
        );
        err.recoverable = true;
        err
    })?;

    let file_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("未命名书籍")
        .to_string();

    let title = result
        .metadata
        .as_ref()
        .map(|m| m.title.clone())
        .unwrap_or(file_stem);
    let author = result.metadata.and_then(|m| m.author);
    let chapter_count = result.content.len();
    let book_id = stable_book_id(path);
    let file_exists = std::path::Path::new(path).exists();

    Ok(LibraryItem {
        book_id,
        title,
        author,
        format,
        source_path: path.to_string(),
        cover_cache_key: None,
        progress_percent: if chapter_count > 0 {
            1.0 / chapter_count as f32
        } else {
            0.0
        },
        last_opened_at: None,
        imported_at: now.to_string(),
        chapter_count,
        file_health: if file_exists {
            FileHealth::Ok
        } else {
            FileHealth::Missing
        },
        stats: ReadingStatsSnapshot::default(),
    })
}
