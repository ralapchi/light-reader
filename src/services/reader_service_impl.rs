use chrono::Utc;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Instant, UNIX_EPOCH};

use crate::domain::app_error::{AppError, AppResult};
use crate::domain::book::Book;
use crate::domain::book_assets::BookAssets;
use crate::domain::book_format::BookFormat;
use crate::domain::book_load_info::BookLoadInfo;
use crate::domain::book_metadata::BookMetadata;
use crate::domain::chapter_builder::*;
use crate::domain::error_codes;
use crate::domain::paragraph::TextLink;
use crate::domain::toc_item::TocItem;
use crate::parser::ParserFactory;

pub struct ReaderServiceImpl;

const BOOK_CACHE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct CachedBook {
    version: u32,
    source_path: PathBuf,
    source_file_size: u64,
    source_modified_nanos: String,
    book: Book,
}

impl ReaderServiceImpl {
    /// Parse a book file and construct a full Book object.
    /// This is the core "open book" operation extracted from CompatAdapter::try_load_book.
    pub fn load_book(path: &str) -> AppResult<Book> {
        info!("正在打开文件: {}", path);
        let start = std::time::Instant::now();
        let book_id = crate::domain::book::stable_book_id(path);

        if let Some(mut book) = Self::load_cached_book(&book_id, path, start) {
            book.load_info.loaded_at = Utc::now().to_rfc3339();
            book.load_info.load_duration_ms = start.elapsed().as_millis() as u64;
            return Ok(book);
        }

        let parser = ParserFactory::get_parser(path).ok_or_else(|| {
            let mut err = AppError::new(error_codes::UNSUPPORTED_FORMAT, "不支持的文件格式");
            err.recoverable = true;
            err
        })?;

        let parse_start = Instant::now();
        let result = parser.parse(path)?;
        info!(
            "解析源文件完成: book={}, elapsed={}ms",
            book_id,
            parse_start.elapsed().as_millis()
        );

        let format = BookFormat::from_path(path).unwrap_or(BookFormat::Txt);

        let chapter_start = Instant::now();
        let chapters = result
            .content
            .iter()
            .enumerate()
            .map(|(index, text)| {
                let title = result
                    .chapter_titles
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| format!("章节 {}", index + 1));
                let img_blocks = result
                    .chapter_image_blocks
                    .get(index)
                    .cloned()
                    .unwrap_or_default();
                let href = result.spine_hrefs.get(index).map(|s| s.as_str());
                let links: &[Vec<TextLink>] = result
                    .chapter_links
                    .get(index)
                    .map(|l| l.as_slice())
                    .unwrap_or(&[]);
                let anchors = result
                    .chapter_anchors
                    .get(index)
                    .cloned()
                    .unwrap_or_default();
                let heading_flags = result
                    .chapter_heading_flags
                    .get(index)
                    .map(|f| f.as_slice())
                    .unwrap_or(&[]);
                build_chapter(
                    index,
                    &title,
                    text,
                    &img_blocks,
                    href,
                    links,
                    anchors,
                    heading_flags,
                )
            })
            .collect::<Vec<_>>();
        info!(
            "构建章节完成: book={}, chapters={}, elapsed={}ms",
            book_id,
            chapters.len(),
            chapter_start.elapsed().as_millis()
        );

        let toc_start = Instant::now();
        let toc = if let Some(structured_toc) = result.toc {
            let href_to_index = build_href_index(&result.spine_hrefs);
            map_toc_chapter_indices(structured_toc, &href_to_index)
        } else if result.chapter_titles.is_empty() {
            chapters
                .iter()
                .enumerate()
                .map(|(index, chapter)| TocItem {
                    id: format!("toc-{}", index),
                    title: chapter.title.clone(),
                    chapter_index: Some(index),
                    href: None,
                    depth: 0,
                    children: Vec::new(),
                    is_generated: true,
                })
                .collect()
        } else {
            result
                .chapter_titles
                .iter()
                .enumerate()
                .map(|(index, title)| TocItem {
                    id: format!("toc-{}", index),
                    title: title.clone(),
                    chapter_index: Some(index),
                    href: None,
                    depth: 0,
                    children: Vec::new(),
                    is_generated: true,
                })
                .collect()
        };
        info!(
            "构建目录完成: book={}, toc_items={}, elapsed={}ms",
            book_id,
            toc.len(),
            toc_start.elapsed().as_millis()
        );

        let duration_ms = start.elapsed().as_millis() as u64;
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let source_path = std::path::PathBuf::from(path);
        let file_stem = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("未命名书籍")
            .to_string();
        let parser_name = match format {
            BookFormat::Epub => "EpubParser",
            BookFormat::Txt => "TxtParser",
        };

        let metadata = result.metadata.unwrap_or(BookMetadata {
            title: file_stem,
            author: None,
            language: None,
            publisher: None,
            description: None,
            identifier: None,
            series: None,
            cover_title: None,
            created_at: None,
            modified_at: None,
        });

        let chapter_count = chapters.len();
        let embedded_styles_detected = matches!(format, BookFormat::Epub);

        let book = Book {
            id: book_id.clone(),
            source_path,
            format,
            metadata,
            toc,
            chapters,
            assets: BookAssets {
                cover_image_bytes: result.cover_image,
                cover_media_type: result.cover_media_type,
                has_images: !result.image_assets.is_empty(),
                embedded_styles_detected,
                image_assets: result.image_assets,
            },
            load_info: BookLoadInfo {
                parser_name: parser_name.to_string(),
                parse_warnings: result.warnings,
                chapter_count,
                loaded_at: Utc::now().to_rfc3339(),
                source_file_size: file_size,
                load_duration_ms: duration_ms,
            },
        };

        Self::save_cached_book(path, &book);

        Ok(book)
    }

    fn load_cached_book(book_id: &str, path: &str, start: Instant) -> Option<Book> {
        let (source_file_size, source_modified_nanos) = source_fingerprint(path)?;
        let cache_path = crate::storage::paths::book_cache_path(book_id);
        let bytes = std::fs::read(&cache_path).ok()?;
        let cached = serde_json::from_slice::<CachedBook>(&bytes).ok()?;

        if cached.version != BOOK_CACHE_VERSION
            || cached.source_file_size != source_file_size
            || cached.source_modified_nanos != source_modified_nanos
            || !same_source_path(&cached.source_path, path)
            || cached.book.id != book_id
        {
            return None;
        }

        info!(
            "命中书籍缓存: book={}, path={}, elapsed={}ms",
            book_id,
            cache_path.display(),
            start.elapsed().as_millis()
        );
        Some(cached.book)
    }

    fn save_cached_book(path: &str, book: &Book) {
        let Some((source_file_size, source_modified_nanos)) = source_fingerprint(path) else {
            return;
        };

        let cache = CachedBook {
            version: BOOK_CACHE_VERSION,
            source_path: book.source_path.clone(),
            source_file_size,
            source_modified_nanos,
            book: book.clone(),
        };

        let cache_path = crate::storage::paths::book_cache_path(&book.id);
        if let Some(parent) = cache_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!("创建书籍缓存目录失败: {}", e);
                return;
            }
        }

        match serde_json::to_vec(&cache)
            .map_err(|e| e.to_string())
            .and_then(|bytes| std::fs::write(&cache_path, bytes).map_err(|e| e.to_string()))
        {
            Ok(()) => info!(
                "写入书籍缓存: book={}, path={}",
                book.id,
                cache_path.display()
            ),
            Err(e) => warn!("写入书籍缓存失败: book={}, error={}", book.id, e),
        }
    }
}

fn source_fingerprint(path: &str) -> Option<(u64, String)> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let modified_nanos = modified.duration_since(UNIX_EPOCH).ok()?.as_nanos().to_string();
    Some((metadata.len(), modified_nanos))
}

fn same_source_path(cached_path: &Path, path: &str) -> bool {
    let current = Path::new(path);
    match (
        std::fs::canonicalize(cached_path),
        std::fs::canonicalize(current),
    ) {
        (Ok(a), Ok(b)) => a == b,
        _ => cached_path == current,
    }
}
