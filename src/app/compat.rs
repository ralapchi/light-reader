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

        // Mark missing files in recent books
        for item in &mut state.recent_books {
            item.is_missing = !std::path::Path::new(&item.source_path).exists();
        }

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

        let toc = if let Some(structured_toc) = result.toc {
            // 将 TOC href 映射到 chapter_index
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
            id: stable_book_id(path),
            source_path,
            format: format.clone(),
            metadata,
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
                parse_warnings: result.warnings,
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

        let mut settings_file = storage::settings_store::SettingsFile::from_reader_settings(
            &state.reader_settings,
            state.current_book.as_ref().map(|b| b.id.clone()),
        );
        settings_file.window_size = state.window_size;
        settings_file.window_pos = state.window_pos;
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

/// EPUB 缩进标记前缀（由 strip_html_tags 注入）
const INDENT_MARKER: &str = "\x01INDENT\x01";

fn build_chapter(index: usize, title: &str, text: &str) -> Chapter {
    let mut line_number = 0usize;
    let paragraphs = text
        .split("\n\n")
        .enumerate()
        .filter_map(|(paragraph_index, raw)| {
            line_number += 1;
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }

            // 检测 EPUB 缩进标记
            let (indent_level, clean_text) = if trimmed.starts_with(INDENT_MARKER) {
                (1u8, &trimmed[INDENT_MARKER.len()..])
            } else {
                // 检测 TXT 行首缩进：全角空格、半角空格、Tab
                let indent = detect_indent(trimmed);
                (indent, trimmed)
            };

            let clean_text = clean_text.trim();
            if clean_text.is_empty() {
                return None;
            }

            Some(Paragraph {
                index: paragraph_index,
                text: clean_text.to_string(),
                kind: infer_paragraph_kind(clean_text),
                indent_level,
                source_line_hint: Some(line_number),
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

/// 检测段落行首缩进级别
/// 全角空格（\u{3000}）、半角空格、Tab 均算作缩进
fn detect_indent(text: &str) -> u8 {
    let mut indent: u8 = 0;
    for ch in text.chars() {
        match ch {
            '\u{3000}' => indent += 1, // 全角空格
            ' ' => indent += 1,
            '\t' => indent += 1,
            _ => break,
        }
        if indent >= 4 {
            break;
        }
    }
    indent.min(4)
}

fn infer_paragraph_kind(text: &str) -> ParagraphKind {
    let char_count = text.chars().count();

    // Separator: pure symbol lines like ***, ---, ＊＊＊, * * *, ————
    if is_separator_line(text) {
        return ParagraphKind::Separator;
    }

    // Quote: lines starting with > (including >> for nested quotes), or wrapped in quotation marks
    if text.starts_with('>') {
        return ParagraphKind::Quote;
    }
    if is_quote_wrapped(text) {
        return ParagraphKind::Quote;
    }

    // Title: 第X章, Chapter X, Part X, and special Chinese chapter words
    if is_title_line(text, char_count) {
        return ParagraphKind::Title;
    }

    // Subtitle: short text (<30 chars) that looks like a section header
    if char_count < 30 && char_count >= 2 && is_subtitle_like(text) {
        return ParagraphKind::Subtitle;
    }

    ParagraphKind::Body
}

fn is_separator_line(text: &str) -> bool {
    let separators = ["***", "---", "＊＊＊", "* * *", "————", "====", "~~~~", "___"];
    let trimmed = text.trim();
    if separators.contains(&trimmed) {
        return true;
    }
    // Repeated single symbol (3+ times)
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3 && chars.iter().all(|&c| c == chars[0]) {
        let c = chars[0];
        if c == '*' || c == '-' || c == '—' || c == '＊' || c == '~' || c == '=' || c == '_' {
            return true;
        }
    }
    // T14 E-3: pure dotted line (........)
    if chars.len() >= 5 && chars.iter().all(|&c| c == '.') {
        return true;
    }
    // T14 E-3: pure centered dashes (─ ─ ─ or · · ·)
    let non_space: Vec<char> = chars.iter().copied().filter(|c| !c.is_whitespace()).collect();
    if non_space.len() >= 3 {
        if non_space.iter().all(|&c| c == '─' || c == '━' || c == '·' || c == '•') && non_space.windows(2).all(|w| w[0] == w[1]) {
            return true;
        }
    }
    false
}

fn is_quote_wrapped(text: &str) -> bool {
    let pairs = [('「', '」'), ('『', '』'), ('"', '"'), ('\'', '\'')];
    let trimmed = text.trim();
    for (open, close) in pairs {
        if trimmed.starts_with(open) && trimmed.ends_with(close) && trimmed.len() > 2 {
            return true;
        }
    }
    // 《》 book title marks wrapping the entire text
    if trimmed.starts_with('《') && trimmed.ends_with('》') && trimmed.len() > 2 {
        return true;
    }
    false
}

fn is_title_line(text: &str, char_count: usize) -> bool {
    if char_count >= 50 {
        return false;
    }
    // Chinese chapter: 第X章/回/节/卷/部分/篇
    if text.starts_with('第') && (text.contains('章') || text.contains('回') || text.contains('节') || text.contains('卷') || text.contains("部分") || text.contains('篇')) {
        return true;
    }
    // English chapter/part
    let upper = text.to_uppercase();
    if upper.starts_with("CHAPTER ") || upper.starts_with("PART ") {
        return true;
    }
    // Special Chinese chapter words
    let special = ["序章", "终章", "番外", "楔子", "尾声", "引子", "后记", "前言", "序言"];
    for word in special {
        if text.starts_with(word) {
            return true;
        }
    }
    false
}

fn is_subtitle_like(text: &str) -> bool {
    let trimmed = text.trim();
    // Chinese number + separator: 一、xxx; 1. xxx
    if let Some(first) = trimmed.chars().next() {
        if first.is_ascii_digit() {
            let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
            if rest.starts_with('.') || rest.starts_with('、') {
                return true;
            }
        }
    }
    // Chinese number prefix + separator (single and multi-digit)
    let cn_numbers = [
        "一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
        "十一", "十二", "十三", "十四", "十五", "十六", "十七", "十八", "十九", "二十",
        "三十", "四十", "五十", "六十", "七十", "八十", "九十",
    ];
    for num in cn_numbers {
        if trimmed.starts_with(num) {
            let rest = &trimmed[num.len()..];
            if rest.starts_with('、') || rest.starts_with('，') || rest.starts_with(',') {
                return true;
            }
        }
    }
    false
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

/// Strip fragment (#...) from href, returning just the file path part.
fn strip_href_fragment(href: &str) -> &str {
    href.split('#').next().unwrap_or(href)
}

/// Extract the filename component from a path (e.g. "OEBPS/ch1.xhtml" → "ch1.xhtml").
fn href_filename(href: &str) -> &str {
    strip_href_fragment(href)
        .rsplit('/')
        .next()
        .unwrap_or(strip_href_fragment(href))
}

/// Build a mapping from href filename to spine chapter index.
fn build_href_index(spine_hrefs: &[String]) -> std::collections::HashMap<String, usize> {
    let mut map = std::collections::HashMap::new();
    for (index, href) in spine_hrefs.iter().enumerate() {
        let key = href_filename(href).to_string();
        map.entry(key).or_insert(index);
    }
    map
}

/// Recursively set chapter_index on TocItems using the href→spine_index mapping.
fn map_toc_chapter_indices(
    items: Vec<TocItem>,
    href_to_index: &std::collections::HashMap<String, usize>,
) -> Vec<TocItem> {
    items
        .into_iter()
        .map(|mut item| {
            if item.chapter_index.is_none() {
                if let Some(ref href) = item.href {
                    let key = href_filename(href).to_string();
                    if let Some(&idx) = href_to_index.get(&key) {
                        item.chapter_index = Some(idx);
                    }
                }
            }
            item.children = map_toc_chapter_indices(item.children, href_to_index);
            item
        })
        .collect()
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
        let thread_id = std::thread::current().id();
        std::env::temp_dir().join(format!("reader-demo-test-{}-{:?}.{ext}", millis, thread_id))
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

    fn create_epub_with_metadata_fixture() -> std::path::PathBuf {
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
<package version="2.0" xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <metadata>
    <dc:title>"#,
        )
        .unwrap();
        zip.write_all("测试书籍标题".as_bytes()).unwrap();
        zip.write_all(
            br#"</dc:title>
    <dc:creator>"#,
        )
        .unwrap();
        zip.write_all("测试作者".as_bytes()).unwrap();
        zip.write_all(
            br#"</dc:creator>
    <dc:language>zh-CN</dc:language>
    <dc:identifier>isbn-1234567890</dc:identifier>
  </metadata>
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

    #[test]
    fn recent_book_selected_reopens_book() {
        let path = create_txt_fixture();
        let path_str = path.to_string_lossy().to_string();
        let mut adapter = CompatAdapter::new();

        // Open the book to populate recent_books
        adapter.dispatch(Action::OpenBookSelected(path_str));
        let book_id = adapter
            .state()
            .current_book
            .as_ref()
            .map(|b| b.id.clone())
            .unwrap();
        assert_eq!(adapter.state().ui_state.screen, ScreenKind::Reader);

        // Close the book
        adapter.dispatch(Action::CloseBook);
        assert_eq!(adapter.state().ui_state.screen, ScreenKind::EmptyLibrary);
        assert!(adapter.state().current_book.is_none());

        // Select from recent books — should reopen
        adapter.dispatch(Action::RecentBookSelected(book_id));
        assert_eq!(adapter.state().ui_state.screen, ScreenKind::Reader);
        assert!(adapter.state().current_book.is_some());
        assert!(adapter.state().ui_state.pending_open_path.is_none());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn epub_metadata_is_extracted() {
        let path = create_epub_with_metadata_fixture();
        let mut adapter = CompatAdapter::new();

        adapter.dispatch(Action::OpenBookSelected(path.to_string_lossy().to_string()));

        let book = adapter.state().current_book.as_ref().unwrap();
        assert_eq!(book.metadata.title, "测试书籍标题");
        assert_eq!(book.metadata.author.as_deref(), Some("测试作者"));
        assert_eq!(book.metadata.language.as_deref(), Some("zh-CN"));
        assert_eq!(book.metadata.identifier.as_deref(), Some("isbn-1234567890"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn epub_toc_chapter_index_mapped_from_ncx() {
        let path = temp_path("epub");
        let file = fs::File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/content.opf", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/toc.ncx", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
  <navMap>
    <navPoint id="np1"><navLabel><text>Chapter One</text></navLabel><content src="ch1.xhtml"/></navPoint>
    <navPoint id="np2"><navLabel><text>Chapter Two</text></navLabel><content src="ch2.xhtml"/></navPoint>
  </navMap>
</ncx>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/ch1.xhtml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Content one.</p></body></html>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/ch2.xhtml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Content two.</p></body></html>"#,
        )
        .unwrap();

        zip.finish().unwrap();

        let mut adapter = CompatAdapter::new();
        adapter.dispatch(Action::OpenBookSelected(path.to_string_lossy().to_string()));

        let book = adapter.state().current_book.as_ref().unwrap();
        assert!(book.toc.len() >= 2, "expected at least 2 TOC items, got {}", book.toc.len());
        assert_eq!(book.toc[0].chapter_index, Some(0), "first TOC item should map to chapter 0");
        assert_eq!(book.toc[1].chapter_index, Some(1), "second TOC item should map to chapter 1");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn strip_href_fragment_removes_hash() {
        assert_eq!(strip_href_fragment("ch1.xhtml#section1"), "ch1.xhtml");
        assert_eq!(strip_href_fragment("ch1.xhtml"), "ch1.xhtml");
        assert_eq!(strip_href_fragment("#fragment"), "");
    }

    #[test]
    fn href_filename_strips_fragment_and_path() {
        assert_eq!(href_filename("OEBPS/ch1.xhtml#section1"), "ch1.xhtml");
        assert_eq!(href_filename("ch1.xhtml"), "ch1.xhtml");
        assert_eq!(href_filename("OEBPS/ch1.xhtml"), "ch1.xhtml");
    }

    #[test]
    fn build_href_index_matches_fragments() {
        let spine = vec!["OEBPS/ch1.xhtml".to_string(), "OEBPS/ch2.xhtml".to_string()];
        let index = build_href_index(&spine);

        assert_eq!(index.get("ch1.xhtml"), Some(&0));
        assert_eq!(index.get("ch2.xhtml"), Some(&1));
    }

    #[test]
    fn epub_toc_with_fragment_hrefs_maps_correctly() {
        let path = temp_path("epub");
        let file = fs::File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();

        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/content.opf", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/toc.ncx", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
  <navMap>
    <navPoint id="np1"><navLabel><text>Chapter One</text></navLabel><content src="ch1.xhtml#intro"/></navPoint>
    <navPoint id="np2"><navLabel><text>Chapter Two</text></navLabel><content src="ch2.xhtml#part1"/></navPoint>
  </navMap>
</ncx>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/ch1.xhtml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Content one.</p></body></html>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/ch2.xhtml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Content two.</p></body></html>"#,
        )
        .unwrap();

        zip.finish().unwrap();

        let mut adapter = CompatAdapter::new();
        adapter.dispatch(Action::OpenBookSelected(path.to_string_lossy().to_string()));

        let book = adapter.state().current_book.as_ref().unwrap();
        assert!(book.toc.len() >= 2, "expected at least 2 TOC items, got {}", book.toc.len());
        assert_eq!(book.toc[0].chapter_index, Some(0), "fragment href should map to chapter 0");
        assert_eq!(book.toc[1].chapter_index, Some(1), "fragment href should map to chapter 1");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn paragraph_kind_separator() {
        assert_eq!(infer_paragraph_kind("***"), ParagraphKind::Separator);
        assert_eq!(infer_paragraph_kind("---"), ParagraphKind::Separator);
        assert_eq!(infer_paragraph_kind("* * *"), ParagraphKind::Separator);
        assert_eq!(infer_paragraph_kind("————"), ParagraphKind::Separator);
        assert_eq!(infer_paragraph_kind("＊＊＊"), ParagraphKind::Separator);
    }

    #[test]
    fn paragraph_kind_quote() {
        assert_eq!(infer_paragraph_kind("> 引用内容"), ParagraphKind::Quote);
        assert_eq!(infer_paragraph_kind("「对话内容」"), ParagraphKind::Quote);
        assert_eq!(infer_paragraph_kind("『引用文字』"), ParagraphKind::Quote);
        assert_eq!(infer_paragraph_kind("\"引用段落\""), ParagraphKind::Quote);
    }

    #[test]
    fn paragraph_kind_title() {
        assert_eq!(infer_paragraph_kind("第1章 开始"), ParagraphKind::Title);
        assert_eq!(infer_paragraph_kind("Chapter 1 Beginning"), ParagraphKind::Title);
        assert_eq!(infer_paragraph_kind("Part 1"), ParagraphKind::Title);
        assert_eq!(infer_paragraph_kind("序章"), ParagraphKind::Title);
        assert_eq!(infer_paragraph_kind("终章 结局"), ParagraphKind::Title);
    }

    #[test]
    fn paragraph_kind_subtitle() {
        assert_eq!(infer_paragraph_kind("一、开篇"), ParagraphKind::Subtitle);
        assert_eq!(infer_paragraph_kind("1. 小节"), ParagraphKind::Subtitle);
        assert_eq!(infer_paragraph_kind("十二、总结"), ParagraphKind::Subtitle);
    }

    #[test]
    fn paragraph_kind_body() {
        assert_eq!(infer_paragraph_kind("这是一段普通的正文内容，足够长以避免被误判为标题。"), ParagraphKind::Body);
        assert_eq!(infer_paragraph_kind("他说：今天天气真好。"), ParagraphKind::Body);
    }
}
