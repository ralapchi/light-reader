use eframe::egui;

use crate::app::Action;
use crate::ui::ThemeConfig;

pub struct TopBarProps<'a> {
    pub chapter_index: usize,
    pub total_chapters: usize,
    pub floating_toc_open: bool,
    pub status_message: &'a str,
}

pub struct TopBar;

impl TopBar {
    pub fn show(ui: &mut egui::Ui, props: &TopBarProps, theme: &ThemeConfig) -> Vec<Action> {
        let s = &theme.spacing;
        let colors = &theme.colors;
        let mut actions = Vec::new();

        let current_page = props.chapter_index;
        let content_len = props.total_chapters;

        let total_width = ui.available_width();

        // Semi-transparent background fill
        let bar_y = ui.next_widget_position().y;
        let bg_rect = egui::Rect::from_min_size(
            egui::pos2(0.0, bar_y),
            egui::vec2(total_width, theme.panel.top_bar_height),
        );
        ui.painter().rect_filled(
            bg_rect,
            egui::CornerRadius::same(0),
            colors.panel_bg.to_color32().gamma_multiply(0.92),
        );

        ui.horizontal(|ui| {
            // === Left section: floating toc toggle, open book ===
            ui.add_space(s.md);

            // Floating TOC toggle — replaces old sidebar
            let toc_btn = egui::Button::new("目录")
                .min_size(egui::vec2(s.xl, s.lg))
                .selected(props.floating_toc_open);
            if ui.add(toc_btn).clicked() {
                actions.push(Action::ToggleFloatingToc);
            }

            ui.add_space(s.md);

            // Book home button (navigate back to library)
            if ui.button("书库").clicked() {
                actions.push(Action::OpenLibraryHome);
            }

            ui.add_space(s.md);

            if ui.button("听书").clicked() {
                actions.push(Action::StartTts);
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

            // === Right section: toc, search, bookmark, settings ===
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Status message (shown here only when status bar is hidden)
                if !props.status_message.is_empty() {
                    ui.label(
                        egui::RichText::new(props.status_message)
                            .size(theme.typography.caption_size)
                            .color(colors.text_secondary.to_color32()),
                    );
                    ui.add_space(s.md);
                }

                if ui.button("设置").clicked() {
                    actions.push(Action::ToggleSettingsPanel);
                }

                ui.add_space(s.md);

                if ui.button("书签").clicked() {
                    actions.push(Action::AddBookmarkRequested);
                }

                ui.add_space(s.md);

                if ui.button("搜索").clicked() {
                    actions.push(Action::ToggleSearchPanel);
                }
            });
        });

        // Bottom border line
        let border_y = bar_y + theme.panel.top_bar_height;
        let border_rect = egui::Rect::from_min_size(
            egui::pos2(0.0, border_y),
            egui::vec2(total_width, 1.0),
        );
        ui.painter().rect_filled(
            border_rect,
            egui::CornerRadius::same(0),
            colors.border_subtle.to_color32(),
        );

        actions
    }
}
