use eframe::egui;

use crate::app::compat::CompatAdapter;
use crate::ui::AppShell;
use crate::ui::ThemeService;

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
}

impl Default for ReaderApp {
    fn default() -> Self {
        Self {
            adapter: CompatAdapter::new(),
        }
    }
}

impl eframe::App for ReaderApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        AppShell::update(&mut self.adapter, ctx, frame);
    }
}
