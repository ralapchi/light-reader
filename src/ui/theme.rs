use eframe::egui;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::domain::ThemeKind;

// ── ColorValue ──────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct ColorValue(egui::Color32);

impl ColorValue {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(egui::Color32::from_rgb(r, g, b))
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(egui::Color32::from_rgba_premultiplied(r, g, b, a))
    }

    pub fn to_color32(&self) -> egui::Color32 {
        self.0
    }

    pub fn hex(&self) -> String {
        let (r, g, b, a) = (self.0.r(), self.0.g(), self.0.b(), self.0.a());
        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }

    pub fn from_hex(s: &str) -> Result<Self, String> {
        let s = s.strip_prefix('#').unwrap_or(s);
        match s.len() {
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| "invalid hex".to_string())?;
                let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| "invalid hex".to_string())?;
                let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| "invalid hex".to_string())?;
                Ok(Self(egui::Color32::from_rgb(r, g, b)))
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| "invalid hex".to_string())?;
                let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| "invalid hex".to_string())?;
                let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| "invalid hex".to_string())?;
                let a = u8::from_str_radix(&s[6..8], 16).map_err(|_| "invalid hex".to_string())?;
                Ok(Self(egui::Color32::from_rgba_premultiplied(r, g, b, a)))
            }
            _ => Err("hex color must be 6 or 8 digits".to_string()),
        }
    }
}

impl Serialize for ColorValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.hex())
    }
}

impl<'de> Deserialize<'de> for ColorValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        ColorValue::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// ── ThemeColors (18 fields, SPEC 8.12) ──────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub window_bg: ColorValue,
    pub panel_bg: ColorValue,
    pub panel_bg_muted: ColorValue,
    pub reader_bg: ColorValue,
    pub text_primary: ColorValue,
    pub text_secondary: ColorValue,
    pub text_muted: ColorValue,
    pub accent: ColorValue,
    pub accent_hover: ColorValue,
    pub accent_pressed: ColorValue,
    pub border_subtle: ColorValue,
    pub border_strong: ColorValue,
    pub selection_bg: ColorValue,
    pub selection_text: ColorValue,
    pub success: ColorValue,
    pub warning: ColorValue,
    pub danger: ColorValue,
    pub focus_ring: ColorValue,
    pub sidebar_bg: ColorValue,
    pub sidebar_selected_bg: ColorValue,
    pub sidebar_selected_text: ColorValue,
    pub sidebar_hover_bg: ColorValue,
}

// ── ThemeSpacing (9 fields, SPEC 8.13) ──────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub reader_top_padding: f32,
    pub paragraph_gap: f32,
    pub panel_gap: f32,
    /// Vertical space after chapter title before first paragraph.
    pub chapter_title_bottom: f32,
    /// Vertical space at the end of a chapter (before next chapter).
    pub chapter_end_spacer: f32,
    /// Vertical padding for the loading/book-cover screen.
    pub loading_screen_spacer: f32,
    /// Opacity multiplier for paragraph highlight background (search/TTS).
    pub highlight_alpha: f32,
}

// ── ThemeTypography (8 fields, SPEC 8.14) ──────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub font_family_ui: String,
    pub font_family_reader: String,
    pub title_size: f32,
    pub section_title_size: f32,
    pub body_size: f32,
    pub caption_size: f32,
    pub toolbar_size: f32,
    pub line_height: f32,
}

// ── ThemeRadius (4 fields, SPEC 8.15, all f32) ─────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeRadius {
    pub button: f32,
    pub panel: f32,
    pub card: f32,
    pub input: f32,
}

// ── ThemeShadow (3 fields, SPEC 8.16, all f32) ─────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeShadow {
    pub panel_blur: f32,
    pub panel_alpha: f32,
    pub floating_blur: f32,
    pub card_shadow_alpha: u8,
    pub badge_bg_alpha: u8,
}

// ── ThemePanel (6 fields, SPEC 8.17, all f32) ──────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemePanel {
    pub top_bar_height: f32,
    pub status_bar_height: f32,
    pub sidebar_min_width: f32,
    pub sidebar_default_width: f32,
    pub sidebar_max_width: f32,
    pub content_max_width: f32,
    pub card_width: f32,
    pub card_height: f32,
    pub card_gap: f32,
}

// ── ThemeConfig (SPEC 8.11, name + 6 sub-structs) ──────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub typography: ThemeTypography,
    pub radius: ThemeRadius,
    pub shadow: ThemeShadow,
    pub panel: ThemePanel,
}

impl ThemeConfig {
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            colors: ThemeColors {
                window_bg: ColorValue::from_rgb(250, 249, 245),       // canvas
                panel_bg: ColorValue::from_rgb(237, 231, 218),        // surface-card (warmer, more depth)
                panel_bg_muted: ColorValue::from_rgb(245, 240, 232),  // surface-soft
                reader_bg: ColorValue::from_rgb(250, 249, 245),       // canvas
                text_primary: ColorValue::from_rgb(20, 20, 19),       // ink
                text_secondary: ColorValue::from_rgb(61, 61, 58),     // body
                text_muted: ColorValue::from_rgb(108, 106, 100),      // muted
                accent: ColorValue::from_rgb(204, 120, 92),           // primary / coral #cc785c
                accent_hover: ColorValue::from_rgb(169, 88, 62),      // primary-active #a9583e
                accent_pressed: ColorValue::from_rgb(169, 88, 62),    // primary-active
                border_subtle: ColorValue::from_rgb(232, 226, 218),   // hairline (softer)
                border_strong: ColorValue::from_rgb(204, 120, 92),    // coral
                selection_bg: ColorValue::from_rgb(232, 224, 210),    // surface-cream-strong
                selection_text: ColorValue::from_rgb(20, 20, 19),     // ink
                success: ColorValue::from_rgb(93, 184, 114),
                warning: ColorValue::from_rgb(212, 160, 23),
                danger: ColorValue::from_rgb(198, 69, 69),
                focus_ring: ColorValue::from_rgba(204, 120, 92, 80),  // coral at low alpha
                sidebar_bg: ColorValue::from_rgb(250, 249, 245),          // window_bg
                sidebar_selected_bg: ColorValue::from_rgba(204, 120, 92, 31), // accent ~12% alpha
                sidebar_selected_text: ColorValue::from_rgb(204, 120, 92), // accent
                sidebar_hover_bg: ColorValue::from_rgb(245, 240, 232),    // panel_bg_muted
            },
            spacing: default_spacing(),
            typography: default_typography(),
            radius: default_radius(),
            shadow: default_shadow(),
            panel: default_panel(),
        }
    }

    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            colors: ThemeColors {
                window_bg: ColorValue::from_rgb(24, 23, 21),          // surface-dark
                panel_bg: ColorValue::from_rgb(42, 40, 37),          // surface-dark-elevated (warmer depth)
                panel_bg_muted: ColorValue::from_rgb(31, 30, 27),    // surface-dark-soft
                reader_bg: ColorValue::from_rgb(24, 23, 21),         // surface-dark
                text_primary: ColorValue::from_rgb(250, 249, 245),   // on-dark / cream-tinted white
                text_secondary: ColorValue::from_rgb(184, 181, 173), // on-dark-soft (improved contrast)
                text_muted: ColorValue::from_rgb(108, 106, 100),
                accent: ColorValue::from_rgb(204, 120, 92),          // coral stays warm
                accent_hover: ColorValue::from_rgb(169, 88, 62),
                accent_pressed: ColorValue::from_rgb(169, 88, 62),
                border_subtle: ColorValue::from_rgb(55, 52, 48),
                border_strong: ColorValue::from_rgb(204, 120, 92),
                selection_bg: ColorValue::from_rgb(204, 120, 92),
                selection_text: ColorValue::from_rgb(24, 23, 21),
                success: ColorValue::from_rgb(93, 184, 114),
                warning: ColorValue::from_rgb(212, 160, 23),
                danger: ColorValue::from_rgb(198, 69, 69),
                focus_ring: ColorValue::from_rgba(204, 120, 92, 80),
                sidebar_bg: ColorValue::from_rgb(31, 30, 27),             // panel_bg_muted
                sidebar_selected_bg: ColorValue::from_rgba(204, 120, 92, 38), // accent ~15% alpha
                sidebar_selected_text: ColorValue::from_rgb(204, 120, 92), // accent
                sidebar_hover_bg: ColorValue::from_rgb(42, 40, 37),       // panel_bg
            },
            spacing: default_spacing(),
            typography: default_typography(),
            radius: default_radius(),
            shadow: default_shadow(),
            panel: default_panel(),
        }
    }

    pub fn sepia() -> Self {
        Self {
            name: "sepia".to_string(),
            colors: ThemeColors {
                window_bg: ColorValue::from_rgb(245, 240, 232),  // surface-soft warm
                panel_bg: ColorValue::from_rgb(239, 233, 222),   // surface-card warm
                panel_bg_muted: ColorValue::from_rgb(232, 224, 210), // surface-cream-strong
                reader_bg: ColorValue::from_rgb(250, 247, 240),  // slightly whiter for reading
                text_primary: ColorValue::from_rgb(37, 37, 35),  // body-strong
                text_secondary: ColorValue::from_rgb(108, 106, 100), // muted
                text_muted: ColorValue::from_rgb(142, 139, 130), // muted-soft
                accent: ColorValue::from_rgb(181, 98, 74),       // warmer coral for sepia
                accent_hover: ColorValue::from_rgb(155, 80, 56),
                accent_pressed: ColorValue::from_rgb(130, 66, 46),
                border_subtle: ColorValue::from_rgb(224, 215, 200),
                border_strong: ColorValue::from_rgb(181, 98, 74),
                selection_bg: ColorValue::from_rgb(224, 215, 200),
                selection_text: ColorValue::from_rgb(37, 37, 35),
                success: ColorValue::from_rgb(93, 184, 114),
                warning: ColorValue::from_rgb(212, 160, 23),
                danger: ColorValue::from_rgb(198, 69, 69),
                focus_ring: ColorValue::from_rgba(181, 98, 74, 80),
                sidebar_bg: ColorValue::from_rgb(245, 240, 232),          // window_bg
                sidebar_selected_bg: ColorValue::from_rgba(181, 98, 74, 31), // accent ~12% alpha
                sidebar_selected_text: ColorValue::from_rgb(181, 98, 74), // accent
                sidebar_hover_bg: ColorValue::from_rgb(232, 224, 210),    // panel_bg_muted
            },
            spacing: default_spacing(),
            typography: default_typography(),
            radius: default_radius(),
            shadow: default_shadow(),
            panel: default_panel(),
        }
    }

    pub fn paper() -> Self {
        Self {
            name: "paper".to_string(),
            colors: ThemeColors {
                window_bg: ColorValue::from_rgb(250, 249, 245),  // canvas
                panel_bg: ColorValue::from_rgb(245, 240, 232),   // surface-soft
                panel_bg_muted: ColorValue::from_rgb(239, 233, 222), // surface-card
                reader_bg: ColorValue::from_rgb(255, 255, 255),  // pure white for print-like
                text_primary: ColorValue::from_rgb(20, 20, 19),  // ink
                text_secondary: ColorValue::from_rgb(61, 61, 58), // body
                text_muted: ColorValue::from_rgb(108, 106, 100), // muted
                accent: ColorValue::from_rgb(108, 117, 125),     // neutral gray
                accent_hover: ColorValue::from_rgb(90, 98, 104),
                accent_pressed: ColorValue::from_rgb(73, 80, 87),
                border_subtle: ColorValue::from_rgb(230, 223, 216), // hairline
                border_strong: ColorValue::from_rgb(108, 117, 125),
                selection_bg: ColorValue::from_rgb(232, 224, 210),  // cream-strong
                selection_text: ColorValue::from_rgb(20, 20, 19),
                success: ColorValue::from_rgb(93, 184, 114),
                warning: ColorValue::from_rgb(212, 160, 23),
                danger: ColorValue::from_rgb(198, 69, 69),
                focus_ring: ColorValue::from_rgba(108, 117, 125, 80),
                sidebar_bg: ColorValue::from_rgb(250, 249, 245),          // window_bg
                sidebar_selected_bg: ColorValue::from_rgba(108, 117, 125, 31), // accent ~12% alpha
                sidebar_selected_text: ColorValue::from_rgb(108, 117, 125), // accent
                sidebar_hover_bg: ColorValue::from_rgb(245, 240, 232),    // panel_bg_muted
            },
            spacing: default_spacing(),
            typography: default_typography(),
            radius: default_radius(),
            shadow: default_shadow(),
            panel: default_panel(),
        }
    }
}

// ── Default helpers (shared by all presets) ─────────────

fn default_spacing() -> ThemeSpacing {
    ThemeSpacing {
        xxs: 4.0,
        xs: 8.0,
        sm: 12.0,
        md: 16.0,
        lg: 24.0,
        xl: 32.0,
        reader_top_padding: 24.0,
        paragraph_gap: 20.0,
        panel_gap: 12.0,
        chapter_title_bottom: 56.0,
        chapter_end_spacer: 80.0,
        loading_screen_spacer: 96.0,
        highlight_alpha: 0.15,
    }
}

fn default_typography() -> ThemeTypography {
    ThemeTypography {
        font_family_ui: "sans-serif".to_string(),
        font_family_reader: "serif".to_string(),
        title_size: 28.0,
        section_title_size: 22.0,
        body_size: 16.0,
        caption_size: 13.0,
        toolbar_size: 14.0,
        line_height: 1.55,
    }
}

fn default_radius() -> ThemeRadius {
    ThemeRadius {
        button: 8.0,
        panel: 12.0,
        card: 12.0,
        input: 8.0,
    }
}

fn default_shadow() -> ThemeShadow {
    ThemeShadow {
        panel_blur: 10.0,
        panel_alpha: 0.12,
        floating_blur: 14.0,
        card_shadow_alpha: 48,
        badge_bg_alpha: 120,
    }
}

fn default_panel() -> ThemePanel {
    ThemePanel {
        top_bar_height: 48.0,
        status_bar_height: 32.0,
        sidebar_min_width: 200.0,
        sidebar_default_width: 240.0,
        sidebar_max_width: 400.0,
        content_max_width: 720.0,
        card_width: 150.0,
        card_height: 200.0,
        card_gap: 16.0,
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::light()
    }
}

impl From<ThemeKind> for ThemeConfig {
    fn from(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Light => Self::light(),
            ThemeKind::Dark => Self::dark(),
            ThemeKind::Sepia => Self::sepia(),
            ThemeKind::Paper => Self::paper(),
            ThemeKind::Custom => Self::light(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_kind_maps_to_named_presets() {
        assert_eq!(ThemeConfig::from(ThemeKind::Light).name, "light");
        assert_eq!(ThemeConfig::from(ThemeKind::Dark).name, "dark");
        assert_eq!(ThemeConfig::from(ThemeKind::Sepia).name, "sepia");
        assert_eq!(ThemeConfig::from(ThemeKind::Paper).name, "paper");
    }
}
