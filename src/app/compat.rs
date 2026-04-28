use log::info;

use crate::domain::app_state::AppState;
use crate::domain::book::Book;
use crate::domain::book_assets::BookAssets;
use crate::domain::book_format::BookFormat;
use crate::domain::book_load_info::BookLoadInfo;
use crate::domain::book_metadata::BookMetadata;
use crate::domain::chapter::Chapter;
use crate::domain::enums::ScreenKind;
use crate::domain::paragraph::Paragraph;
use crate::domain::paragraph_kind::ParagraphKind;
use crate::domain::reading_progress::ReadingProgress;
use crate::domain::recent_book_item::RecentBookItem;
use crate::domain::toc_item::TocItem;
use crate::domain::ui_state::UiState;
use crate::parser::ParserFactory;

pub struct CompatAdapter {
    state: AppState,
    cached_contents: Vec<String>,
    cached_titles: Vec<String>,
    cached_book_id: Option<String>,
}

impl CompatAdapter {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            cached_contents: Vec::new(),
            cached_titles: Vec::new(),
            cached_book_id: None,
        }
    }

    fn refresh_cache(&mut self) {
        let book_id = self
            .state
            .current_book
            .as_ref()
            .map(|b| b.id.clone());
        if book_id != self.cached_book_id {
            if let Some(ref book) = self.state.current_book {
                self.cached_contents = book.chapters.iter().map(|ch| ch.content.clone()).collect();
                self.cached_titles = book.chapters.iter().map(|ch| ch.title.clone()).collect();
            } else {
                self.cached_contents.clear();
                self.cached_titles.clear();
            }
            self.cached_book_id = book_id;
        }
    }

    fn add_recent_book(&mut self, item: RecentBookItem) {
        self.state.recent_books.retain(|b| b.book_id != item.book_id);
        self.state.recent_books.insert(0, item);
        if self.state.recent_books.len() > 20 {
            self.state.recent_books.truncate(20);
        }
    }

    fn remove_recent_book(&mut self, book_id: &str) {
        self.state.recent_books.retain(|b| b.book_id != book_id);
    }

    // Old API surface -- backward-compatible accessors

    pub fn content(&mut self) -> &[String] {
        self.refresh_cache();
        &self.cached_contents
    }

    pub fn chapter_titles(&mut self) -> &[String] {
        self.refresh_cache();
        &self.cached_titles
    }

    pub fn current_page(&self) -> usize {
        self.state
            .reading_progress
            .as_ref()
            .map(|p| p.chapter_index)
            .unwrap_or(0)
    }

    pub fn set_current_page(&mut self, page: usize) {
        if let Some(ref mut progress) = self.state.reading_progress {
            progress.chapter_index = page;
        } else {
            let book_id = self
                .state
                .current_book
                .as_ref()
                .map(|b| b.id.clone())
                .unwrap_or_default();
            self.state.reading_progress = Some(ReadingProgress {
                book_id,
                chapter_index: page,
                paragraph_index: None,
                scroll_offset: 0.0,
                progress_percent: 0.0,
                last_read_at: chrono::Utc::now().to_rfc3339(),
                session_read_seconds: 0,
                total_read_seconds: 0,
            });
        }
    }

    pub fn status(&self) -> &str {
        &self.state.status_message
    }

    pub fn set_status(&mut self, msg: String) {
        self.state.status_message = msg;
    }

    pub fn open_book(&mut self, path: &str) {
        self.state.status_message = format!("正在打开文件: {}", path);
        info!("{}", self.state.status_message);

        let start = std::time::Instant::now();

        let parser = match ParserFactory::get_parser(path) {
            Some(p) => p,
            None => {
                self.state.status_message = "不支持的文件格式".to_string();
                return;
            }
        };

        let result = match parser.parse(path) {
            Ok(r) => r,
            Err(e) => {
                self.state.status_message = format!("解析失败: {}", e);
                return;
            }
        };

        let format = if path.ends_with(".epub") {
            BookFormat::Epub
        } else {
            BookFormat::Txt
        };

        let chapters: Vec<Chapter> = result
            .content
            .into_iter()
            .enumerate()
            .map(|(i, text)| {
                let paragraphs: Vec<Paragraph> = text
                    .split("\n\n")
                    .enumerate()
                    .map(|(pi, p)| Paragraph {
                        index: pi,
                        text: p.trim().to_string(),
                        kind: ParagraphKind::Body,
                        indent_level: 0,
                        source_line_hint: None,
                    })
                    .filter(|p| !p.text.is_empty())
                    .collect();

                let title = result
                    .chapter_titles
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("章节 {}", i + 1));

                let full_text = paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n");

                Chapter {
                    id: format!("ch-{}", i),
                    index: i,
                    title,
                    raw_title: None,
                    content: full_text,
                    paragraphs,
                    word_count: 0,
                    char_count: 0,
                    source_href: None,
                    anchor: None,
                    warnings: Vec::new(),
                }
            })
            .collect();

        let toc: Vec<TocItem> = result
            .chapter_titles
            .iter()
            .enumerate()
            .map(|(i, title)| TocItem {
                id: format!("toc-{}", i),
                title: title.clone(),
                chapter_index: Some(i),
                href: None,
                depth: 0,
                children: Vec::new(),
                is_generated: true,
            })
            .collect();

        let book_id = format!("book-{}", chrono::Utc::now().timestamp_millis());
        let duration_ms = start.elapsed().as_millis() as u64;
        let file_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);

        let chapter_count = result.chapter_titles.len();

        let metadata = BookMetadata {
            title: result
                .chapter_titles
                .first()
                .cloned()
                .unwrap_or_else(|| "未命名书籍".to_string()),
            author: None,
            language: None,
            publisher: None,
            description: None,
            identifier: None,
            series: None,
            cover_title: None,
            created_at: None,
            modified_at: None,
        };

        let book = Book {
            id: book_id.clone(),
            source_path: std::path::PathBuf::from(path),
            format,
            metadata: metadata.clone(),
            toc,
            chapters,
            assets: BookAssets {
                cover_image_bytes: None,
                cover_media_type: None,
                has_images: false,
                embedded_styles_detected: false,
            },
            load_info: BookLoadInfo {
                parser_name: if path.ends_with(".epub") {
                    "EpubParser".to_string()
                } else {
                    "TxtParser".to_string()
                },
                parse_warnings: Vec::new(),
                chapter_count,
                loaded_at: chrono::Utc::now().to_rfc3339(),
                source_file_size: file_size,
                load_duration_ms: duration_ms,
            },
        };

        self.state.current_book = Some(book);
        self.state.reading_progress = Some(ReadingProgress {
            book_id: book_id.clone(),
            chapter_index: 0,
            paragraph_index: None,
            scroll_offset: 0.0,
            progress_percent: 0.0,
            last_read_at: chrono::Utc::now().to_rfc3339(),
            session_read_seconds: 0,
            total_read_seconds: 0,
        });

        // Add to recent books
        self.add_recent_book(RecentBookItem {
            book_id,
            title: metadata.title.clone(),
            author: None,
            source_path: path.to_string(),
            format: "epub".to_string(),
            last_opened_at: chrono::Utc::now().to_rfc3339(),
            last_progress_percent: 0.0,
            cover_cached: false,
            is_missing: false,
        });

        self.state.ui_state = UiState {
            screen: ScreenKind::Reader,
            ..self.state.ui_state.clone()
        };
        self.state.status_message = format!("内容已加载，共 {} 章", chapter_count);
        info!("{}", self.state.status_message);

        // Invalidate cache so refresh_cache picks up new data
        self.cached_book_id = None;
    }

    // Access to new state

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    pub fn dispatch(&mut self, _action: super::Action) {
        // Future: reducer/controller
    }
}
