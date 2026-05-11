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

    let desired_size = Vec2::new(140.0, 180.0);
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

        // Cover area — preserve aspect ratio from the real texture
        let card_w = rect.width();
        let cover_height = if let Some(tex) = cover_texture {
            let tex_size = tex.size();
            let tex_w = tex_size[0] as f32;
            let tex_h = tex_size[1] as f32;
            if tex_w > 0.0 && tex_h > 0.0 {
                (card_w * tex_h / tex_w).min(160.0)
            } else {
                140.0
            }
        } else {
            140.0
        };
        let cover_rect = egui::Rect::from_min_size(rect.min, Vec2::new(card_w, cover_height));
        let cover_rounding = CornerRadius { nw: r.card as u8, ne: r.card as u8, sw: 0, se: 0 };

        if let Some(texture) = cover_texture {
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

        // Progress percentage overlaid on cover bottom
        let pct = (item.progress_percent * 100.0) as u32;
        if pct > 0 && pct < 100 {
            let pct_text = format!("{}%", pct);
            let font_id = egui::FontId::new(typo.caption_size - 1.0, egui::FontFamily::Proportional);
            let galley = ui.ctx().fonts_mut(|f| f.layout_no_wrap(pct_text.clone(), font_id.clone(), egui::Color32::WHITE));
            let text_w = galley.rect.width();
            let text_h = galley.rect.height();
            let pad = 4.0;
            let bg_rect = egui::Rect::from_min_size(
                egui::pos2(cover_rect.right() - text_w - pad * 2.0 - 4.0, cover_rect.bottom() - text_h - pad * 2.0 - 2.0),
                Vec2::new(text_w + pad * 2.0, text_h + pad * 2.0),
            );
            painter.rect_filled(bg_rect, CornerRadius::same(4), egui::Color32::from_black_alpha(140));
            painter.text(
                bg_rect.center(),
                egui::Align2::CENTER_CENTER,
                &pct_text,
                font_id,
                egui::Color32::WHITE,
            );
        } else if pct >= 100 {
            let font_id = egui::FontId::new(typo.caption_size - 1.0, egui::FontFamily::Proportional);
            let galley = ui.ctx().fonts_mut(|f| f.layout_no_wrap("✓".to_string(), font_id.clone(), egui::Color32::WHITE));
            let sz = galley.rect.width().max(galley.rect.height()) + 6.0;
            let bg_rect = egui::Rect::from_min_size(
                egui::pos2(cover_rect.right() - sz - 4.0, cover_rect.bottom() - sz - 2.0),
                Vec2::new(sz, sz),
            );
            painter.rect_filled(bg_rect, CornerRadius::same(4), colors.accent.to_color32().gamma_multiply(0.7));
            painter.text(bg_rect.center(), egui::Align2::CENTER_CENTER, "✓", font_id, egui::Color32::WHITE);
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

pub(crate) fn format_cover_color(format: &BookFormat) -> egui::Color32 {
    match format {
        BookFormat::Epub => egui::Color32::from_rgb(204, 120, 92),   // coral
        BookFormat::Txt => egui::Color32::from_rgb(93, 168, 150),    // accent-teal
        BookFormat::ReservedPdf => egui::Color32::from_rgb(198, 69, 69),
        BookFormat::ReservedMobi => egui::Color32::from_rgb(232, 165, 90), // amber
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
