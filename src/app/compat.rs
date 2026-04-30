use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::Utc;
use log::{info, warn};

use crate::app::Action;
use crate::domain::app_error::{AppError, AppResult};
use crate::domain::app_state::AppState;
use crate::domain::book::Book;
use crate::domain::book_assets::BookAssets;
use crate::domain::book_format::BookFormat;
use crate::domain::book_load_info::BookLoadInfo;
use crate::domain::book_metadata::BookMetadata;
use crate::domain::chapter::Chapter;
use crate::domain::error_codes;
use crate::domain::paragraph::Paragraph;
use crate::domain::paragraph_kind::ParagraphKind;
use crate::domain::toc_item::TocItem;
use crate::parser::ParserFactory;
use crate::storage;

pub struct CompatAdapter {
    state: AppState,
}

impl CompatAdapter {
    pub fn new() -> Self {
        let _ = storage::paths::ensure_dirs();

        let settings_file = storage::settings_store::load();
        let recent_books = storage::recent_store::load();

        let mut state = AppState::default();
        state.reader_settings = settings_file.reader_settings;
        state.recent_books = recent_books;

        Self { state }
    }

    pub(crate) fn try_load_book(&self, path: &str) -> AppResult<Book> {
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
                build_chapter(index, &title, text)
            })
            .collect::<Vec<_>>();

        let toc = if result.chapter_titles.is_empty() {
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
        let file_size = std::fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0);
        let source_path = std::path::PathBuf::from(path);
        let file_stem = source_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("未命名书籍")
            .to_string();
        let parser_name = match format {
            BookFormat::Epub => "EpubParser",
            BookFormat::Txt => "TxtParser",
            BookFormat::ReservedPdf => "ReservedPdf",
            BookFormat::ReservedMobi => "ReservedMobi",
        };

        Ok(Book {
            id: stable_book_id(path),
            source_path,
            format: format.clone(),
            metadata: BookMetadata {
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
            },
            toc,
            chapters: chapters.clone(),
            assets: BookAssets {
                cover_image_bytes: None,
                cover_media_type: None,
                has_images: false,
                embedded_styles_detected: matches!(format, BookFormat::Epub),
            },
            load_info: BookLoadInfo {
                parser_name: parser_name.to_string(),
                parse_warnings: Vec::new(),
                chapter_count: chapters.len(),
                loaded_at: Utc::now().to_rfc3339(),
                source_file_size: file_size,
                load_duration_ms: duration_ms,
            },
        })
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub(crate) fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    pub fn dispatch(&mut self, action: Action) {
        crate::app::controller::dispatch(self, action);
    }

    pub fn save_persisted_state(&self) {
        let state = &self.state;

        let settings_file = storage::settings_store::SettingsFile::from_reader_settings(
            &state.reader_settings,
            state.current_book.as_ref().map(|b| b.id.clone()),
        );
        if let Err(e) = storage::settings_store::save(&settings_file) {
            warn!("保存设置失败: {}", e);
        }

        if let Err(e) = storage::recent_store::save(&state.recent_books) {
            warn!("保存最近阅读失败: {}", e);
        }

        if let (Some(book), Some(progress)) = (&state.current_book, &state.reading_progress) {
            if let Err(e) = storage::progress_store::save(&book.id, progress) {
                warn!("保存阅读进度失败: {}", e);
            }
        }

        if let Some(book) = &state.current_book {
            if let Err(e) = storage::bookmark_store::save(&book.id, &state.bookmarks) {
                warn!("保存书签失败: {}", e);
            }
        }
    }
}

fn build_chapter(index: usize, title: &str, text: &str) -> Chapter {
    let paragraphs = text
        .split("\n\n")
        .enumerate()
        .filter_map(|(paragraph_index, raw)| {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }

            Some(Paragraph {
                index: paragraph_index,
                text: trimmed.to_string(),
                kind: infer_paragraph_kind(trimmed),
                indent_level: 0,
                source_line_hint: None,
            })
        })
        .collect::<Vec<_>>();

    let content = paragraphs
        .iter()
        .map(|paragraph| paragraph.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Chapter {
        id: format!("ch-{}", index),
        index,
        title: title.to_string(),
        raw_title: Some(title.to_string()),
        word_count: content.split_whitespace().count(),
        char_count: content.chars().count(),
        content,
        paragraphs,
        source_href: None,
        anchor: None,
        warnings: Vec::new(),
    }
}

fn infer_paragraph_kind(text: &str) -> ParagraphKind {
    let is_cn_title = text.starts_with('第')
        && (text.contains('章') || text.contains('节') || text.contains('卷'))
        && text.chars().count() < 50;
    let is_en_title = text.starts_with("Chapter") && text.chars().count() < 50;

    if is_cn_title || is_en_title {
        ParagraphKind::Title
    } else {
        ParagraphKind::Body
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::ScreenKind;
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::ZipWriter;
    use zip::write::FileOptions;

    fn temp_path(ext: &str) -> std::path::PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        std::env::temp_dir().join(format!("reader-demo-test-{}.{ext}", millis))
    }

    fn create_txt_fixture() -> std::path::PathBuf {
        let path = temp_path("txt");
        fs::write(&path, "第1章 起始\n\n第一段内容。\n\n第二段内容。").unwrap();
        path
    }

    fn create_epub_fixture() -> std::path::PathBuf {
        let path = temp_path("epub");
        let file = fs::File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .unwrap();

        zip.start_file("content.opf", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="chap1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chap1"/>
  </spine>
</package>"#,
        )
        .unwrap();

        zip.start_file("chapter1.xhtml", options).unwrap();
        zip.write_all(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <body>
    <h1>第1章 开始</h1>
    <p>第一段。</p>
  </body>
</html>"#
                .as_bytes(),
        )
        .unwrap();

        zip.finish().unwrap();
        path
    }

    #[test]
    fn unsupported_format_sets_error_state() {
        let mut adapter = CompatAdapter::new();
        adapter.dispatch(Action::OpenBookSelected("unsupported.xyz".to_string()));

        assert_eq!(adapter.state().ui_state.screen, ScreenKind::Error);
        assert_eq!(
            adapter.state().last_error.as_ref().map(|err| err.code.as_str()),
            Some(error_codes::UNSUPPORTED_FORMAT)
        );
    }

    #[test]
    fn txt_open_uses_stable_id_and_recent_format() {
        let path = create_txt_fixture();
        let path_str = path.to_string_lossy().to_string();
        let mut adapter = CompatAdapter::new();

        adapter.dispatch(Action::OpenBookSelected(path_str.clone()));
        let first_id = adapter
            .state()
            .current_book
            .as_ref()
            .map(|book| book.id.clone())
            .unwrap();
        assert_eq!(adapter.state().recent_books[0].format, "txt");
        assert!(adapter.state().current_book.as_ref().unwrap().chapters[0].char_count > 0);

        adapter.dispatch(Action::OpenBookSelected(path_str));
        let second_id = adapter
            .state()
            .current_book
            .as_ref()
            .map(|book| book.id.clone())
            .unwrap();
        assert_eq!(first_id, second_id);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn epub_open_uses_epub_recent_format() {
        let path = create_epub_fixture();
        let mut adapter = CompatAdapter::new();

        adapter.dispatch(Action::OpenBookSelected(path.to_string_lossy().to_string()));

        assert_eq!(adapter.state().recent_books[0].format, "epub");
        assert_eq!(adapter.state().ui_state.screen, crate::domain::enums::ScreenKind::Reader);

        let _ = fs::remove_file(path);
    }
}
