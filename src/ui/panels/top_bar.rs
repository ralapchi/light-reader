use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::app::Action;
use crate::domain::app_state::AppState;
use crate::domain::enums::LeftPanelTab;
use crate::domain::theme_kind::ThemeKind;
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

            // ThemeSwitcher - cycles Light -> Dark -> Sepia -> Light
            let current_theme = &state.reader_settings.theme;
            let theme_label = match current_theme {
                ThemeKind::Light => "浅色",
                ThemeKind::Dark => "深色",
                ThemeKind::Sepia => "护眼",
                ThemeKind::Paper => "纸张",
                ThemeKind::Custom => "自定义",
            };
            if ui.button(theme_label).clicked() {
                let next_theme = match current_theme {
                    ThemeKind::Light => ThemeKind::Dark,
                    ThemeKind::Dark => ThemeKind::Sepia,
                    ThemeKind::Sepia => ThemeKind::Light,
                    _ => ThemeKind::Light,
                };
                actions.push(Action::ThemeChanged(next_theme));
            }

            ui.add_space(s.sm);

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
