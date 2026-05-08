use eframe::egui::{self, CornerRadius, Vec2};
use crate::domain::book_format::BookFormat;
use crate::domain::library_item::{FileHealth, LibraryItem};
use crate::ui::ThemeConfig;

/// A reusable book card widget with a realistic cover appearance.
/// Renders cover with title, author, format label, and progress bar.
/// `cover_texture` is an optional pre-loaded egui texture for the real cover.
/// Returns click responses.
pub fn book_card(
    ui: &mut egui::Ui,
    item: &LibraryItem,
    theme: &ThemeConfig,
    cover_texture: Option<&egui::TextureHandle>,
) -> Vec<egui::Response> {
    let r = &theme.radius;
    let colors = &theme.colors;
    let typo = &theme.typography;
    let shadow = &theme.shadow;

    let is_missing = item.file_health != FileHealth::Ok;

    let desired_size = Vec2::new(140.0, 200.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);

        // Card shadow
        let shadow_rect = rect.translate(Vec2::new(2.0, 3.0));
        painter.rect_filled(
            shadow_rect,
            CornerRadius::same(r.card as u8 + 2),
            egui::Color32::from_black_alpha(shadow.card_shadow_alpha),
        );

        // Card background
        let card_bg = colors.panel_bg.to_color32();
        let card_rounding = CornerRadius::same(r.card as u8);
        painter.rect_filled(rect, card_rounding, card_bg);

        // Cover area (top ~70%)
        let cover_height = 130.0;
        let cover_rect = egui::Rect::from_min_size(rect.min, Vec2::new(rect.width(), cover_height));
        let cover_rounding = CornerRadius { nw: r.card as u8, ne: r.card as u8, sw: 0, se: 0 };

        if let Some(texture) = cover_texture {
            // Render real cover image
            painter.image(
                texture.id(),
                cover_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        // Fallback: color background with spine effect
        let cover_base = format_cover_color(&item.format);
        let cover_dark = cover_base.gamma_multiply(0.7);
        if cover_texture.is_none() {
            painter.rect_filled(cover_rect, cover_rounding, cover_base);
        }

        // Book spine effect (left edge darker strip)
        let spine_rect = egui::Rect::from_min_size(cover_rect.min, Vec2::new(6.0, cover_rect.height()));
        painter.rect_filled(
            spine_rect,
            CornerRadius { nw: r.card as u8, ne: 0, sw: 0, se: 0 },
            cover_dark,
        );

        // Bottom accent line on cover
        let accent_line = egui::Rect::from_min_size(
            egui::pos2(cover_rect.left() + 12.0, cover_rect.bottom() - 4.0),
            Vec2::new(cover_rect.width() - 24.0, 3.0),
        );
        painter.rect_filled(accent_line, CornerRadius::same(1), colors.accent.to_color32());

        // Title on cover (centered, multi-line)
        let title_text = truncate_title(&item.title, 16);
        let title_font = egui::FontId::new(typo.body_size - 1.0, egui::FontFamily::Proportional);
        painter.text(
            egui::pos2(cover_rect.center().x, cover_rect.top() + 40.0),
            egui::Align2::CENTER_TOP,
            &title_text,
            title_font,
            egui::Color32::WHITE,
        );

        // Author on cover
        if let Some(ref author) = item.author {
            let author_short = truncate_title(author, 12);
            painter.text(
                egui::pos2(cover_rect.center().x, cover_rect.top() + 60.0 + typo.body_size),
                egui::Align2::CENTER_TOP,
                &author_short,
                egui::FontId::new(typo.caption_size - 1.0, egui::FontFamily::Proportional),
                egui::Color32::from_white_alpha(200),
            );
        }

        // Format badge
        let badge_text = format_tag(&item.format);
        let badge_size = Vec2::new(36.0, 18.0);
        let badge_pos = egui::pos2(cover_rect.right() - badge_size.x - 4.0, cover_rect.top() + 4.0);
        let badge_rect = egui::Rect::from_min_size(badge_pos, badge_size);
        painter.rect_filled(badge_rect, CornerRadius::same(3), egui::Color32::from_black_alpha(shadow.badge_bg_alpha));
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            badge_text,
            egui::FontId::new(10.0, egui::FontFamily::Proportional),
            egui::Color32::WHITE,
        );

        // Missing file indicator
        if is_missing {
            let warn_pos = egui::pos2(cover_rect.left() + 6.0, cover_rect.top() + 6.0);
            painter.text(
                warn_pos,
                egui::Align2::LEFT_TOP,
                "\u{26A0}",
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
                colors.warning.to_color32(),
            );
        }

        // Bottom info area
        let info_x = rect.left() + 8.0;
        let mut info_y = cover_rect.bottom() + 6.0;

        // Book title (below cover, strong)
        let display_title = truncate_title(&item.title, 14);
        painter.text(
            egui::pos2(info_x, info_y),
            egui::Align2::LEFT_TOP,
            &display_title,
            egui::FontId::new(typo.caption_size, egui::FontFamily::Proportional),
            if is_missing { colors.text_muted.to_color32() } else { colors.text_primary.to_color32() },
        );
        info_y += typo.caption_size + 2.0;

        // Chapter count & progress
        let sub_info = format!("{}章 · {:.0}%", item.chapter_count, item.progress_percent * 100.0);
        painter.text(
            egui::pos2(info_x, info_y),
            egui::Align2::LEFT_TOP,
            &sub_info,
            egui::FontId::new(typo.caption_size - 2.0, egui::FontFamily::Proportional),
            colors.text_secondary.to_color32(),
        );
        info_y += typo.caption_size - 1.0;

        // Mini progress bar
        if item.chapter_count > 0 {
            let bar_width = rect.width() - 16.0;
            let bar_height = 3.0;
            let bar_y = info_y + 4.0;
            let progress_filled = item.progress_percent.clamp(0.0, 1.0);

            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(info_x, bar_y), Vec2::new(bar_width, bar_height)),
                CornerRadius::same(2),
                colors.border_subtle.to_color32(),
            );
            if progress_filled > 0.0 {
                painter.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(info_x, bar_y), Vec2::new(bar_width * progress_filled, bar_height)),
                    CornerRadius::same(2),
                    colors.accent.to_color32(),
                );
            }
        }

        // Hover highlight
        if response.hovered() {
            painter.rect_stroke(rect, card_rounding, egui::Stroke::new(1.5, colors.accent.to_color32()), egui::StrokeKind::Inside);
        }

        // Missing file border
        if is_missing {
            painter.rect_stroke(rect, card_rounding, egui::Stroke::new(2.0, colors.danger.to_color32()), egui::StrokeKind::Inside);
        }
    }

    let mut responses = Vec::new();
    if response.clicked() {
        responses.push(response);
    }
    responses
}

fn truncate_title(title: &str, max_chars: usize) -> String {
    let count = title.chars().count();
    if count <= max_chars {
        title.to_string()
    } else {
        title.chars().take(max_chars - 1).chain(std::iter::once('…')).collect()
    }
}

pub(crate) fn format_cover_color(format: &BookFormat) -> egui::Color32 {
    match format {
        BookFormat::Epub => egui::Color32::from_rgb(60, 100, 160),
        BookFormat::Txt => egui::Color32::from_rgb(80, 140, 80),
        BookFormat::ReservedPdf => egui::Color32::from_rgb(170, 70, 70),
        BookFormat::ReservedMobi => egui::Color32::from_rgb(100, 75, 150),
    }
}

pub fn format_tag(format: &BookFormat) -> &'static str {
    match format {
        BookFormat::Epub => "EPUB",
        BookFormat::Txt => "TXT",
        BookFormat::ReservedPdf => "PDF",
        BookFormat::ReservedMobi => "MOBI",
    }
}
