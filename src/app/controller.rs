use std::path::PathBuf;

use chrono::Utc;

use crate::app::Action;
use crate::app::compat::CompatAdapter;
use crate::app::reducer;
use crate::domain::enums::ScreenKind;
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
        Action::RemoveRecentBook(_) => {
            reducer::reduce(adapter.state_mut(), action);
            save_recent(adapter);
        }
        Action::CloseBook => {
            save_progress(adapter);
            save_bookmarks(adapter);
            reducer::reduce(adapter.state_mut(), action);
        }
        Action::ThemeChanged(_)
        | Action::ReaderSettingChanged(_, _)
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

    if let Some(progress) = storage::progress_store::load(&book_id) {
        adapter.state_mut().reading_progress = Some(progress);
    }

    let bookmarks = storage::bookmark_store::load(&book_id);
    if !bookmarks.is_empty() {
        adapter.state_mut().bookmarks = bookmarks;
    }

    save_recent(adapter);
}

fn save_progress(adapter: &CompatAdapter) {
    let state = adapter.state();
    if let (Some(book), Some(progress)) = (&state.current_book, &state.reading_progress) {
        if let Err(e) = storage::progress_store::save(&book.id, progress) {
            log::warn!("保存阅读进度失败: {}", e);
        }
    }
}

fn save_bookmarks(adapter: &CompatAdapter) {
    let state = adapter.state();
    if let Some(book) = &state.current_book {
        if let Err(e) = storage::bookmark_store::save(&book.id, &state.bookmarks) {
            log::warn!("保存书签失败: {}", e);
        }
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
