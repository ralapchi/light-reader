use chrono::Utc;

use crate::app::Action;
use crate::domain::app_state::AppState;
use crate::domain::book::Book;
use crate::domain::book_format::BookFormat;
use crate::domain::enums::ScreenKind;
use crate::domain::reading_progress::ReadingProgress;
use crate::domain::recent_book_item::RecentBookItem;

pub fn reduce(state: &mut AppState, action: Action) {
    match action {
        Action::OpenBookSucceeded(book) => open_book_succeeded(state, book),
        Action::OpenBookFailed(err) => {
            state.status_message = err.message.clone();
            state.last_error = Some(err);
            state.ui_state.is_loading = false;
            state.ui_state.screen = ScreenKind::Error;
            state.ui_state.pending_open_path = None;
        }
        Action::GoToChapter(index) => go_to_chapter(state, index),
        Action::NextChapter => {
            let next_index = current_chapter_index(state).saturating_add(1);
            go_to_chapter(state, next_index);
        }
        Action::PrevChapter => {
            let prev_index = current_chapter_index(state).saturating_sub(1);
            go_to_chapter(state, prev_index);
        }
        Action::ThemeChanged(theme) => {
            state.reader_settings.theme = theme;
        }
        Action::SwitchLeftPanelTab(tab) => {
            state.ui_state.left_panel_tab = tab;
        }
        Action::DismissError => {
            state.last_error = None;
            state.ui_state.screen = if state.current_book.is_some() {
                ScreenKind::Reader
            } else {
                ScreenKind::EmptyLibrary
            };
            if state.status_message.is_empty() {
                state.status_message = "就绪".to_string();
            }
        }
        _ => {}
    }
}

fn open_book_succeeded(state: &mut AppState, book: Book) {
    let chapter_count = book.chapters.len();
    let book_id = book.id.clone();
    let recent_item = RecentBookItem {
        book_id: book_id.clone(),
        title: book.metadata.title.clone(),
        author: book.metadata.author.clone(),
        source_path: book.source_path.to_string_lossy().into_owned(),
        format: format_label(&book.format).to_string(),
        last_opened_at: Utc::now().to_rfc3339(),
        last_progress_percent: if chapter_count > 0 {
            1.0 / chapter_count as f32
        } else {
            0.0
        },
        cover_cached: book.assets.cover_image_bytes.is_some(),
        is_missing: false,
    };

    state.current_book = Some(book);
    state.reading_progress = Some(progress_for(&book_id, 0, chapter_count));
    state.recent_books.retain(|item| item.book_id != book_id);
    state.recent_books.insert(0, recent_item);
    if state.recent_books.len() > 20 {
        state.recent_books.truncate(20);
    }
    state.search_state = Default::default();
    state.last_error = None;
    state.ui_state.is_loading = false;
    state.ui_state.pending_open_path = None;
    state.ui_state.screen = ScreenKind::Reader;
    state.status_message = format!("内容已加载，共 {} 章", chapter_count);
}

fn go_to_chapter(state: &mut AppState, index: usize) {
    let total = state
        .current_book
        .as_ref()
        .map(|book| book.chapters.len())
        .unwrap_or(0);

    if total == 0 {
        return;
    }

    let clamped = index.min(total.saturating_sub(1));
    let book_id = state
        .current_book
        .as_ref()
        .map(|book| book.id.clone())
        .unwrap_or_default();

    state.reading_progress = Some(progress_for(&book_id, clamped, total));
}

fn progress_for(book_id: &str, chapter_index: usize, total: usize) -> ReadingProgress {
    let progress_percent = if total == 0 {
        0.0
    } else {
        ((chapter_index + 1) as f32 / total as f32).clamp(0.0, 1.0)
    };

    ReadingProgress {
        book_id: book_id.to_string(),
        chapter_index,
        paragraph_index: None,
        scroll_offset: 0.0,
        progress_percent,
        last_read_at: Utc::now().to_rfc3339(),
        session_read_seconds: 0,
        total_read_seconds: 0,
    }
}

fn current_chapter_index(state: &AppState) -> usize {
    state
        .reading_progress
        .as_ref()
        .map(|progress| progress.chapter_index)
        .unwrap_or(0)
}

fn format_label(format: &BookFormat) -> &'static str {
    match format {
        BookFormat::Epub => "epub",
        BookFormat::Txt => "txt",
        BookFormat::ReservedPdf => "pdf",
        BookFormat::ReservedMobi => "mobi",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::book_assets::BookAssets;
    use crate::domain::book_load_info::BookLoadInfo;
    use crate::domain::book_metadata::BookMetadata;
    use crate::domain::chapter::Chapter;
    use crate::domain::paragraph::Paragraph;
    use crate::domain::paragraph_kind::ParagraphKind;
    use crate::domain::theme_kind::ThemeKind;
    use crate::domain::toc_item::TocItem;
    use std::path::PathBuf;

    fn sample_book(format: BookFormat) -> Book {
        Book {
            id: "book-1".to_string(),
            source_path: PathBuf::from("/tmp/sample.txt"),
            format,
            metadata: BookMetadata {
                title: "Sample".to_string(),
                author: Some("Tester".to_string()),
                language: None,
                publisher: None,
                description: None,
                identifier: None,
                series: None,
                cover_title: None,
                created_at: None,
                modified_at: None,
            },
            toc: vec![TocItem {
                id: "toc-1".to_string(),
                title: "Chapter 1".to_string(),
                chapter_index: Some(0),
                href: None,
                depth: 0,
                children: Vec::new(),
                is_generated: true,
            }],
            chapters: vec![
                Chapter {
                    id: "ch-1".to_string(),
                    index: 0,
                    title: "Chapter 1".to_string(),
                    raw_title: None,
                    content: "Body".to_string(),
                    paragraphs: vec![Paragraph {
                        index: 0,
                        text: "Body".to_string(),
                        kind: ParagraphKind::Body,
                        indent_level: 0,
                        source_line_hint: None,
                    }],
                    word_count: 1,
                    char_count: 4,
                    source_href: None,
                    anchor: None,
                    warnings: Vec::new(),
                },
                Chapter {
                    id: "ch-2".to_string(),
                    index: 1,
                    title: "Chapter 2".to_string(),
                    raw_title: None,
                    content: "Body 2".to_string(),
                    paragraphs: vec![Paragraph {
                        index: 0,
                        text: "Body 2".to_string(),
                        kind: ParagraphKind::Body,
                        indent_level: 0,
                        source_line_hint: None,
                    }],
                    word_count: 2,
                    char_count: 6,
                    source_href: None,
                    anchor: None,
                    warnings: Vec::new(),
                },
            ],
            assets: BookAssets {
                cover_image_bytes: None,
                cover_media_type: None,
                has_images: false,
                embedded_styles_detected: false,
            },
            load_info: BookLoadInfo {
                parser_name: "Test".to_string(),
                parse_warnings: Vec::new(),
                chapter_count: 2,
                loaded_at: Utc::now().to_rfc3339(),
                source_file_size: 0,
                load_duration_ms: 0,
            },
        }
    }

    #[test]
    fn open_book_success_uses_actual_format_for_recent_item() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        assert_eq!(state.recent_books[0].format, "txt");

        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Epub)));
        assert_eq!(state.recent_books[0].format, "epub");
    }

    #[test]
    fn chapter_navigation_updates_progress() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));

        reduce(&mut state, Action::GoToChapter(1));
        assert_eq!(
            state.reading_progress.as_ref().map(|progress| progress.chapter_index),
            Some(1)
        );

        reduce(&mut state, Action::PrevChapter);
        assert_eq!(
            state.reading_progress.as_ref().map(|progress| progress.chapter_index),
            Some(0)
        );

        reduce(&mut state, Action::NextChapter);
        assert_eq!(
            state.reading_progress.as_ref().map(|progress| progress.chapter_index),
            Some(1)
        );
    }

    #[test]
    fn theme_change_updates_reader_settings() {
        let mut state = AppState::default();
        reduce(&mut state, Action::ThemeChanged(ThemeKind::Sepia));
        assert_eq!(state.reader_settings.theme, ThemeKind::Sepia);
    }
}
