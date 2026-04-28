use eframe::egui;

use crate::domain::chapter::Chapter;
use crate::domain::paragraph_kind::ParagraphKind;
use crate::domain::reader_settings::ReaderSettings;
use crate::ui::ThemeConfig;

pub fn reader_view(
    ui: &mut egui::Ui,
    chapters: &[Chapter],
    chapter_index: usize,
    settings: &ReaderSettings,
    theme: &ThemeConfig,
) {
    let s = &theme.spacing;
    let max_width = settings.content_width;
    let margin = settings.side_margin;
    let available_width = ui.available_width();
    let cw = available_width.min(max_width);
    let sm = (available_width - cw) / 2.0;

    egui::ScrollArea::vertical()
        .id_salt("reader_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(sm.max(margin));

                ui.vertical(|ui| {
                    ui.set_width(cw);
                    ui.add_space(s.xl);

                    if let Some(chapter) = chapters.get(chapter_index) {
                        let font_size = settings.font_size;
                        let line_height = Some(font_size * settings.line_height);

                        for paragraph in &chapter.paragraphs {
                            match paragraph.kind {
                                ParagraphKind::Title => {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(s.lg);
                                        ui.label(
                                            egui::RichText::new(&paragraph.text)
                                                .size(font_size * 1.5)
                                                .strong()
                                                .line_height(line_height.map(|l| l * 1.2)),
                                        );
                                        ui.add_space(s.sm);
                                    });
                                }
                                ParagraphKind::Subtitle => {
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new(&paragraph.text)
                                                .size(font_size * 1.15)
                                                .weak()
                                                .line_height(line_height),
                                        );
                                        ui.add_space(s.xs);
                                    });
                                }
                                ParagraphKind::Quote => {
                                    ui.add_space(s.sm);
                                    ui.label(
                                        egui::RichText::new(&paragraph.text)
                                            .size(font_size)
                                            .italics()
                                            .line_height(line_height),
                                    );
                                    ui.add_space(s.sm);
                                }
                                ParagraphKind::Separator => {
                                    ui.add_space(s.md);
                                    ui.separator();
                                    ui.add_space(s.md);
                                }
                                ParagraphKind::Body => {
                                    ui.label(
                                        egui::RichText::new(&paragraph.text)
                                            .size(font_size)
                                            .line_height(line_height),
                                    );
                                    ui.add_space(paragraph.indent_level as f32 * s.lg);
                                }
                            }
                        }

                        // Fallback for chapters with no paragraphs
                        if chapter.paragraphs.is_empty() {
                            for text in chapter.content.split("\n\n") {
                                let trimmed = text.trim();
                                if !trimmed.is_empty() {
                                    ui.label(
                                        egui::RichText::new(trimmed)
                                            .size(font_size)
                                            .line_height(line_height),
                                    );
                                    ui.add_space(s.sm);
                                }
                            }
                        }

                        ui.add_space(s.xl * 2.0);
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(s.xl * 4.0);
                            ui.label("无内容");
                        });
                    }
                });

                ui.add_space(sm.max(margin));
            });
        });
}
