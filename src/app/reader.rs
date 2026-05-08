use eframe::egui;

use crate::app::compat::CompatAdapter;
use crate::ui::AppShell;
use crate::ui::ThemeService;

pub struct ReaderApp {
    pub adapter: CompatAdapter,
    last_window_size: Option<(f32, f32)>,
    last_window_pos: Option<(f32, f32)>,
    startup_book_path: Option<String>,
    was_focused: bool,
}

impl ReaderApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ThemeService::init_fonts(&cc.egui_ctx);
        let adapter = CompatAdapter::new();
        let startup_book_path = Self::resolve_startup_book(&adapter);
        Self {
            adapter,
            last_window_size: None,
            last_window_pos: None,
            startup_book_path,
            was_focused: true,
        }
    }

    fn resolve_startup_book(adapter: &CompatAdapter) -> Option<String> {
        let state = adapter.state();
        if !state.reader_settings.open_last_book_on_startup {
            return None;
        }
        let settings_file = crate::storage::settings_store::load();
        let last_id = settings_file.last_opened_book_id?;
        let item = state.recent_books.iter().find(|r| r.book_id == last_id)?;
        if item.is_missing {
            return None;
        }
        Some(item.source_path.clone())
    }
}

impl Default for ReaderApp {
    fn default() -> Self {
        let adapter = CompatAdapter::new();
        let startup_book_path = Self::resolve_startup_book(&adapter);
        Self {
            adapter,
            last_window_size: None,
            last_window_pos: None,
            startup_book_path,
            was_focused: true,
        }
    }
}

impl eframe::App for ReaderApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Auto-open last book on startup
        if let Some(path) = self.startup_book_path.take() {
            self.adapter.dispatch(crate::app::Action::OpenBookSelected(path));
        }

        // Track window size and position
        ctx.input(|i| {
            if let Some(rect) = i.viewport().inner_rect {
                let size = (rect.width(), rect.height());
                if self.last_window_size != Some(size) {
                    self.last_window_size = Some(size);
                    self.adapter.state_mut().window_size = Some(size);
                }
            }
            if let Some(rect) = i.viewport().outer_rect {
                let pos = (rect.min.x, rect.min.y);
                if self.last_window_pos != Some(pos) {
                    self.last_window_pos = Some(pos);
                    self.adapter.state_mut().window_pos = Some(pos);
                }
            }
        });

        // Save progress on focus lost
        let focused = ctx.input(|i| i.focused);
        if self.was_focused && !focused {
            self.adapter.save_persisted_state();
        }
        self.was_focused = focused;

        // Poll TTS thread results before rendering
        self.adapter.poll_tts_results();

        AppShell::update(&mut self.adapter, ctx, frame);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.adapter.save_persisted_state();
    }
}
