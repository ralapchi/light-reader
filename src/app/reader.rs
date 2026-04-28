use eframe::egui;

use crate::app::{Action, compat::CompatAdapter};
use crate::domain::chapter::Chapter;
use crate::domain::reader_settings::ReaderSettings;
use crate::domain::toc_item::TocItem;
use crate::ui::{ContentViewer, TableOfContents, ThemeConfig, ThemeService, Toolbar};

pub struct ReaderApp {
    pub adapter: CompatAdapter,
}

impl ReaderApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ThemeService::init_fonts(&cc.egui_ctx);
        Self {
            adapter: CompatAdapter::new(),
        }
    }

    fn current_theme(&self) -> ThemeConfig {
        let kind = &self.adapter.state().reader_settings.theme;
        ThemeConfig::from(kind.clone())
    }
}

impl Default for ReaderApp {
    fn default() -> Self {
        Self {
            adapter: CompatAdapter::new(),
        }
    }
}

impl eframe::App for ReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme = self.current_theme();
        ThemeService::apply_theme(ctx, &theme);

        let current_page = self.adapter.current_page();
        let mut page = current_page;
        let mut pending_open_book = None;

        {
            let state = self.adapter.state();
            let settings: &ReaderSettings = &state.reader_settings;
            let book = state.current_book.as_ref();
            let toc: &[TocItem] = book.map(|book| book.toc.as_slice()).unwrap_or(&[]);
            let chapters: &[Chapter] = book.map(|book| book.chapters.as_slice()).unwrap_or(&[]);
            let content_len = chapters.len();

            if settings.show_toc {
                TableOfContents::show(ctx, toc, &mut page, &theme);
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                Toolbar::show(ui, &state.status_message, content_len, &mut page, &theme);
                pending_open_book = Toolbar::take_open_book_path();
                ui.separator();
                ContentViewer::show(ui, chapters, page, settings, &theme);
            });
        }

        if let Some(path) = pending_open_book {
            self.adapter.dispatch(Action::OpenBookSelected(path));
        }

        if page != current_page {
            self.adapter.dispatch(Action::GoToChapter(page));
        }
    }
}
