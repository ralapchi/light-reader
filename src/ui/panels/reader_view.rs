use std::cell::Cell;

use eframe::egui;

use crate::app::Action;
use crate::domain::chapter::Chapter;
use crate::domain::paragraph_kind::ParagraphKind;
use crate::domain::reader_settings::ReaderSettings;
use crate::domain::search_result::SearchResult;
use crate::ui::image_cache::ImageCache;
use crate::ui::ThemeConfig;
use crate::ui::theme::ThemeSpacing;
use crate::ui::widgets::render_highlighted_text;

const SCROLL_OFFSET_THRESHOLD: f32 = 1.0;

thread_local! {
    static LAST_SCROLL_OFFSET: Cell<f32> = const { Cell::new(0.0) };
    static IMG_CACHE: std::cell::RefCell<ImageCache> = std::cell::RefCell::new(ImageCache::new());
}

pub fn reader_view(
    ui: &mut egui::Ui,
    chapters: &[Chapter],
    chapter_index: usize,
    settings: &ReaderSettings,
    theme: &ThemeConfig,
    selected_search_result: Option<&SearchResult>,
    search_keyword: Option<&str>,
    case_sensitive: bool,
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
    let next_chapter_clicked = Cell::new(false);

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
                        let font_family = parse_font_family(&settings.font_family);

                        // Chapter header
                        chapter_header(ui, &chapter.title, theme);

                        // Render content blocks in order (E-1: images interleaved with paragraphs)
                        if !chapter.blocks.is_empty() {
                            for block in &chapter.blocks {
                                match block {
                                    crate::domain::chapter_block::ChapterBlock::Paragraph(paragraph) => {
                                        let is_highlighted = highlight_para_index == Some(paragraph.index);
                                        if is_highlighted && !scroll_to_highlight {
                                            scroll_to_highlight = true;
                                        }
                                        render_paragraph_block(
                                            ui, paragraph, is_highlighted, font_size, &font_family,
                                            line_height, settings, theme, case_sensitive, search_keyword, s,
                                        );
                                    }
                                    crate::domain::chapter_block::ChapterBlock::Image(img) => {
                                        render_image_block(ui, img, settings.content_width, s.sm, theme);
                                    }
                                    crate::domain::chapter_block::ChapterBlock::Separator => {
                                        ui.add_space(s.md);
                                        ui.separator();
                                        ui.add_space(s.md);
                                    }
                                }
                            }
                        } else {
                            // Fallback to paragraphs-only rendering
                            for paragraph in &chapter.paragraphs {
                                let is_highlighted = highlight_para_index == Some(paragraph.index);
                                if is_highlighted && !scroll_to_highlight {
                                    scroll_to_highlight = true;
                                }
                                render_paragraph_block(
                                    ui, paragraph, is_highlighted, font_size, &font_family,
                                    line_height, settings, theme, case_sensitive, search_keyword, s,
                                );
                            }
                        }

                        // Fallback for chapters with no paragraphs
                        if chapter.paragraphs.is_empty() {
                            let fallback_font_id = egui::FontId::new(font_size, font_family.clone());
                            for text in chapter.content.split("\n\n") {
                                let trimmed = text.trim();
                                if !trimmed.is_empty() {
                                    ui.label(
                                        egui::RichText::new(trimmed)
                                            .font(fallback_font_id.clone())
                                            .line_height(line_height),
                                    );
                                    ui.add_space(settings.paragraph_spacing);
                                }
                            }
                        }

                        // Chapter end spacer with next chapter button
                        chapter_end_spacer(ui, theme, &next_chapter_clicked);
                    } else {
                        empty_chapter(ui, theme);
                    }
                });

                ui.add_space(sm.max(margin));
            });
        });

    // Check if "next chapter" button was clicked
    if next_chapter_clicked.get() {
        actions.push(Action::NextChapter);
    }

    // Track scroll offset with change detection
    let scroll_offset = scroll_output.state.offset.y;

    LAST_SCROLL_OFFSET.with(|last| {
        let prev = last.get();
        if (scroll_offset - prev).abs() > SCROLL_OFFSET_THRESHOLD {
            last.set(scroll_offset);
            actions.push(Action::UpdateScrollOffset(scroll_offset));
        }
    });

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

fn chapter_end_spacer(ui: &mut egui::Ui, theme: &ThemeConfig, next_chapter_clicked: &Cell<bool>) {
    let s = &theme.spacing;
    ui.add_space(s.xl * 3.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("--- 章节结束 ---")
                .size(theme.typography.caption_size)
                .color(theme.colors.text_muted.to_color32()),
        );
        ui.add_space(s.md);
        if ui.button("下一章").clicked() {
            next_chapter_clicked.set(true);
        }
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

/// Render a single paragraph block.
fn render_paragraph_block(
    ui: &mut egui::Ui,
    paragraph: &crate::domain::paragraph::Paragraph,
    is_highlighted: bool,
    font_size: f32,
    font_family: &egui::FontFamily,
    line_height: Option<f32>,
    settings: &ReaderSettings,
    theme: &ThemeConfig,
    case_sensitive: bool,
    search_keyword: Option<&str>,
    s: &ThemeSpacing,
) {
    match paragraph.kind {
        ParagraphKind::Title => {
            let title_font_id = egui::FontId::new(font_size * 1.5, font_family.clone());
            let resp = ui.vertical_centered(|ui| {
                ui.add_space(s.lg);
                ui.label(
                    egui::RichText::new(&paragraph.text)
                        .font(title_font_id)
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
            let subtitle_font_id = egui::FontId::new(font_size * 1.15, font_family.clone());
            let resp = ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(&paragraph.text)
                        .font(subtitle_font_id)
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
            let quote_font_id = egui::FontId::new(font_size, font_family.clone());
            let resp = ui.label(
                egui::RichText::new(&paragraph.text)
                    .font(quote_font_id)
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
            let font_id = egui::FontId::new(font_size, font_family.clone());
            let resp = ui.horizontal_wrapped(|ui| {
                if indent > 0.0 {
                    ui.add_space(indent);
                }
                if is_highlighted {
                    if let Some(keyword) = search_keyword {
                        render_highlighted_text(
                            ui, &paragraph.text, keyword, font_size, Some(&font_id),
                            line_height, theme, case_sensitive,
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(&paragraph.text)
                                .font(font_id.clone()).line_height(line_height),
                        );
                    }
                } else {
                    ui.label(
                        egui::RichText::new(&paragraph.text)
                            .font(font_id).line_height(line_height),
                    );
                }
            }).response;
            if is_highlighted {
                highlight_paragraph(ui, resp.rect, theme);
            }
            ui.add_space(settings.paragraph_spacing);
        }
    }
}

/// Render an inline image block from EPUB content.
fn render_image_block(
    ui: &mut egui::Ui,
    img: &crate::domain::chapter_block::InlineImageBlock,
    max_width: f32,
    spacing: f32,
    theme: &ThemeConfig,
) {
    ui.add_space(spacing);
    ui.vertical_centered(|ui| {
        // Try to load the cached image
        let mut found = false;
        log::info!("渲染图片块: asset_id={}", img.asset_id);
        if let Some(tex) = IMG_CACHE.with(|c| c.borrow_mut().image_texture(
            ui.ctx(), "", &img.asset_id,
        )) {
            let tex_size = tex.size_vec2();
            let scale = (max_width / tex_size.x).min(1.0);
            let display_size = egui::Vec2::new(tex_size.x * scale, tex_size.y * scale);
            ui.image(egui::load::SizedTexture::new(tex.id(), display_size));
            found = true;
        }
        if !found {
            // Placeholder for failed/missing images
            let ph = egui::Rect::from_min_size(ui.next_widget_position(), egui::Vec2::new(max_width.min(200.0), 100.0));
            let (_r, _resp) = ui.allocate_exact_size(egui::Vec2::new(max_width.min(200.0), 100.0), egui::Sense::hover());
            let painter = ui.painter_at(ph);
            painter.rect_filled(ph, egui::CornerRadius::same(4), theme.colors.border_subtle.to_color32());
            painter.text(
                ph.center(),
                egui::Align2::CENTER_CENTER,
                if let Some(ref alt) = img.alt_text { alt } else { "[图片加载失败]" },
                egui::FontId::new(theme.typography.caption_size, egui::FontFamily::Proportional),
                theme.colors.text_muted.to_color32(),
            );
        }
        // Caption
        if let Some(ref caption) = img.caption {
            ui.add_space(spacing * 0.5);
            ui.label(
                egui::RichText::new(caption)
                    .size(theme.typography.caption_size)
                    .color(theme.colors.text_secondary.to_color32()),
            );
        }
    });
    ui.add_space(spacing);
}

/// Parse font family string to egui::FontFamily
fn parse_font_family(family: &str) -> egui::FontFamily {
    match family {
        "monospace" => egui::FontFamily::Monospace,
        "serif" => egui::FontFamily::Name("serif".into()),
        _ => egui::FontFamily::Proportional, // sans-serif and default
    }
}
