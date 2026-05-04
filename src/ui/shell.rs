use chrono::Utc;
use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::app::compat::CompatAdapter;
use crate::app::Action;
use crate::domain::enums::ScreenKind;
use crate::ui::panels::left_sidebar::left_sidebar;
use crate::ui::panels::reader_view::reader_view;
use crate::ui::panels::search_panel::search_panel;
use crate::ui::panels::settings_panel::settings_panel;
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

        // Auto-clear status messages after 3 seconds
        if let Some(ref set_at) = shell.state().status_message_set_at {
            if let Ok(set_time) = chrono::DateTime::parse_from_rfc3339(set_at) {
                let elapsed = Utc::now().signed_duration_since(set_time);
                if elapsed.num_seconds() >= 3 {
                    shell.dispatch(Action::StatusMessageTimedOut);
                }
            }
        }

        // Keyboard shortcuts
        let shortcuts = Self::collect_shortcuts(ctx);
        for action in shortcuts {
            shell.dispatch(action);
        }

        let screen = shell.state().ui_state.screen.clone();

        match screen {
            ScreenKind::EmptyLibrary => {
                let recent_books = shell.state().recent_books.clone();
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

                    // Show recent books if available
                    if !recent_books.is_empty() {
                        let s = &config.spacing;
                        ui.add_space(s.lg);
                        ui.separator();
                        ui.add_space(s.md);

                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("最近阅读")
                                    .size(config.typography.body_size)
                                    .strong()
                                    .color(config.colors.text_secondary.to_color32()),
                            );
                            ui.add_space(s.sm);

                            for item in recent_books.iter().take(5) {
                                if item.is_missing {
                                    continue;
                                }
                                let label = if let Some(author) = &item.author {
                                    format!("{} - {}", item.title, author)
                                } else {
                                    item.title.clone()
                                };
                                let btn = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(&label)
                                            .size(config.typography.body_size),
                                    )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE),
                                );
                                if btn.clicked() {
                                    shell.dispatch(Action::RecentBookSelected(item.book_id.clone()));
                                }
                                btn.on_hover_text(&item.source_path);
                            }
                        });
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

    fn collect_shortcuts(ctx: &egui::Context) -> Vec<Action> {
        use crate::domain::theme_kind::ThemeKind;

        let mut actions = Vec::new();

        ctx.input(|i| {
            let ctrl_or_cmd = i.modifiers.command || i.modifiers.ctrl;

            // Ctrl/Cmd + O → 打开书籍
            if ctrl_or_cmd && i.key_pressed(egui::Key::O) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("电子书", &["epub", "txt"])
                    .pick_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    actions.push(Action::OpenBookSelected(path_str));
                }
            }

            // Ctrl/Cmd + F → 打开搜索
            if ctrl_or_cmd && i.key_pressed(egui::Key::F) {
                actions.push(Action::ToggleSearchPanel);
            }

            // Ctrl/Cmd + , → 打开设置
            if ctrl_or_cmd && i.key_pressed(egui::Key::Comma) {
                actions.push(Action::ToggleSettingsPanel);
            }

            // Ctrl/Cmd + B → 添加书签
            if ctrl_or_cmd && i.key_pressed(egui::Key::B) {
                actions.push(Action::AddBookmarkRequested);
            }

            // Left / PageUp → 上一章
            if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::PageUp) {
                actions.push(Action::PrevChapter);
            }

            // Right / PageDown → 下一章
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::PageDown) {
                actions.push(Action::NextChapter);
            }

            // Esc → 关闭搜索或设置面板
            if i.key_pressed(egui::Key::Escape) {
                actions.push(Action::CloseSearchOrSettings);
            }

            // Ctrl/Cmd + 1/2/3 → 主题切换
            if ctrl_or_cmd && i.key_pressed(egui::Key::Num1) {
                actions.push(Action::ThemeChanged(ThemeKind::Light));
            }
            if ctrl_or_cmd && i.key_pressed(egui::Key::Num2) {
                actions.push(Action::ThemeChanged(ThemeKind::Dark));
            }
            if ctrl_or_cmd && i.key_pressed(egui::Key::Num3) {
                actions.push(Action::ThemeChanged(ThemeKind::Sepia));
            }
        });

        actions
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
        if settings.show_toc && !state_snapshot.ui_state.sidebar_collapsed {
            if let Some(action) = left_sidebar(
                ctx,
                active_tab,
                &toc,
                &state_snapshot.bookmarks,
                &state_snapshot.recent_books,
                theme,
                settings.toc_width,
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
        let selected_search_result = state_snapshot
            .search_state
            .selected_result_index
            .and_then(|idx| state_snapshot.search_state.results.get(idx));
        let search_keyword = state_snapshot
            .search_state
            .current_query
            .as_ref()
            .map(|q| q.keyword.as_str());
        let status_message = state_snapshot.status_message.clone();
        egui::CentralPanel::default().show(ctx, |ui| {
            let actions = reader_view(ui, &chapters, current_page, settings, theme, selected_search_result, search_keyword, &status_message);
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
            let char_count = chapters
                .get(current_page)
                .map(|c| c.char_count)
                .unwrap_or(0);
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                status_bar(ui, progress, &chapter_pos, &state_snapshot.status_message, char_count, theme);
            });
        }

        // Search panel (overlay on right side)
        if state_snapshot.ui_state.show_search_panel {
            let actions = search_panel(ctx, &state_snapshot, theme);
            pending_actions.extend(actions);
        }

        // Settings panel (overlay on right side)
        if state_snapshot.ui_state.show_settings_panel {
            let actions = settings_panel(ctx, &state_snapshot, theme);
            pending_actions.extend(actions);
        }

        // Dispatch all collected actions
        for action in pending_actions {
            shell.dispatch(action);
        }
    }
}
