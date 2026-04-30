use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::app::compat::CompatAdapter;
use crate::app::Action;
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
                            shell.dispatch(Action::OpenBookSelected(path_str));
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
                let last_path = shell.state().ui_state.last_attempted_path.clone();
                egui::CentralPanel::default().show(ctx, |ui| {
                    let (retry, reopen) =
                        error_state(ui, "出错了", &err_msg, "重试", "重新打开", &config);
                    if retry {
                        if let Some(path) = &last_path {
                            let path_str = path.to_string_lossy().to_string();
                            shell.dispatch(Action::OpenBookSelected(path_str));
                        }
                    }
                    if reopen {
                        if let Some(path) = FileDialog::new()
                            .add_filter("电子书", &["epub", "txt"])
                            .pick_file()
                        {
                            let path_str = path.to_string_lossy().to_string();
                            shell.dispatch(Action::OpenBookSelected(path_str));
                        }
                    }
                });
            }
        }
    }

    fn reader_layout(shell: &mut CompatAdapter, ctx: &egui::Context, theme: &ThemeConfig) {
        // Clone state upfront to avoid borrow conflicts during UI rendering
        let state_snapshot = shell.state().clone();
        let chapters = state_snapshot
            .current_book
            .as_ref()
            .map(|b| b.chapters.clone())
            .unwrap_or_default();
        let toc = state_snapshot
            .current_book
            .as_ref()
            .map(|b| b.toc.clone())
            .unwrap_or_default();
        let settings = &state_snapshot.reader_settings;
        let active_tab = &state_snapshot.ui_state.left_panel_tab;

        let mut pending_actions: Vec<Action> = Vec::new();

        // Left sidebar
        if settings.show_toc {
            if let Some(action) = left_sidebar(
                ctx,
                active_tab,
                &toc,
                &state_snapshot.bookmarks,
                &state_snapshot.recent_books,
                theme,
            ) {
                pending_actions.push(action);
            }
        }

        // Top bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            let actions = TopBar::show(ui, &state_snapshot, theme);
            pending_actions.extend(actions);
        });

        // Reader content
        let current_page = state_snapshot
            .reading_progress
            .as_ref()
            .map(|p| p.chapter_index)
            .unwrap_or(0);
        egui::CentralPanel::default().show(ctx, |ui| {
            let actions = reader_view(ui, &chapters, current_page, settings, theme);
            pending_actions.extend(actions);
        });

        // Status bar
        if settings.show_status_bar {
            let progress = state_snapshot
                .reading_progress
                .as_ref()
                .map(|p| p.progress_percent)
                .unwrap_or(0.0);
            let content_len = chapters.len();
            let chapter_pos = if content_len > 0 {
                format!("{}/{}", current_page + 1, content_len)
            } else {
                String::new()
            };
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                status_bar(ui, progress, &chapter_pos, &state_snapshot.status_message, theme);
            });
        }

        // Dispatch all collected actions
        for action in pending_actions {
            shell.dispatch(action);
        }
    }
}
