use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::app::Action;
use crate::domain::app_state::AppState;
use crate::domain::enums::LeftPanelTab;
use crate::ui::ThemeConfig;

pub struct TopBar;

impl TopBar {
    pub fn show(ui: &mut egui::Ui, state: &AppState, theme: &ThemeConfig) -> Vec<Action> {
        let s = &theme.spacing;
        let mut actions = Vec::new();

        let current_page = state
            .reading_progress
            .as_ref()
            .map(|p| p.chapter_index)
            .unwrap_or(0);
        let content_len = state
            .current_book
            .as_ref()
            .map(|b| b.chapters.len())
            .unwrap_or(0);

        ui.horizontal(|ui| {
            ui.add_space(s.sm);

            // SidebarToggleButton
            let sidebar_open = !state.ui_state.sidebar_collapsed;
            let sidebar_label = if sidebar_open { "☰" } else { "☰" };
            let sidebar_btn = egui::Button::new(sidebar_label)
                .min_size(egui::vec2(32.0, 24.0))
                .selected(sidebar_open);
            if ui.add(sidebar_btn).clicked() {
                actions.push(Action::ToggleSidebar);
            }

            ui.add_space(s.sm);

            // OpenBookButton
            if ui.button("打开书籍").clicked() {
                info!("点击了打开书籍按钮");
                if let Some(path) = FileDialog::new()
                    .add_filter("电子书", &["epub", "txt"])
                    .add_filter("EPUB", &["epub"])
                    .add_filter("文本文件", &["txt"])
                    .pick_file()
                {
                    let path_str = path.to_str().unwrap_or("").to_string();
                    actions.push(Action::OpenBookSelected(path_str));
                }
            }

            ui.add_space(s.sm);

            // RecentButton
            if ui.button("最近").clicked() {
                actions.push(Action::SwitchLeftPanelTab(LeftPanelTab::Recent));
            }

            ui.add_space(s.sm);

            // SearchButton
            if ui.button("搜索").clicked() {
                actions.push(Action::ToggleSearchPanel);
            }

            ui.add_space(s.lg);
            ui.separator();
            ui.add_space(s.lg);

            // PrevChapterButton
            let prev_enabled = current_page > 0;
            ui.add_enabled_ui(prev_enabled, |ui| {
                if ui.button("上一章").clicked() {
                    actions.push(Action::PrevChapter);
                }
            });

            ui.add_space(s.sm);

            // ChapterProgressLabel
            if content_len > 0 {
                ui.label(format!("{} / {}", current_page + 1, content_len));
            }

            ui.add_space(s.sm);

            // NextChapterButton
            let next_enabled = current_page < content_len.saturating_sub(1);
            ui.add_enabled_ui(next_enabled, |ui| {
                if ui.button("下一章").clicked() {
                    actions.push(Action::NextChapter);
                }
            });

            ui.add_space(s.lg);
            ui.separator();
            ui.add_space(s.lg);

            // BookmarkButton
            if ui.button("书签").clicked() {
                actions.push(Action::AddBookmarkRequested);
            }

            ui.add_space(s.sm);

            // SettingsButton
            if ui.button("设置").clicked() {
                actions.push(Action::ToggleSettingsPanel);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(s.sm);
                ui.label(&state.status_message);
            });
        });

        actions
    }
}
