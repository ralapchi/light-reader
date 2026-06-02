use chrono::Utc;
use log::info;

use crate::domain::app_error::{AppError, AppResult};
use crate::domain::book::Book;
use crate::domain::book_assets::BookAssets;
use crate::domain::book_format::BookFormat;
use crate::domain::book_load_info::BookLoadInfo;
use crate::domain::book_metadata::BookMetadata;
use crate::domain::chapter_builder::*;
use crate::domain::error_codes;
use crate::domain::paragraph::TextLink;
use crate::domain::reading_progress::ReadingProgress;
use crate::domain::toc_item::TocItem;
use crate::parser::ParserFactory;
use crate::storage;

pub struct ReaderServiceImpl;

impl ReaderServiceImpl {
    /// Parse a book file and construct a full Book object.
    /// This is the core "open book" operation extracted from CompatAdapter::try_load_book.
    pub fn load_book(path: &str) -> AppResult<Book> {
        info!("正在解析文件: {}", path);
        let start = std::time::Instant::now();

        let parser = ParserFactory::get_parser(path).ok_or_else(|| {
            let mut err = AppError::new(error_codes::UNSUPPORTED_FORMAT, "不支持的文件格式");
            err.recoverable = true;
            err
        })?;

        let result = parser.parse(path).map_err(|err| {
            let mut app_error =
                AppError::with_detail(error_codes::FILE_OPEN_FAILED, "解析失败", err);
            app_error.recoverable = true;
            app_error
        })?;

        let format = if path.ends_with(".epub") {
            BookFormat::Epub
        } else {
            BookFormat::Txt
        };

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
                build_chapter(index, &title, text, &img_blocks, href, links, anchors)
            })
            .collect::<Vec<_>>();

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

        Ok(Book {
            id: crate::domain::book::stable_book_id(path),
            source_path,
            format: format.clone(),
            metadata,
            toc,
            chapters: chapters.clone(),
            assets: BookAssets {
                cover_image_bytes: result.cover_image,
                cover_media_type: result.cover_media_type,
                has_images: !result.image_assets.is_empty(),
                embedded_styles_detected: matches!(format, BookFormat::Epub),
                image_assets: result.image_assets,
            },
            load_info: BookLoadInfo {
                parser_name: parser_name.to_string(),
                parse_warnings: result.warnings,
                chapter_count: chapters.len(),
                loaded_at: Utc::now().to_rfc3339(),
                source_file_size: file_size,
                load_duration_ms: duration_ms,
            },
        })
    }

    /// Save reading progress to disk.
    pub fn persist_progress(
        book_id: &str,
        progress: &ReadingProgress,
        session_started_at: Option<&str>,
        total_at_start: u64,
    ) {
        if let Some(started_at) = session_started_at {
            if let Ok(start) = chrono::DateTime::parse_from_rfc3339(started_at) {
                let elapsed = Utc::now().signed_duration_since(start).num_seconds().max(0) as u64;
                let mut progress = progress.clone();
                progress.session_read_seconds = elapsed;
                progress.total_read_seconds = total_at_start + elapsed;
                let _ = storage::progress_store::save(book_id, &progress);
                return;
            }
        }
        let _ = storage::progress_store::save(book_id, progress);
    }
}
