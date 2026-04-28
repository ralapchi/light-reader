use eframe::egui;

use crate::app::compat::CompatAdapter;
use crate::ui::{ContentViewer, TableOfContents, ThemeConfig, ThemeService, Toolbar};

pub struct ReaderApp {
    pub adapter: CompatAdapter,
}

impl Default for ReaderApp {
    fn default() -> Self {
        Self {
            adapter: CompatAdapter::new(),
        }
    }
}

impl ReaderApp {
    fn current_theme(&self) -> ThemeConfig {
        let kind = &self.adapter.state().reader_settings.theme;
        ThemeConfig::from(kind.clone())
    }
}

impl eframe::App for ReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ThemeService::init_fonts(ctx);

        let theme = self.current_theme();
        ThemeService::apply_theme(ctx, &theme);

        let titles = self.adapter.chapter_titles().to_vec();
        let mut page = self.adapter.current_page();

        TableOfContents::show(ctx, &titles, &mut page, &theme);

        egui::CentralPanel::default().show(ctx, |ui| {
            let status = self.adapter.status().to_owned();
            let content_len = self.adapter.content().len();

            Toolbar::show(ui, &status, content_len, &mut page, &theme);

            if let Some(path) = Toolbar::take_open_book_path() {
                self.adapter.open_book(&path);
            }

            ui.separator();

            let content = self.adapter.content().to_vec();
            ContentViewer::show(ui, &content, page, &theme, None, None);
        });

        if page != self.adapter.current_page() {
            self.adapter.set_current_page(page);
        }
    }
}
