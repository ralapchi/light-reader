use eframe::egui;

use crate::ui::ThemeConfig;

pub struct ContentViewer;

impl ContentViewer {
    pub fn show(
        ui: &mut egui::Ui,
        content: &[String],
        current_page: usize,
        theme: &ThemeConfig,
        content_width: Option<f32>,
        side_margin: Option<f32>,
    ) {
        let spacing = &theme.spacing;

        let max_width = content_width.unwrap_or(theme.panel.content_max_width);
        let margin = side_margin.unwrap_or(spacing.lg * 2.0);
        let available_width = ui.available_width();
        let cw = available_width.min(max_width);
        let sm = (available_width - cw) / 2.0;

        egui::ScrollArea::vertical()
            .id_salt("content_scroll_area")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(sm.max(margin));

                    ui.vertical(|ui| {
                        ui.set_width(cw);

                        ui.add_space(spacing.xl);

                        if let Some(chapter_content) = content.get(current_page) {
                            for paragraph in chapter_content.split("\n\n") {
                                if !paragraph.trim().is_empty() {
                                    let trimmed = paragraph.trim();

                                    let is_title = (trimmed.starts_with("第")
                                        && (trimmed.contains("章")
                                            || trimmed.contains("节")
                                            || trimmed.contains("卷"))
                                        && trimmed.chars().count() < 50)
                                        || (trimmed.starts_with("Chapter") && trimmed.len() < 50);

                                    if is_title {
                                        ui.vertical_centered(|ui| {
                                            ui.label(egui::RichText::new(trimmed).strong());
                                        });
                                    } else {
                                        let indented_text = format!("{}", trimmed);
                                        ui.label(indented_text);
                                    }
                                }
                            }

                            ui.add_space(spacing.xl * 2.0);
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(theme.spacing.xl * 4.0);
                                ui.label("请打开 EPUB 或 TXT 文件开始阅读");
                            });
                        }
                    });

                    ui.add_space(sm.max(margin));
                });
            });
    }
}
