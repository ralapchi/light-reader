use eframe::egui;

use crate::domain::chapter::Chapter;
use crate::domain::paragraph::Paragraph;
use crate::domain::paragraph_kind::ParagraphKind;
use crate::domain::reader_settings::ReaderSettings;
use crate::ui::ThemeConfig;
use crate::ui::theme::ThemeSpacing;

pub struct ContentViewer;

impl ContentViewer {
    pub fn show(
        ui: &mut egui::Ui,
        chapters: &[Chapter],
        current_page: usize,
        settings: &ReaderSettings,
        theme: &ThemeConfig,
    ) {
        let spacing = &theme.spacing;
        let layout = ContentLayout::from_settings(ui.available_width(), settings);
        let available_width = ui.available_width();
        let content_width = available_width.min(layout.content_width);
        let side_margin = (available_width - content_width) / 2.0;

        egui::ScrollArea::vertical()
            .id_salt("content_scroll_area")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(side_margin.max(layout.side_margin));

                    ui.vertical(|ui| {
                        ui.set_width(content_width);
                        ui.add_space(spacing.reader_top_padding);

                        if let Some(chapter) = chapters.get(current_page) {
                            for paragraph in &chapter.paragraphs {
                                render_paragraph(ui, paragraph, &layout, spacing);
                            }

                            if chapter.paragraphs.is_empty() && !chapter.content.is_empty() {
                                ui.label(
                                    egui::RichText::new(&chapter.content)
                                        .size(layout.body_font_size)
                                        .line_height(Some(layout.line_height)),
                                );
                            }

                            ui.add_space(layout.paragraph_spacing + spacing.xl);
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(theme.spacing.xl * 4.0);
                                ui.label("请打开 EPUB 或 TXT 文件开始阅读");
                            });
                        }
                    });

                    ui.add_space(side_margin.max(layout.side_margin));
                });
            });
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ContentLayout {
    content_width: f32,
    side_margin: f32,
    body_font_size: f32,
    title_font_size: f32,
    subtitle_font_size: f32,
    line_height: f32,
    paragraph_spacing: f32,
}

impl ContentLayout {
    fn from_settings(available_width: f32, settings: &ReaderSettings) -> Self {
        Self {
            content_width: settings.content_width.min(available_width.max(0.0)),
            side_margin: settings.side_margin.max(0.0),
            body_font_size: settings.font_size.max(1.0),
            title_font_size: (settings.font_size * 1.5).max(settings.font_size),
            subtitle_font_size: (settings.font_size * 1.15).max(settings.font_size),
            line_height: (settings.font_size * settings.line_height).max(settings.font_size),
            paragraph_spacing: settings.paragraph_spacing.max(0.0),
        }
    }
}

fn render_paragraph(
    ui: &mut egui::Ui,
    paragraph: &Paragraph,
    layout: &ContentLayout,
    spacing: &ThemeSpacing,
) {
    match paragraph.kind {
        ParagraphKind::Title => {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(&paragraph.text)
                        .size(layout.title_font_size)
                        .strong()
                        .line_height(Some(layout.line_height * 1.15)),
                );
            });
            ui.add_space(layout.paragraph_spacing.max(spacing.sm));
        }
        ParagraphKind::Subtitle => {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(&paragraph.text)
                        .size(layout.subtitle_font_size)
                        .weak()
                        .line_height(Some(layout.line_height)),
                );
            });
            ui.add_space(layout.paragraph_spacing.max(spacing.xs));
        }
        ParagraphKind::Quote => {
            ui.label(
                egui::RichText::new(&paragraph.text)
                    .size(layout.body_font_size)
                    .italics()
                    .line_height(Some(layout.line_height)),
            );
            ui.add_space(layout.paragraph_spacing);
        }
        ParagraphKind::Separator => {
            ui.add_space(spacing.md);
            ui.separator();
            ui.add_space(spacing.md);
        }
        ParagraphKind::Body => {
            if paragraph.indent_level > 0 {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(paragraph.indent_level as f32 * spacing.lg);
                    ui.label(
                        egui::RichText::new(&paragraph.text)
                            .size(layout.body_font_size)
                            .line_height(Some(layout.line_height)),
                    );
                });
            } else {
                ui.label(
                    egui::RichText::new(&paragraph.text)
                        .size(layout.body_font_size)
                        .line_height(Some(layout.line_height)),
                );
            }
            ui.add_space(layout.paragraph_spacing);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::reader_settings::ReaderSettings;

    #[test]
    fn content_layout_uses_reader_settings() {
        let mut settings = ReaderSettings::default();
        settings.content_width = 640.0;
        settings.side_margin = 48.0;
        settings.font_size = 18.0;
        settings.line_height = 1.8;
        settings.paragraph_spacing = 14.0;

        let layout = ContentLayout::from_settings(900.0, &settings);
        assert_eq!(layout.content_width, 640.0);
        assert_eq!(layout.side_margin, 48.0);
        assert_eq!(layout.body_font_size, 18.0);
        assert!((layout.line_height - 32.4).abs() < 1e-4);
        assert_eq!(layout.paragraph_spacing, 14.0);
    }

    #[test]
    fn content_layout_clamps_to_available_width() {
        let mut settings = ReaderSettings::default();
        settings.content_width = 900.0;

        let layout = ContentLayout::from_settings(600.0, &settings);
        assert_eq!(layout.content_width, 600.0);
    }
}
