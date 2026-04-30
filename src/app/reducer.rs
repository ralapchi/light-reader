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
            set_status_message(state, err.message.clone());
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
        Action::ToggleSidebar => {
            state.ui_state.sidebar_collapsed = !state.ui_state.sidebar_collapsed;
        }
        Action::ToggleSettingsPanel => {
            state.ui_state.show_settings_panel = !state.ui_state.show_settings_panel;
        }
        Action::ToggleSearchPanel => {
            state.ui_state.show_search_panel = !state.ui_state.show_search_panel;
            if state.ui_state.show_search_panel {
                state.ui_state.focused_search_input = true;
            }
        }
        Action::CloseBook => {
            state.current_book = None;
            state.reading_progress = None;
            state.bookmarks.clear();
            state.search_state = Default::default();
            state.last_error = None;
            state.ui_state.screen = ScreenKind::EmptyLibrary;
            state.ui_state.is_loading = false;
            state.ui_state.pending_open_path = None;
            clear_status_message(state);
        }
        Action::ReaderSettingChanged(key, value) => {
            apply_reader_setting(&mut state.reader_settings, &key, &value);
        }
        Action::RestoreDefaultSettings => {
            state.reader_settings = Default::default();
        }
        Action::UpdateScrollOffset(offset) => {
            if let Some(ref mut progress) = state.reading_progress {
                progress.scroll_offset = offset;
            }
        }
        Action::AddBookmarkRequested => {
            if let (Some(book), Some(progress)) = (&state.current_book, &state.reading_progress) {
                let chapter_index = progress.chapter_index;
                let paragraph_index = progress.paragraph_index;
                let snippet = book
                    .chapters
                    .get(chapter_index)
                    .and_then(|ch| {
                        paragraph_index
                            .and_then(|pi| ch.paragraphs.get(pi))
                            .or(ch.paragraphs.first())
                            .map(|p| p.text.chars().take(100).collect::<String>())
                    })
                    .unwrap_or_default();
                let title = book
                    .chapters
                    .get(chapter_index)
                    .map(|ch| ch.title.clone())
                    .unwrap_or_else(|| "书签".to_string());
                let bookmark = crate::domain::bookmark::Bookmark {
                    id: format!("bm-{}", Utc::now().timestamp_millis()),
                    book_id: book.id.clone(),
                    chapter_index,
                    paragraph_index,
                    title,
                    snippet,
                    created_at: Utc::now().to_rfc3339(),
                    note: None,
                };
                state.bookmarks.push(bookmark);
                set_status_message(state, "已添加书签".to_string());
            }
        }
        Action::RemoveBookmark(id) => {
            state.bookmarks.retain(|b| b.id != id);
        }
        Action::JumpToBookmark(id) => {
            if let Some(bookmark) = state.bookmarks.iter().find(|b| b.id == id).cloned() {
                go_to_chapter(state, bookmark.chapter_index);
                if let Some(para_idx) = bookmark.paragraph_index {
                    if let Some(ref mut progress) = state.reading_progress {
                        progress.paragraph_index = Some(para_idx);
                    }
                }
            }
        }
        Action::SearchQueryChanged(query) => {
            state.search_state.current_query = Some(query);
        }
        Action::SearchSubmitted => {
            if let Some(query) = &state.search_state.current_query {
                let results = execute_search(state, query);
                state.search_state.results = results;
                state.search_state.selected_result_index = None;
                state.search_state.last_search_at = Some(Utc::now().to_rfc3339());
            }
        }
        Action::SearchResultSelected(index) => {
            state.search_state.selected_result_index = Some(index);
            if let Some(result) = state.search_state.results.get(index).cloned() {
                go_to_chapter(state, result.chapter_index);
                if let Some(ref mut progress) = state.reading_progress {
                    progress.paragraph_index = Some(result.paragraph_index);
                }
            }
        }
        Action::ClearSearch => {
            state.search_state = Default::default();
        }
        Action::RecentBookSelected(book_id) => {
            if let Some(item) = state.recent_books.iter().find(|r| r.book_id == book_id) {
                let path = item.source_path.clone();
                state.ui_state.pending_open_path = Some(std::path::PathBuf::from(path));
            }
        }
        Action::RemoveRecentBook(book_id) => {
            state.recent_books.retain(|item| item.book_id != book_id);
        }
        Action::StatusMessageTimedOut => {
            if state.last_error.is_none() {
                clear_status_message(state);
            }
        }
        Action::DismissError => {
            state.last_error = None;
            state.ui_state.screen = if state.current_book.is_some() {
                ScreenKind::Reader
            } else {
                ScreenKind::EmptyLibrary
            };
            if state.status_message.is_empty() {
                clear_status_message(state);
            }
        }
        Action::CloseSearchOrSettings => {
            if state.ui_state.show_search_panel {
                state.ui_state.show_search_panel = false;
                state.search_state = Default::default();
            }
            if state.ui_state.show_settings_panel {
                state.ui_state.show_settings_panel = false;
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
    set_status_message(state, format!("内容已加载，共 {} 章", chapter_count));
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

fn execute_search(state: &AppState, query: &crate::domain::search_query::SearchQuery) -> Vec<crate::domain::search_result::SearchResult> {
    let book = match &state.current_book {
        Some(book) => book,
        None => return Vec::new(),
    };
    let keyword = if query.case_sensitive {
        query.keyword.clone()
    } else {
        query.keyword.to_lowercase()
    };
    let mut results = Vec::new();
    let search_all = matches!(query.scope, crate::domain::search_enums::SearchScope::EntireBook);
    let current_chapter = state
        .reading_progress
        .as_ref()
        .map(|p| p.chapter_index)
        .unwrap_or(0);

    for chapter in &book.chapters {
        if !search_all && chapter.index != current_chapter {
            continue;
        }
        for paragraph in &chapter.paragraphs {
            let haystack = if query.case_sensitive {
                paragraph.text.clone()
            } else {
                paragraph.text.to_lowercase()
            };
            if let Some(pos) = haystack.find(&keyword) {
                let snippet_start = floor_char_boundary(&paragraph.text, pos.saturating_sub(20));
                let snippet_end = ceil_char_boundary(&paragraph.text, (pos + keyword.len() + 20).min(paragraph.text.len()));
                let snippet = paragraph.text[snippet_start..snippet_end].to_string();
                results.push(crate::domain::search_result::SearchResult {
                    book_id: book.id.clone(),
                    chapter_index: chapter.index,
                    paragraph_index: paragraph.index,
                    match_start: pos,
                    match_end: pos + keyword.len(),
                    chapter_title: chapter.title.clone(),
                    snippet,
                    score: 1.0,
                });
            }
        }
    }
    results
}

/// Find the nearest valid UTF-8 char boundary at or before `index`.
fn floor_char_boundary(s: &str, index: usize) -> usize {
    let mut i = index.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Find the nearest valid UTF-8 char boundary at or after `index`.
fn ceil_char_boundary(s: &str, index: usize) -> usize {
    let mut i = index.min(s.len());
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn set_status_message(state: &mut AppState, message: String) {
    state.status_message = message;
    state.status_message_set_at = Some(Utc::now().to_rfc3339());
}

fn clear_status_message(state: &mut AppState) {
    state.status_message = "就绪".to_string();
    state.status_message_set_at = None;
}

fn apply_reader_setting(settings: &mut crate::domain::reader_settings::ReaderSettings, key: &str, value: &str) {
    match key {
        "font_size" => {
            if let Ok(v) = value.parse::<f32>() {
                settings.font_size = v.max(8.0);
            }
        }
        "line_height" => {
            if let Ok(v) = value.parse::<f32>() {
                settings.line_height = v.max(1.0);
            }
        }
        "paragraph_spacing" => {
            if let Ok(v) = value.parse::<f32>() {
                settings.paragraph_spacing = v.max(0.0);
            }
        }
        "content_width" => {
            if let Ok(v) = value.parse::<f32>() {
                settings.content_width = v.max(200.0);
            }
        }
        "side_margin" => {
            if let Ok(v) = value.parse::<f32>() {
                settings.side_margin = v.max(0.0);
            }
        }
        "show_toc" => {
            if let Ok(v) = value.parse::<bool>() {
                settings.show_toc = v;
            }
        }
        "show_status_bar" => {
            if let Ok(v) = value.parse::<bool>() {
                settings.show_status_bar = v;
            }
        }
        _ => {}
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

    #[test]
    fn toggle_sidebar_flips_collapsed() {
        let mut state = AppState::default();
        assert!(!state.ui_state.sidebar_collapsed);
        reduce(&mut state, Action::ToggleSidebar);
        assert!(state.ui_state.sidebar_collapsed);
        reduce(&mut state, Action::ToggleSidebar);
        assert!(!state.ui_state.sidebar_collapsed);
    }

    #[test]
    fn toggle_settings_panel() {
        let mut state = AppState::default();
        assert!(!state.ui_state.show_settings_panel);
        reduce(&mut state, Action::ToggleSettingsPanel);
        assert!(state.ui_state.show_settings_panel);
        reduce(&mut state, Action::ToggleSettingsPanel);
        assert!(!state.ui_state.show_settings_panel);
    }

    #[test]
    fn toggle_search_panel_sets_focus() {
        let mut state = AppState::default();
        assert!(!state.ui_state.show_search_panel);
        reduce(&mut state, Action::ToggleSearchPanel);
        assert!(state.ui_state.show_search_panel);
        assert!(state.ui_state.focused_search_input);
        reduce(&mut state, Action::ToggleSearchPanel);
        assert!(!state.ui_state.show_search_panel);
    }

    #[test]
    fn close_book_resets_to_empty_library() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        assert!(state.current_book.is_some());
        assert_eq!(state.ui_state.screen, ScreenKind::Reader);

        reduce(&mut state, Action::CloseBook);
        assert!(state.current_book.is_none());
        assert!(state.reading_progress.is_none());
        assert!(state.bookmarks.is_empty());
        assert_eq!(state.ui_state.screen, ScreenKind::EmptyLibrary);
        assert_eq!(state.status_message, "就绪");
    }

    #[test]
    fn reader_setting_changed_font_size() {
        let mut state = AppState::default();
        reduce(&mut state, Action::ReaderSettingChanged("font_size".to_string(), "20.0".to_string()));
        assert!((state.reader_settings.font_size - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reader_setting_changed_font_size_clamp() {
        let mut state = AppState::default();
        reduce(&mut state, Action::ReaderSettingChanged("font_size".to_string(), "1.0".to_string()));
        assert!((state.reader_settings.font_size - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reader_setting_changed_line_height() {
        let mut state = AppState::default();
        reduce(&mut state, Action::ReaderSettingChanged("line_height".to_string(), "2.0".to_string()));
        assert!((state.reader_settings.line_height - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reader_setting_changed_content_width() {
        let mut state = AppState::default();
        reduce(&mut state, Action::ReaderSettingChanged("content_width".to_string(), "800.0".to_string()));
        assert!((state.reader_settings.content_width - 800.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reader_setting_changed_show_toc() {
        let mut state = AppState::default();
        assert!(state.reader_settings.show_toc);
        reduce(&mut state, Action::ReaderSettingChanged("show_toc".to_string(), "false".to_string()));
        assert!(!state.reader_settings.show_toc);
    }

    #[test]
    fn reader_setting_changed_show_status_bar() {
        let mut state = AppState::default();
        assert!(state.reader_settings.show_status_bar);
        reduce(&mut state, Action::ReaderSettingChanged("show_status_bar".to_string(), "false".to_string()));
        assert!(!state.reader_settings.show_status_bar);
    }

    #[test]
    fn reader_setting_changed_unknown_key_is_noop() {
        let mut state = AppState::default();
        let before = state.reader_settings.clone();
        reduce(&mut state, Action::ReaderSettingChanged("unknown_key".to_string(), "123".to_string()));
        assert_eq!(state.reader_settings, before);
    }

    #[test]
    fn restore_default_settings() {
        let mut state = AppState::default();
        state.reader_settings.font_size = 99.0;
        reduce(&mut state, Action::RestoreDefaultSettings);
        assert!((state.reader_settings.font_size - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn update_scroll_offset() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        reduce(&mut state, Action::UpdateScrollOffset(42.5));
        assert_eq!(
            state.reading_progress.as_ref().map(|p| p.scroll_offset),
            Some(42.5)
        );
    }

    #[test]
    fn add_bookmark_from_current_position() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        reduce(&mut state, Action::AddBookmarkRequested);
        assert_eq!(state.bookmarks.len(), 1);
        assert_eq!(state.bookmarks[0].chapter_index, 0);
        assert_eq!(state.bookmarks[0].book_id, "book-1");
        assert_eq!(state.status_message, "已添加书签");
    }

    #[test]
    fn remove_bookmark() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        reduce(&mut state, Action::AddBookmarkRequested);
        let bm_id = state.bookmarks[0].id.clone();
        reduce(&mut state, Action::RemoveBookmark(bm_id));
        assert!(state.bookmarks.is_empty());
    }

    #[test]
    fn jump_to_bookmark() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        reduce(&mut state, Action::GoToChapter(1));
        reduce(&mut state, Action::AddBookmarkRequested);
        let bm_id = state.bookmarks[0].id.clone();
        reduce(&mut state, Action::GoToChapter(0));
        assert_eq!(state.reading_progress.as_ref().map(|p| p.chapter_index), Some(0));

        reduce(&mut state, Action::JumpToBookmark(bm_id));
        assert_eq!(state.reading_progress.as_ref().map(|p| p.chapter_index), Some(1));
    }

    #[test]
    fn search_query_changed() {
        let mut state = AppState::default();
        let query = crate::domain::search_query::SearchQuery {
            keyword: "test".to_string(),
            case_sensitive: false,
            scope: crate::domain::search_enums::SearchScope::CurrentChapter,
        };
        reduce(&mut state, Action::SearchQueryChanged(query));
        assert!(state.search_state.current_query.is_some());
        assert_eq!(state.search_state.current_query.as_ref().unwrap().keyword, "test");
    }

    #[test]
    fn search_submitted_finds_matches() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        let query = crate::domain::search_query::SearchQuery {
            keyword: "Body".to_string(),
            case_sensitive: true,
            scope: crate::domain::search_enums::SearchScope::EntireBook,
        };
        reduce(&mut state, Action::SearchQueryChanged(query));
        reduce(&mut state, Action::SearchSubmitted);
        assert!(!state.search_state.results.is_empty());
    }

    #[test]
    fn search_result_selected_jumps_to_chapter() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        let query = crate::domain::search_query::SearchQuery {
            keyword: "Body 2".to_string(),
            case_sensitive: true,
            scope: crate::domain::search_enums::SearchScope::EntireBook,
        };
        reduce(&mut state, Action::SearchQueryChanged(query));
        reduce(&mut state, Action::SearchSubmitted);
        let result_count = state.search_state.results.len();
        assert!(result_count > 0);

        reduce(&mut state, Action::SearchResultSelected(0));
        assert_eq!(state.search_state.selected_result_index, Some(0));
    }

    #[test]
    fn clear_search_resets_state() {
        let mut state = AppState::default();
        state.search_state.is_searching = true;
        reduce(&mut state, Action::ClearSearch);
        assert!(!state.search_state.is_searching);
        assert!(state.search_state.current_query.is_none());
    }

    #[test]
    fn recent_book_selected_sets_pending_path() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        let book_id = state.current_book.as_ref().unwrap().id.clone();
        state.current_book = None;
        state.ui_state.screen = ScreenKind::EmptyLibrary;

        reduce(&mut state, Action::RecentBookSelected(book_id));
        assert!(state.ui_state.pending_open_path.is_some());
    }

    #[test]
    fn remove_recent_book() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        assert_eq!(state.recent_books.len(), 1);
        reduce(&mut state, Action::RemoveRecentBook("book-1".to_string()));
        assert!(state.recent_books.is_empty());
    }

    #[test]
    fn status_message_timed_out_clears_message() {
        let mut state = AppState::default();
        state.status_message = "测试消息".to_string();
        reduce(&mut state, Action::StatusMessageTimedOut);
        assert_eq!(state.status_message, "就绪");
    }

    #[test]
    fn status_message_timed_out_preserves_error() {
        let mut state = AppState::default();
        state.status_message = "错误消息".to_string();
        state.last_error = Some(crate::domain::app_error::AppError::new("TEST", "error"));
        reduce(&mut state, Action::StatusMessageTimedOut);
        assert_eq!(state.status_message, "错误消息");
    }

    #[test]
    fn add_bookmark_sets_timestamp() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        let before = state.status_message_set_at.clone();
        reduce(&mut state, Action::AddBookmarkRequested);
        assert!(state.status_message_set_at.is_some());
        assert_ne!(state.status_message_set_at, before);
    }

    #[test]
    fn open_book_succeeded_sets_timestamp() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        assert!(state.status_message_set_at.is_some());
        assert!(state.status_message.contains("内容已加载"));
    }

    #[test]
    fn close_book_clears_timestamp() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        assert!(state.status_message_set_at.is_some());
        reduce(&mut state, Action::CloseBook);
        assert!(state.status_message_set_at.is_none());
        assert_eq!(state.status_message, "就绪");
    }

    #[test]
    fn status_message_timed_out_clears_timestamp() {
        let mut state = AppState::default();
        reduce(&mut state, Action::OpenBookSucceeded(sample_book(BookFormat::Txt)));
        assert!(state.status_message_set_at.is_some());
        reduce(&mut state, Action::StatusMessageTimedOut);
        assert!(state.status_message_set_at.is_none());
    }

    #[test]
    fn search_chinese_text_does_not_panic_on_utf8_boundary() {
        let mut state = AppState::default();
        let mut book = sample_book(BookFormat::Txt);
        book.chapters = vec![Chapter {
            id: "ch-cn".to_string(),
            index: 0,
            title: "中文章节".to_string(),
            raw_title: None,
            content: "这是一段很长的中文测试文本，用于验证搜索摘要提取时不会因为UTF-8字符边界问题而崩溃。".to_string(),
            paragraphs: vec![Paragraph {
                index: 0,
                text: "这是一段很长的中文测试文本，用于验证搜索摘要提取时不会因为UTF-8字符边界问题而崩溃。".to_string(),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            }],
            word_count: 40,
            char_count: 40,
            source_href: None,
            anchor: None,
            warnings: Vec::new(),
        }];
        reduce(&mut state, Action::OpenBookSucceeded(book));
        let query = crate::domain::search_query::SearchQuery {
            keyword: "验证".to_string(),
            case_sensitive: true,
            scope: crate::domain::search_enums::SearchScope::EntireBook,
        };
        reduce(&mut state, Action::SearchQueryChanged(query));
        reduce(&mut state, Action::SearchSubmitted);
        assert!(!state.search_state.results.is_empty());
        let snippet = &state.search_state.results[0].snippet;
        assert!(snippet.contains("验证"));
    }
}
