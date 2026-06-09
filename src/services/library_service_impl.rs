use chrono::Utc;

use crate::domain::app_error::{AppError, AppResult};
use crate::domain::book_format::BookFormat;
use crate::domain::error_codes;
use crate::domain::library_item::{FileHealth, LibraryIndex, LibraryItem, ReadingStatsSnapshot};
use crate::parser::ParserFactory;
use crate::services::library_service::LibraryService;
use crate::storage;

pub struct LibraryServiceImpl;

impl LibraryServiceImpl {
    pub fn new() -> Self {
        Self
    }

    // ── Pure operations used by the controller ──────────────

    /// Parse a single book file and produce a LibraryItem for the index.
    pub fn parse_book_item(path: &str, imported_at: &str) -> AppResult<LibraryItem> {
        let format = if path.ends_with(".epub") {
            BookFormat::Epub
        } else {
            BookFormat::Txt
        };

        let parser = ParserFactory::get_parser(path).ok_or_else(|| {
            let mut err = AppError::new(error_codes::UNSUPPORTED_FORMAT, "不支持的文件格式");
            err.recoverable = true;
            err
        })?;

        let result = parser.parse(path).map_err(|e| {
            let mut err = AppError::with_detail(error_codes::FILE_OPEN_FAILED, "解析失败", e);
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
        let book_id = crate::domain::book::stable_book_id(path);
        let file_exists = std::path::Path::new(path).exists();

        Ok(LibraryItem {
            book_id,
            title,
            author,
            format,
            source_path: path.to_string(),
            cover_cache_key: None,
            progress_percent: 0.0,
            last_opened_at: None,
            imported_at: imported_at.to_string(),
            chapter_count,
            file_health: if file_exists {
                FileHealth::Ok
            } else {
                FileHealth::Missing
            },
            stats: ReadingStatsSnapshot::default(),
        })
    }

    /// Repair a book's source path and file health.
    pub fn repair_item_path(index: &mut LibraryIndex, book_id: &str, new_path: &str) {
        if let Some(item) = index.items.iter_mut().find(|i| i.book_id == book_id) {
            item.source_path = new_path.to_string();
            item.file_health = if std::path::Path::new(new_path).exists() {
                FileHealth::Ok
            } else {
                FileHealth::Missing
            };
        }
    }

    /// Save library index to disk.
    pub fn save_index(index: &LibraryIndex) {
        if let Err(e) = storage::library_store::save(index) {
            log::warn!("保存书库索引失败: {}", e);
        }
    }

    /// Load library index from disk.
    pub fn load_index() -> LibraryIndex {
        storage::library_store::load()
    }
}

impl LibraryService for LibraryServiceImpl {
    fn list_books(&self) -> Vec<LibraryItem> {
        let index = Self::load_index();
        index.items
    }

    fn import_books(&self, paths: Vec<String>) -> AppResult<Vec<LibraryItem>> {
        let now = Utc::now().to_rfc3339();
        let mut index = Self::load_index();
        let mut imported = Vec::new();

        for path in &paths {
            match Self::parse_book_item(path, &now) {
                Ok(mut item) => {
                    let book_id = item.book_id.clone();
                    // Cache cover at import time (not on every list call)
                    if item.format == BookFormat::Epub {
                        let epub_path = std::path::Path::new(&item.source_path);
                        if epub_path.exists() {
                            if let Some(cover_path) = crate::services::asset_service_impl::extract_and_cache_cover(
                                epub_path, &book_id,
                            ) {
                                if let Some(ext) = cover_path.extension().and_then(|e| e.to_str()) {
                                    item.cover_cache_key = Some(ext.to_string());
                                }
                            }
                        }
                    }
                    if let Some(existing) = index.items.iter_mut().find(|i| i.book_id == book_id) {
                        let imported_at = existing.imported_at.clone();
                        imported.push(item.clone());
                        *existing = item;
                        existing.imported_at = imported_at;
                    } else {
                        index.items.push(item.clone());
                        imported.push(item);
                    }
                    index.last_selected_book_id = Some(book_id);
                }
                Err(e) => {
                    log::warn!("导入书籍失败: {} - {}", path, e);
                    // Continue with other books; don't abort the batch
                }
            }
        }

        Self::save_index(&index);

        if imported.is_empty() && !paths.is_empty() {
            Err(AppError::new(
                error_codes::FILE_OPEN_FAILED,
                "所有书籍导入失败",
            ))
        } else {
            Ok(imported)
        }
    }

    fn open_book(&self, book_id: &str) -> AppResult<()> {
        // The actual book loading is handled by ReaderService / controller.
        // This just validates the book exists in the index.
        let index = Self::load_index();
        let item = index
            .items
            .iter()
            .find(|i| i.book_id == book_id)
            .ok_or_else(|| AppError::new(error_codes::FILE_OPEN_FAILED, "书籍不在书库中"))?;

        if item.file_health == FileHealth::Missing {
            let mut err = AppError::new(error_codes::FILE_OPEN_FAILED, "书籍文件缺失");
            err.recoverable = true;
            return Err(err);
        }

        Ok(())
    }

    fn remove_book(&self, book_id: &str) -> AppResult<()> {
        let mut index = Self::load_index();
        let before = index.items.len();
        index.items.retain(|i| i.book_id != book_id);
        if index.items.len() == before {
            return Err(AppError::new(
                error_codes::FILE_OPEN_FAILED,
                "书籍不在书库中",
            ));
        }
        Self::save_index(&index);
        // Clean up bookmarks
        let _ = crate::storage::bookmark_store::save(book_id, &[]);
        // Clean up cached assets (best-effort)
        let cover_dir = crate::storage::paths::app_data_dir().join("cache/covers");
        for ext in &["png", "jpg", "jpeg", "webp", "gif", "svg"] {
            let cover_path = cover_dir.join(format!("{}.{}", book_id, ext));
            if cover_path.exists() {
                let _ = std::fs::remove_file(&cover_path);
            }
        }
        let img_dir = crate::storage::paths::app_data_dir()
            .join("cache/images")
            .join(book_id);
        if img_dir.exists() {
            let _ = std::fs::remove_dir_all(&img_dir);
        }
        Ok(())
    }

    fn search(&self, query: &str) -> Vec<LibraryItem> {
        let index = Self::load_index();
        let q = query.to_lowercase();
        index
            .items
            .into_iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&q)
                    || item
                        .author
                        .as_ref()
                        .map_or(false, |a| a.to_lowercase().contains(&q))
            })
            .collect()
    }

    fn repair_path(&self, book_id: &str, new_path: &str) -> AppResult<()> {
        let mut index = Self::load_index();
        if !index.items.iter().any(|i| i.book_id == book_id) {
            return Err(AppError::new(
                error_codes::FILE_OPEN_FAILED,
                "书籍不在书库中",
            ));
        }
        if ParserFactory::get_parser(new_path).is_none() {
            let mut err = AppError::new(error_codes::UNSUPPORTED_FORMAT, "不支持的文件格式");
            err.recoverable = true;
            return Err(err);
        }
        if !std::path::Path::new(new_path).exists() {
            let mut err = AppError::new(error_codes::FILE_OPEN_FAILED, "修复路径不存在");
            err.recoverable = true;
            return Err(err);
        }
        Self::repair_item_path(&mut index, book_id, new_path);
        Self::save_index(&index);
        Ok(())
    }
}
