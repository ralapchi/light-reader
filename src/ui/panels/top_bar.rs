use std::sync::Mutex;

use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::ui::ThemeConfig;

static OPEN_BOOK_PATH: Mutex<Option<String>> = Mutex::new(None);

pub struct TopBar;

impl TopBar {
    pub fn show(
        ui: &mut egui::Ui,
        status: &str,
        content_len: usize,
        current_page: &mut usize,
        theme: &ThemeConfig,
    ) {
        let s = &theme.spacing;

        ui.horizontal(|ui| {
            ui.add_space(s.sm);

            if ui.button("打开书籍").clicked() {
                info!("点击了打开书籍按钮");
                if let Some(path) = FileDialog::new()
                    .add_filter("电子书", &["epub", "txt"])
                    .add_filter("EPUB", &["epub"])
                    .add_filter("文本文件", &["txt"])
                    .pick_file()
                {
                    let path_str = path.to_str().unwrap_or("").to_string();
                    *OPEN_BOOK_PATH.lock().unwrap() = Some(path_str);
                }
            }

            ui.add_space(s.lg);
            ui.separator();
            ui.add_space(s.lg);

            let prev_enabled = *current_page > 0;
            ui.add_enabled_ui(prev_enabled, |ui| {
                if ui.button("上一章").clicked() && *current_page > 0 {
                    *current_page -= 1;
                }
            });

            ui.add_space(s.sm);

            if content_len > 0 {
                ui.label(format!("{} / {}", *current_page + 1, content_len));
            }

            ui.add_space(s.sm);

            let next_enabled = *current_page < content_len.saturating_sub(1);
            ui.add_enabled_ui(next_enabled, |ui| {
                if ui.button("下一章").clicked() && *current_page < content_len.saturating_sub(1) {
                    *current_page += 1;
                }
            });

            ui.add_space(s.lg);
            ui.separator();
            ui.add_space(s.lg);

            ui.label(status);

            ui.add_space(s.sm);
        });
    }

    pub fn take_open_book_path() -> Option<String> {
        OPEN_BOOK_PATH.lock().unwrap().take()
    }
}
