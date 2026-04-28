use std::sync::OnceLock;

use eframe::egui;
use log::debug;

use crate::ui::ThemeConfig;

static FONTS_INITIALIZED: OnceLock<()> = OnceLock::new();

pub struct ThemeService;

impl ThemeService {
    /// Initialize fonts once per app lifetime.
    /// Safe to call multiple times; only the first call takes effect.
    pub fn init_fonts(ctx: &egui::Context) {
        FONTS_INITIALIZED.get_or_init(|| {
            debug!("Initializing fonts (one-shot)");
            let mut fonts = egui::FontDefinitions::default();

            #[cfg(target_os = "macos")]
            {
                let path = "/System/Library/Fonts/Hiragino Sans GB.ttc";
                if std::path::Path::new(path).exists() {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_static(include_bytes!(
                            "/System/Library/Fonts/Hiragino Sans GB.ttc"
                        ))
                        .into(),
                    );
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .insert(0, "chinese".to_owned());
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .push("chinese".to_owned());
                    debug!("Loaded Hiragino Sans GB font");
                } else {
                    debug!("Hiragino Sans GB not found, using default fonts");
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                debug!("Not on macOS, using default egui fonts");
            }

            ctx.set_fonts(fonts);
        });
    }

    /// Apply a ThemeConfig to the egui context (egui 0.33 API).
    ///
    /// This sets:
    /// - Colors (widgets, selection, window, panel)
    /// - Text styles (heading / body / button / small sizes)
    /// - Widget corner radii
    /// - Window / popup shadows
    /// - Spacing baseline values (item_spacing, button_padding, margins)
    pub fn apply_theme(ctx: &egui::Context, theme: &ThemeConfig) {
        let mut style = (*ctx.style()).clone();
        let v = &mut style.visuals;
        let c = &theme.colors;

        // ── Fill colors ──
        v.window_fill = c.window_bg.to_color32();
        v.panel_fill = c.panel_bg.to_color32();
        v.extreme_bg_color = c.window_bg.to_color32();
        v.faint_bg_color = c.reader_bg.to_color32();

        // ── Text / accent ──
        v.override_text_color = Some(c.text_primary.to_color32());
        v.hyperlink_color = c.accent.to_color32();
        v.code_bg_color = c.panel_bg_muted.to_color32();

        // ── Selection ──
        v.selection.bg_fill = c.selection_bg.to_color32();
        v.selection.stroke = egui::Stroke::new(1.0, c.border_strong.to_color32());

        // ── Widget states ──
        let set_widget = |w: &mut egui::style::WidgetVisuals,
                         bg: egui::Color32,
                         weak_bg: egui::Color32,
                         border: egui::Color32,
                         fg: egui::Color32,
                         cr: egui::CornerRadius| {
            w.bg_fill = bg;
            w.weak_bg_fill = weak_bg;
            w.bg_stroke = egui::Stroke::new(1.0, border);
            w.fg_stroke = egui::Stroke::new(1.0, fg);
            w.corner_radius = cr;
        };

        let rad = &theme.radius;
        let cr_widget = egui::CornerRadius::same(rad.button as u8);

        set_widget(
            &mut v.widgets.noninteractive,
            c.panel_bg.to_color32(),
            c.window_bg.to_color32(),
            c.border_subtle.to_color32(),
            c.text_primary.to_color32(),
            cr_widget,
        );
        set_widget(
            &mut v.widgets.inactive,
            c.panel_bg.to_color32(),
            c.panel_bg_muted.to_color32(),
            c.border_subtle.to_color32(),
            c.text_primary.to_color32(),
            cr_widget,
        );
        set_widget(
            &mut v.widgets.hovered,
            c.panel_bg_muted.to_color32(),
            c.selection_bg.to_color32(),
            c.border_strong.to_color32(),
            c.text_primary.to_color32(),
            cr_widget,
        );
        v.widgets.hovered.expansion = 1.0;
        set_widget(
            &mut v.widgets.active,
            c.selection_bg.to_color32(),
            c.accent.to_color32(),
            c.accent.to_color32(),
            c.text_primary.to_color32(),
            cr_widget,
        );
        v.widgets.active.expansion = 0.0;

        // ── Corner radius ──
        v.window_corner_radius = egui::CornerRadius::same(rad.panel as u8);

        // ── Shadow ──
        let sh = &theme.shadow;
        let shadow_color = egui::Color32::from_black_alpha((sh.panel_alpha * 255.0) as u8);
        v.window_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: sh.panel_blur as u8,
            spread: 0,
            color: shadow_color,
        };
        v.popup_shadow = egui::epaint::Shadow {
            offset: [0, 6],
            blur: sh.floating_blur as u8,
            spread: 0,
            color: shadow_color,
        };

        // ── Defaults ──
        let dark = c.window_bg.to_color32().r() < 128;
        v.dark_mode = dark;

        // ── Text styles (maps ThemeTypography -> egui::FontId) ──
        let t = &theme.typography;
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::proportional(t.title_size),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::proportional(t.body_size),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::monospace(t.body_size * 0.9),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::proportional(t.toolbar_size),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::proportional(t.caption_size),
            ),
        ]
        .into();

        // ── Spacing baseline (maps ThemeSpacing -> egui::Spacing boundaries) ──
        let s = &theme.spacing;
        let sp = &mut style.spacing;
        sp.item_spacing = egui::vec2(s.sm, s.sm);
        sp.button_padding = egui::vec2(s.sm, s.xs);
        sp.indent = s.lg;
        sp.menu_margin = egui::Margin::same(s.xs as i8);
        sp.window_margin = egui::Margin::same(s.sm as i8);

        ctx.set_style(style);
    }
}
