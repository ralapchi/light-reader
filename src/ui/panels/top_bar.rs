use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::app::Action;
use crate::ui::ThemeConfig;

pub struct TopBarProps {
    pub sidebar_collapsed: bool,
    pub chapter_index: usize,
    pub total_chapters: usize,
}

pub struct TopBar;

impl TopBar {
    pub fn show(ui: &mut egui::Ui, props: &TopBarProps, theme: &ThemeConfig) -> Vec<Action> {
        let s = &theme.spacing;
        let mut actions = Vec::new();

        let current_page = props.chapter_index;
        let content_len = props.total_chapters;

        let total_width = ui.available_width();

        ui.horizontal(|ui| {
            // === Left section: sidebar toggle, open book ===
            ui.add_space(s.sm);

            let sidebar_open = !props.sidebar_collapsed;
            let sidebar_label = if sidebar_open { "✕" } else { "☰" };
            let sidebar_btn = egui::Button::new(sidebar_label)
                .min_size(egui::vec2(32.0, 24.0))
                .selected(sidebar_open);
            if ui.add(sidebar_btn).clicked() {
                actions.push(Action::ToggleSidebar);
            }

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
                    actions.push(Action::OpenBookSelected(path_str));
                }
            }

            // === Center section: chapter navigation ===
            let left_width = ui.next_widget_position().x;
            let center_target = total_width / 2.0;
            let center_pad = (center_target - left_width - 80.0).max(0.0);
            ui.add_space(center_pad);

            let prev_enabled = current_page > 0;
            ui.add_enabled_ui(prev_enabled, |ui| {
                if ui.button("上一章").clicked() {
                    actions.push(Action::PrevChapter);
                }
            });

            ui.add_space(s.sm);

            if content_len > 0 {
                ui.label(format!("{} / {}", current_page + 1, content_len));
            }

            ui.add_space(s.sm);

            let next_enabled = current_page < content_len.saturating_sub(1);
            ui.add_enabled_ui(next_enabled, |ui| {
                if ui.button("下一章").clicked() {
                    actions.push(Action::NextChapter);
                }
            });

            // === Right section: search, bookmark, settings ===
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(s.sm);

                if ui.button("设置").clicked() {
                    actions.push(Action::ToggleSettingsPanel);
                }

                ui.add_space(s.sm);

                if ui.button("书签").clicked() {
                    actions.push(Action::AddBookmarkRequested);
                }

                ui.add_space(s.sm);

                if ui.button("搜索").clicked() {
                    actions.push(Action::ToggleSearchPanel);
                }
            });
        });

        actions
    }
}
