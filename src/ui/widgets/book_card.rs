use eframe::egui::{self, CornerRadius, Vec2};
use crate::app::Action;
use crate::domain::book_format::BookFormat;
use crate::domain::library_item::{FileHealth, LibraryItem};
use crate::ui::ThemeConfig;

/// A reusable book card widget with a realistic cover appearance.
/// Renders cover with title, author, format label, and progress bar.
/// `cover_texture` is an optional pre-loaded egui texture for the real cover.
/// Returns (click responses, actions from popup menu).
pub fn book_card(
    ui: &mut egui::Ui,
    item: &LibraryItem,
    theme: &ThemeConfig,
    cover_texture: Option<&egui::TextureHandle>,
) -> (Vec<egui::Response>, Vec<Action>) {
    let r = &theme.radius;
    let colors = &theme.colors;
    let shadow = &theme.shadow;

    let is_missing = item.file_health != FileHealth::Ok;

    let desired_size = Vec2::new(theme.panel.card_width, theme.panel.card_height);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // ── Hover float animation (3px lift) ──────────────────
    let card_id = ui.id().with(&item.book_id);
    let hover_id = card_id.with("hover");
    let float_offset = ui.ctx().animate_bool(hover_id, response.hovered());
    let mut popup_actions = Vec::new();

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);

        // Apply hover float: shift card up by 3px when hovered
        let float_rect = rect.translate(Vec2::new(0.0, -3.0 * float_offset));

        // ── Dual-layer shadow ──────────────────────────────
        // Far layer: larger blur, more spread
        let far_shadow = rect.translate(Vec2::new(0.0, 4.0 + 2.0 * float_offset));
        painter.rect_filled(
            far_shadow,
            CornerRadius::same(r.card as u8 + 2),
            egui::Color32::from_black_alpha((shadow.card_shadow_alpha as f32 * 0.5 * (1.0 + float_offset)) as u8),
        );
        // Near layer: tighter, darker
        let near_shadow = rect.translate(Vec2::new(0.0, 2.0 + 1.0 * float_offset));
        painter.rect_filled(
            near_shadow,
            CornerRadius::same(r.card as u8),
            egui::Color32::from_black_alpha((shadow.card_shadow_alpha as f32 * 0.7 * (1.0 + float_offset)) as u8),
        );

        // Card background
        let card_bg = colors.panel_bg.to_color32();
        let card_rounding = CornerRadius::same(r.card as u8);
        painter.rect_filled(float_rect, card_rounding, card_bg);

        // Cover area — preserve aspect ratio from the real texture
        let card_w = float_rect.width();
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
        let cover_rect = egui::Rect::from_min_size(float_rect.min, Vec2::new(card_w, cover_height));
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

        // Bottom gradient overlay for text readability
        let gradient_h = 32.0;
        let gradient_rect = egui::Rect::from_min_size(
            egui::pos2(cover_rect.left(), cover_rect.bottom() - gradient_h),
            Vec2::new(cover_rect.width(), gradient_h),
        );
        painter.rect_filled(gradient_rect, CornerRadius { nw: 0, ne: 0, sw: r.card as u8, se: r.card as u8 },
            egui::Color32::from_black_alpha(80));

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

        // Progress bar at cover bottom
        let pct = (item.progress_percent * 100.0) as u32;
        if pct > 0 && pct < 100 {
            let bar_h = 3.0;
            let bar_pad = 6.0;
            let bar_bg_rect = egui::Rect::from_min_size(
                egui::pos2(cover_rect.left() + bar_pad, cover_rect.bottom() - bar_h - bar_pad),
                Vec2::new(cover_rect.width() - bar_pad * 2.0, bar_h),
            );
            // Track
            painter.rect_filled(bar_bg_rect, CornerRadius::same(2), egui::Color32::from_black_alpha(60));
            // Fill
            let fill_w = bar_bg_rect.width() * (pct as f32 / 100.0);
            if fill_w > 0.0 {
                let fill_rect = egui::Rect::from_min_size(
                    bar_bg_rect.min,
                    Vec2::new(fill_w, bar_h),
                );
                painter.rect_filled(fill_rect, CornerRadius::same(2), colors.accent.to_color32());
            }
        } else if pct >= 100 {
            // Completed badge: accent-colored circle with checkmark
            let badge_r = 10.0;
            let badge_center = egui::pos2(cover_rect.right() - badge_r - 6.0, cover_rect.bottom() - badge_r - 6.0);
            painter.circle_filled(badge_center, badge_r, colors.accent.to_color32());
            painter.text(
                badge_center,
                egui::Align2::CENTER_CENTER,
                "✓",
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
                egui::Color32::WHITE,
            );
        }

        // ── Reading status label (capsule) ────────────────
        {
            let (label_text, label_color) = if pct >= 100 {
                ("已读完".to_string(), colors.success.to_color32())
            } else if pct > 0 {
                (format!("{}%", pct), colors.accent.to_color32())
            } else {
                ("新".to_string(), colors.text_muted.to_color32())
            };
            let font_id = egui::FontId::new(10.0, egui::FontFamily::Proportional);
            let galley = painter.layout_no_wrap(label_text, font_id, egui::Color32::WHITE);
            let label_w = galley.size().x + 12.0;
            let label_h = 18.0;
            let label_pos = egui::pos2(float_rect.left() + 6.0, float_rect.bottom() - label_h - 6.0);
            let label_rect = egui::Rect::from_min_size(label_pos, Vec2::new(label_w, label_h));
            painter.rect_filled(label_rect, CornerRadius::same(label_h as u8 / 2), label_color);
            painter.galley(
                egui::pos2(label_rect.center().x - galley.size().x / 2.0, label_rect.center().y - galley.size().y / 2.0),
                galley,
                egui::Color32::WHITE,
            );
        }

        // Hover effect: accent overlay + stroke
        if response.hovered() {
            painter.rect_filled(float_rect, card_rounding, colors.accent.to_color32().gamma_multiply(0.08));
            painter.rect_stroke(float_rect, card_rounding, egui::Stroke::new(1.5, colors.accent.to_color32()), egui::StrokeKind::Inside);
        }

        // Missing file border
        if is_missing {
            painter.rect_stroke(float_rect, card_rounding, egui::Stroke::new(2.0, colors.danger.to_color32()), egui::StrokeKind::Inside);
        }

        // ── Three-dot menu (show on hover) ────────────────
        if response.hovered() {
            let menu_size = Vec2::new(24.0, 24.0);
            let menu_pos = egui::pos2(float_rect.right() - menu_size.x - 4.0, float_rect.bottom() - menu_size.y - 4.0);
            let menu_rect = egui::Rect::from_min_size(menu_pos, menu_size);
            painter.rect_filled(menu_rect, CornerRadius::same(4), egui::Color32::from_black_alpha(60));
            painter.text(
                menu_rect.center(),
                egui::Align2::CENTER_CENTER,
                "⋮",
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
                egui::Color32::WHITE,
            );

            // Handle click on the menu area
            let menu_response = ui.interact(menu_rect, card_id.with("menu"), egui::Sense::click());
            if menu_response.clicked() {
                egui::Popup::toggle_id(ui.ctx(), card_id.with("popup"));
            }
        }

        // Popup menu
        let popup_id = card_id.with("popup");
        if egui::Popup::is_id_open(ui.ctx(), popup_id) {
            let popup_pos = egui::pos2(float_rect.right() - 120.0, float_rect.bottom() + 4.0);
            let area = egui::Area::new(popup_id)
                .fixed_pos(popup_pos)
                .order(egui::Order::Foreground);
            let frame = egui::Frame::popup(ui.style());
            let inner = area.show(ui.ctx(), |ui| {
                frame.show(ui, |ui| {
                    ui.set_min_width(110.0);
                    if ui.button("打开").clicked() {
                        popup_actions.push(Action::LibraryBookSelected(item.book_id.clone()));
                        egui::Popup::close_id(ui.ctx(), popup_id);
                    }
                    if ui.button("移除").clicked() {
                        popup_actions.push(Action::RemoveFromLibrary(item.book_id.clone()));
                        egui::Popup::close_id(ui.ctx(), popup_id);
                    }
                });
            });
            // Close popup when clicking outside
            if inner.response.clicked_elsewhere() {
                egui::Popup::close_id(ui.ctx(), popup_id);
            }
        }
    }

    let mut responses = Vec::new();
    if response.clicked() {
        responses.push(response);
    }
    (responses, popup_actions)
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

/// A scaled version of `book_card`. `scale` multiplies the card dimensions.
/// e.g. `1.3` produces a 1.3x larger card for hero/continue-reading displays.
pub fn book_card_scaled(
    ui: &mut egui::Ui,
    item: &LibraryItem,
    theme: &ThemeConfig,
    cover_texture: Option<&egui::TextureHandle>,
    scale: f32,
) -> (Vec<egui::Response>, Vec<Action>) {
    let scale = scale.max(1.0);
    let base_w = theme.panel.card_width * scale;
    let base_h = theme.panel.card_height * scale;
    let desired_size = Vec2::new(base_w, base_h);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    let r = &theme.radius;
    let colors = &theme.colors;
    let shadow = &theme.shadow;
    let is_missing = item.file_health != FileHealth::Ok;

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);

        // Shadow
        let shadow_rect = rect.translate(Vec2::new(2.0 * scale, 3.0 * scale));
        painter.rect_filled(
            shadow_rect,
            CornerRadius::same((r.card * scale) as u8 + 2),
            egui::Color32::from_black_alpha(shadow.card_shadow_alpha),
        );

        // Card bg
        let card_bg = colors.panel_bg.to_color32();
        let card_rounding = CornerRadius::same((r.card * scale) as u8);
        painter.rect_filled(rect, card_rounding, card_bg);

        // Cover
        let card_w = rect.width();
        let cover_height = if let Some(tex) = cover_texture {
            let tex_size = tex.size();
            let tex_w = tex_size[0] as f32;
            let tex_h = tex_size[1] as f32;
            if tex_w > 0.0 && tex_h > 0.0 {
                (card_w * tex_h / tex_w).min(160.0 * scale)
            } else {
                140.0 * scale
            }
        } else {
            140.0 * scale
        };
        let cover_rect = egui::Rect::from_min_size(rect.min, Vec2::new(card_w, cover_height));
        let cover_rounding = CornerRadius { nw: (r.card * scale) as u8, ne: (r.card * scale) as u8, sw: 0, se: 0 };

        if let Some(texture) = cover_texture {
            painter.image(
                texture.id(),
                cover_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        let cover_base = format_cover_color(&item.format);
        let cover_dark = cover_base.gamma_multiply(0.7);
        if cover_texture.is_none() {
            painter.rect_filled(cover_rect, cover_rounding, cover_base);
        }

        // Spine
        let spine_w = 6.0 * scale;
        let spine_rect = egui::Rect::from_min_size(cover_rect.min, Vec2::new(spine_w, cover_rect.height()));
        painter.rect_filled(
            spine_rect,
            CornerRadius { nw: (r.card * scale) as u8, ne: 0, sw: 0, se: 0 },
            cover_dark,
        );

        // Accent line
        let pad = 12.0 * scale;
        let line_h = 3.0 * scale;
        let accent_line = egui::Rect::from_min_size(
            egui::pos2(cover_rect.left() + pad, cover_rect.bottom() - line_h - 1.0 * scale),
            Vec2::new(cover_rect.width() - pad * 2.0, line_h),
        );
        painter.rect_filled(accent_line, CornerRadius::same(1), colors.accent.to_color32());

        // Gradient
        let gradient_h = 32.0 * scale;
        let gradient_rect = egui::Rect::from_min_size(
            egui::pos2(cover_rect.left(), cover_rect.bottom() - gradient_h),
            Vec2::new(cover_rect.width(), gradient_h),
        );
        painter.rect_filled(gradient_rect, CornerRadius { nw: 0, ne: 0, sw: (r.card * scale) as u8, se: (r.card * scale) as u8 },
            egui::Color32::from_black_alpha(80));

        // Badge
        let badge_text = format_tag(&item.format);
        let badge_size = Vec2::new(36.0 * scale, 18.0 * scale);
        let badge_pos = egui::pos2(cover_rect.right() - badge_size.x - 4.0 * scale, cover_rect.top() + 4.0 * scale);
        let badge_rect = egui::Rect::from_min_size(badge_pos, badge_size);
        painter.rect_filled(badge_rect, CornerRadius::same(3), egui::Color32::from_black_alpha(shadow.badge_bg_alpha));
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            badge_text,
            egui::FontId::new(10.0 * scale, egui::FontFamily::Proportional),
            egui::Color32::WHITE,
        );

        // Progress
        let pct = (item.progress_percent * 100.0) as u32;
        if pct > 0 && pct < 100 {
            let bar_h = 3.0 * scale;
            let bar_pad = 6.0 * scale;
            let bar_bg_rect = egui::Rect::from_min_size(
                egui::pos2(cover_rect.left() + bar_pad, cover_rect.bottom() - bar_h - bar_pad),
                Vec2::new(cover_rect.width() - bar_pad * 2.0, bar_h),
            );
            painter.rect_filled(bar_bg_rect, CornerRadius::same(2), egui::Color32::from_black_alpha(60));
            let fill_w = bar_bg_rect.width() * (pct as f32 / 100.0);
            if fill_w > 0.0 {
                let fill_rect = egui::Rect::from_min_size(bar_bg_rect.min, Vec2::new(fill_w, bar_h));
                painter.rect_filled(fill_rect, CornerRadius::same(2), colors.accent.to_color32());
            }
        } else if pct >= 100 {
            let badge_r = 10.0 * scale;
            let badge_center = egui::pos2(cover_rect.right() - badge_r - 6.0 * scale, cover_rect.bottom() - badge_r - 6.0 * scale);
            painter.circle_filled(badge_center, badge_r, colors.accent.to_color32());
            painter.text(
                badge_center,
                egui::Align2::CENTER_CENTER,
                "✓",
                egui::FontId::new(12.0 * scale, egui::FontFamily::Proportional),
                egui::Color32::WHITE,
            );
        }

        // Hover
        if response.hovered() {
            painter.rect_filled(rect, card_rounding, colors.accent.to_color32().gamma_multiply(0.08));
            painter.rect_stroke(rect, card_rounding, egui::Stroke::new(1.5, colors.accent.to_color32()), egui::StrokeKind::Inside);
        }
        if is_missing {
            painter.rect_stroke(rect, card_rounding, egui::Stroke::new(2.0, colors.danger.to_color32()), egui::StrokeKind::Inside);
        }
    }

    let mut responses = Vec::new();
    if response.clicked() {
        responses.push(response);
    }
    (responses, Vec::new())
}
