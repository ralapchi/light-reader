use std::cell::Cell;

use eframe::egui;

use crate::app::Action;
use crate::domain::chapter::Chapter;
use crate::domain::paragraph_kind::ParagraphKind;
use crate::domain::reader_settings::ReaderSettings;
use crate::domain::search_result::SearchResult;
use crate::ui::ThemeConfig;

const SCROLL_OFFSET_THRESHOLD: f32 = 1.0;

thread_local! {
    static LAST_SCROLL_OFFSET: Cell<f32> = const { Cell::new(0.0) };
}

pub fn reader_view(
    ui: &mut egui::Ui,
    chapters: &[Chapter],
    chapter_index: usize,
    settings: &ReaderSettings,
    theme: &ThemeConfig,
    selected_search_result: Option<&SearchResult>,
) -> Vec<Action> {
    let s = &theme.spacing;
    let max_width = settings.content_width;
    let margin = settings.side_margin;
    let available_width = ui.available_width();
    let cw = available_width.min(max_width);
    let sm = (available_width - cw) / 2.0;
    let mut actions = Vec::new();

    // Check if we need to highlight a search result
    let highlight_para_index = selected_search_result
        .filter(|r| r.chapter_index == chapter_index)
        .map(|r| r.paragraph_index);

    let mut scroll_to_highlight = false;

    // Use chapter_index in id_salt so scroll resets when switching chapters
    let scroll_id = format!("reader_chapter_{}", chapter_index);
    let scroll_output = egui::ScrollArea::vertical()
        .id_salt(scroll_id)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Ensure horizontal layout expands to full width
                ui.set_min_width(ui.available_width());

                ui.add_space(sm.max(margin));

                ui.vertical(|ui| {
                    ui.set_width(cw);
                    ui.add_space(s.reader_top_padding);

                    if let Some(chapter) = chapters.get(chapter_index) {
                        let font_size = settings.font_size;
                        let line_height = Some(font_size * settings.line_height);

                        // Chapter header
                        chapter_header(ui, &chapter.title, theme);

                        for paragraph in &chapter.paragraphs {
                            let is_highlighted = highlight_para_index == Some(paragraph.index);

                            if is_highlighted && !scroll_to_highlight {
                                scroll_to_highlight = true;
                            }

                            match paragraph.kind {
                                ParagraphKind::Title => {
                                    let resp = ui.vertical_centered(|ui| {
                                        ui.add_space(s.lg);
                                        ui.label(
                                            egui::RichText::new(&paragraph.text)
                                                .size(font_size * 1.5)
                                                .strong()
                                                .line_height(line_height.map(|l| l * 1.2)),
                                        );
                                        ui.add_space(s.sm);
                                    }).response;
                                    if is_highlighted {
                                        highlight_paragraph(ui, resp.rect, theme);
                                    }
                                }
                                ParagraphKind::Subtitle => {
                                    let resp = ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new(&paragraph.text)
                                                .size(font_size * 1.15)
                                                .weak()
                                                .line_height(line_height),
                                        );
                                        ui.add_space(s.xs);
                                    }).response;
                                    if is_highlighted {
                                        highlight_paragraph(ui, resp.rect, theme);
                                    }
                                }
                                ParagraphKind::Quote => {
                                    ui.add_space(s.sm);
                                    ui.add_space(s.lg);
                                    let resp = ui.label(
                                        egui::RichText::new(&paragraph.text)
                                            .size(font_size)
                                            .italics()
                                            .color(theme.colors.text_secondary.to_color32())
                                            .line_height(line_height),
                                    );
                                    if is_highlighted {
                                        highlight_paragraph(ui, resp.rect, theme);
                                    }
                                    ui.add_space(s.sm);
                                }
                                ParagraphKind::Separator => {
                                    ui.add_space(s.md);
                                    ui.separator();
                                    ui.add_space(s.md);
                                }
                                ParagraphKind::Body => {
                                    let indent = paragraph.indent_level as f32 * s.lg;
                                    ui.add_space(indent);
                                    let resp = ui.label(
                                        egui::RichText::new(&paragraph.text)
                                            .size(font_size)
                                            .line_height(line_height),
                                    );
                                    if is_highlighted {
                                        highlight_paragraph(ui, resp.rect, theme);
                                    }
                                    ui.add_space(s.paragraph_gap);
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
                                    ui.add_space(s.paragraph_gap);
                                }
                            }
                        }

                        // Chapter end spacer
                        chapter_end_spacer(ui, theme);
                    } else {
                        empty_chapter(ui, theme);
                    }
                });

                ui.add_space(sm.max(margin));
            });
        });

    // Track scroll offset with change detection
    let scroll_offset = scroll_output.state.offset.y;
    let content_height = scroll_output.content_size.y;
    let viewport_height = scroll_output.inner_rect.height();

    LAST_SCROLL_OFFSET.with(|last| {
        let prev = last.get();
        if (scroll_offset - prev).abs() > SCROLL_OFFSET_THRESHOLD {
            last.set(scroll_offset);
            actions.push(Action::UpdateScrollOffset(scroll_offset));
        }
    });

    // Auto-advance to next chapter when scrolled near bottom
    // Only trigger if content is tall enough to scroll
    if content_height > viewport_height {
        let distance_to_bottom = content_height - scroll_offset - viewport_height;
        if distance_to_bottom < 50.0 {
            // Use thread_local to avoid repeated triggers
            thread_local! {
                static REACHED_BOTTOM: Cell<bool> = const { Cell::new(false) };
            }
            REACHED_BOTTOM.with(|reached| {
                if !reached.get() {
                    reached.set(true);
                    actions.push(Action::NextChapter);
                }
            });
        } else {
            // Reset when scrolled away from bottom
            thread_local! {
                static REACHED_BOTTOM: Cell<bool> = const { Cell::new(false) };
            }
            REACHED_BOTTOM.with(|reached| {
                reached.set(false);
            });
        }
    }

    actions
}

fn chapter_header(ui: &mut egui::Ui, title: &str, theme: &ThemeConfig) {
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.add_space(s.lg);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(title)
                .size(t.title_size)
                .strong()
                .color(theme.colors.text_primary.to_color32()),
        );
    });
    ui.add_space(s.md);
    ui.separator();
    ui.add_space(s.lg);
}

fn chapter_end_spacer(ui: &mut egui::Ui, theme: &ThemeConfig) {
    let s = &theme.spacing;
    ui.add_space(s.xl * 3.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("--- 章节结束 ---")
                .size(theme.typography.caption_size)
                .color(theme.colors.text_muted.to_color32()),
        );
    });
    ui.add_space(s.xl * 2.0);
}

fn empty_chapter(ui: &mut egui::Ui, theme: &ThemeConfig) {
    let s = &theme.spacing;
    ui.add_space(s.xl * 4.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("无内容")
                .size(theme.typography.body_size)
                .color(theme.colors.text_muted.to_color32()),
        );
    });
}

fn highlight_paragraph(ui: &mut egui::Ui, rect: egui::Rect, theme: &ThemeConfig) {
    let painter = ui.painter();

    // Background highlight
    painter.rect_filled(
        rect,
        egui::CornerRadius::same(4),
        theme.colors.accent.to_color32().gamma_multiply(0.15),
    );

    // Left accent bar
    let bar_rect = egui::Rect::from_min_size(
        rect.left_top(),
        egui::vec2(3.0, rect.height()),
    );
    painter.rect_filled(
        bar_rect,
        egui::CornerRadius::same(2),
        theme.colors.accent.to_color32(),
    );
}
