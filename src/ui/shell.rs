use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::app::compat::CompatAdapter;
use crate::domain::enums::ScreenKind;
use crate::ui::panels::left_sidebar::left_sidebar;
use crate::ui::panels::reader_view::reader_view;
use crate::ui::panels::status_bar::status_bar;
use crate::ui::panels::top_bar::TopBar;
use crate::ui::widgets::{empty_state_with_button, error_state, loading_state};
use crate::ui::{ThemeConfig, ThemeService};

pub struct AppShell;

impl AppShell {
    pub fn update(shell: &mut CompatAdapter, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme_kind = shell.state().reader_settings.theme.clone();
        let config = ThemeConfig::from(theme_kind);
        ThemeService::apply_theme(ctx, &config);

        let screen = shell.state().ui_state.screen.clone();

        match screen {
            ScreenKind::EmptyLibrary => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if empty_state_with_button(
                        ui,
                        "Reader Demo",
                        "打开 EPUB 或 TXT 文件开始阅读",
                        "打开书籍",
                        &config,
                    ) {
                        if let Some(path) = FileDialog::new()
                            .add_filter("电子书", &["epub", "txt"])
                            .add_filter("EPUB", &["epub"])
                            .add_filter("文本文件", &["txt"])
                            .pick_file()
                        {
                            let path_str = path.to_str().unwrap_or("").to_string();
                            info!("打开文件: {}", path_str);
                            shell.open_book(&path_str);
                        }
                    }
                });
            }
            ScreenKind::LoadingBook => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    loading_state(ui, "正在加载", "请稍候...", &config);
                });
            }
            ScreenKind::Reader => {
                Self::reader_layout(shell, ctx, &config);
            }
            ScreenKind::Error => {
                let err_msg = shell
                    .state()
                    .last_error
                    .as_ref()
                    .map(|e| format!("[{}] {}", e.code, e.message))
                    .unwrap_or_else(|| "未知错误".to_string());
                egui::CentralPanel::default().show(ctx, |ui| {
                    let (_retry, _reopen) =
                        error_state(ui, "出错了", &err_msg, "重试", "重新打开", &config);
                });
            }
        }
    }

    fn reader_layout(shell: &mut CompatAdapter, ctx: &egui::Context, theme: &ThemeConfig) {
        if let Some(path) = TopBar::take_open_book_path() {
            shell.open_book(&path);
        }

        let current_page = shell.current_page();
        let content_len = shell.content().len();
        let status = shell.status().to_owned();
        let chapters = shell
            .state()
            .current_book
            .as_ref()
            .map(|b| b.chapters.clone())
            .unwrap_or_default();
        let settings = shell.state().reader_settings.clone();

        let mut page = current_page;
        let mut active_tab = shell.state().ui_state.left_panel_tab.clone();

        // Left sidebar
        if settings.show_toc {
            let toc = shell
                .state()
                .current_book
                .as_ref()
                .map(|b| b.toc.clone())
                .unwrap_or_default();
            left_sidebar(ctx, &mut active_tab, &toc, theme);
        }

        // Top bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            TopBar::show(ui, &status, content_len, &mut page, theme);
        });

        // Reader content
        egui::CentralPanel::default().show(ctx, |ui| {
            reader_view(ui, &chapters, page, &settings, theme);
        });

        // Status bar
        if settings.show_status_bar {
            let progress = shell
                .state()
                .reading_progress
                .as_ref()
                .map(|p| p.progress_percent)
                .unwrap_or(0.0);
            let chapter_pos = if content_len > 0 {
                format!("{}/{}", page + 1, content_len)
            } else {
                String::new()
            };
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                status_bar(ui, progress, &chapter_pos, &shell.state().status_message, theme);
            });
        }

        // Sync back
        if page != current_page {
            shell.set_current_page(page);
        }
    }
}
