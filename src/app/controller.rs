use std::path::PathBuf;

use crate::app::Action;
use crate::app::compat::CompatAdapter;
use crate::app::reducer;
use crate::domain::enums::ScreenKind;

pub fn dispatch(adapter: &mut CompatAdapter, action: Action) {
    match action {
        Action::OpenBookSelected(path) => {
            {
                let state = adapter.state_mut();
                state.status_message = format!("正在打开文件: {}", path);
                state.last_error = None;
                state.ui_state.is_loading = true;
                state.ui_state.screen = ScreenKind::LoadingBook;
                state.ui_state.pending_open_path = Some(PathBuf::from(&path));
                state.ui_state.last_attempted_path = Some(PathBuf::from(&path));
            }

            match adapter.try_load_book(&path) {
                Ok(book) => reducer::reduce(adapter.state_mut(), Action::OpenBookSucceeded(book)),
                Err(err) => reducer::reduce(adapter.state_mut(), Action::OpenBookFailed(err)),
            }
        }
        other => reducer::reduce(adapter.state_mut(), other),
    }
}
