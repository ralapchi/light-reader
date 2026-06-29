use crate::domain::app_error::{AppError, AppResult};
use crate::domain::book_format::BookFormat;
use crate::domain::error_codes;
use crate::domain::library_item::{FileHealth, LibraryIndex, LibraryItem, ReadingStatsSnapshot};
use crate::parser::ParserFactory;

pub struct LibraryServiceImpl;

impl LibraryServiceImpl {
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
}
