use serde::{Deserialize, Serialize};

use crate::domain::core_state::CoreState;
use crate::domain::frontend_ui_state::FrontendUiState;
use crate::domain::library_view_state::LibraryViewState;
use crate::domain::session_state::SessionState;
use crate::domain::ui_state::UiState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub core: CoreState,
    pub session: SessionState,
    pub frontend_ui: FrontendUiState,
    pub ui_state: UiState,
    pub library_view_state: LibraryViewState,
    pub session_started_at: Option<String>,
    pub total_read_seconds_at_session_start: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            core: CoreState::default(),
            session: SessionState::default(),
            frontend_ui: FrontendUiState::default(),
            ui_state: UiState::default(),
            library_view_state: LibraryViewState::default(),
            session_started_at: None,
            total_read_seconds_at_session_start: 0,
        }
    }
}
