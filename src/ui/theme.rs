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
                window_bg: ColorValue::from_rgb(245, 242, 237),
                panel_bg: ColorValue::from_rgb(237, 233, 227),
                panel_bg_muted: ColorValue::from_rgb(229, 224, 216),
                reader_bg: ColorValue::from_rgb(255, 255, 255),
                text_primary: ColorValue::from_rgb(26, 26, 26),
                text_secondary: ColorValue::from_rgb(120, 120, 120),
                text_muted: ColorValue::from_rgb(160, 160, 160),
                accent: ColorValue::from_rgb(59, 130, 246),
                accent_hover: ColorValue::from_rgb(37, 99, 235),
                accent_pressed: ColorValue::from_rgb(29, 78, 216),
                border_subtle: ColorValue::from_rgb(224, 224, 224),
                border_strong: ColorValue::from_rgb(59, 130, 246),
                selection_bg: ColorValue::from_rgb(219, 234, 254),
                selection_text: ColorValue::from_rgb(26, 26, 26),
                success: ColorValue::from_rgb(16, 185, 129),
                warning: ColorValue::from_rgb(245, 158, 11),
                danger: ColorValue::from_rgb(239, 68, 68),
                focus_ring: ColorValue::from_rgba(59, 130, 246, 120),
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
                window_bg: ColorValue::from_rgb(26, 26, 46),
                panel_bg: ColorValue::from_rgb(22, 33, 62),
                panel_bg_muted: ColorValue::from_rgb(30, 41, 72),
                reader_bg: ColorValue::from_rgb(15, 15, 35),
                text_primary: ColorValue::from_rgb(224, 224, 224),
                text_secondary: ColorValue::from_rgb(140, 140, 160),
                text_muted: ColorValue::from_rgb(107, 107, 128),
                accent: ColorValue::from_rgb(79, 195, 247),
                accent_hover: ColorValue::from_rgb(41, 182, 246),
                accent_pressed: ColorValue::from_rgb(3, 155, 229),
                border_subtle: ColorValue::from_rgb(55, 65, 81),
                border_strong: ColorValue::from_rgb(79, 195, 247),
                selection_bg: ColorValue::from_rgb(30, 58, 95),
                selection_text: ColorValue::from_rgb(224, 224, 224),
                success: ColorValue::from_rgb(52, 211, 153),
                warning: ColorValue::from_rgb(251, 191, 36),
                danger: ColorValue::from_rgb(248, 113, 113),
                focus_ring: ColorValue::from_rgba(79, 195, 247, 120),
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
                window_bg: ColorValue::from_rgb(245, 236, 215),
                panel_bg: ColorValue::from_rgb(235, 219, 179),
                panel_bg_muted: ColorValue::from_rgb(225, 206, 160),
                reader_bg: ColorValue::from_rgb(250, 242, 225),
                text_primary: ColorValue::from_rgb(74, 55, 40),
                text_secondary: ColorValue::from_rgb(140, 115, 90),
                text_muted: ColorValue::from_rgb(168, 152, 128),
                accent: ColorValue::from_rgb(160, 82, 45),
                accent_hover: ColorValue::from_rgb(139, 69, 19),
                accent_pressed: ColorValue::from_rgb(107, 52, 16),
                border_subtle: ColorValue::from_rgb(200, 180, 150),
                border_strong: ColorValue::from_rgb(160, 120, 70),
                selection_bg: ColorValue::from_rgb(210, 196, 168),
                selection_text: ColorValue::from_rgb(74, 55, 40),
                success: ColorValue::from_rgb(107, 142, 35),
                warning: ColorValue::from_rgb(205, 133, 63),
                danger: ColorValue::from_rgb(178, 34, 34),
                focus_ring: ColorValue::from_rgba(160, 82, 45, 120),
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
                window_bg: ColorValue::from_rgb(248, 249, 250),
                panel_bg: ColorValue::from_rgb(233, 236, 239),
                panel_bg_muted: ColorValue::from_rgb(222, 226, 230),
                reader_bg: ColorValue::from_rgb(255, 255, 255),
                text_primary: ColorValue::from_rgb(52, 58, 64),
                text_secondary: ColorValue::from_rgb(134, 142, 150),
                text_muted: ColorValue::from_rgb(173, 181, 189),
                accent: ColorValue::from_rgb(108, 117, 125),
                accent_hover: ColorValue::from_rgb(90, 98, 104),
                accent_pressed: ColorValue::from_rgb(73, 80, 87),
                border_subtle: ColorValue::from_rgb(222, 226, 230),
                border_strong: ColorValue::from_rgb(108, 117, 125),
                selection_bg: ColorValue::from_rgb(206, 212, 218),
                selection_text: ColorValue::from_rgb(52, 58, 64),
                success: ColorValue::from_rgb(108, 140, 108),
                warning: ColorValue::from_rgb(184, 160, 96),
                danger: ColorValue::from_rgb(160, 96, 96),
                focus_ring: ColorValue::from_rgba(108, 117, 125, 120),
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
        xxs: 2.0,
        xs: 4.0,
        sm: 8.0,
        md: 12.0,
        lg: 16.0,
        xl: 24.0,
        reader_top_padding: 20.0,
        paragraph_gap: 16.0,
        panel_gap: 8.0,
        chapter_title_bottom: 48.0,
        chapter_end_spacer: 72.0,
        loading_screen_spacer: 96.0,
        highlight_alpha: 0.15,
    }
}

fn default_typography() -> ThemeTypography {
    ThemeTypography {
        font_family_ui: "sans-serif".to_string(),
        font_family_reader: "sans-serif".to_string(),
        title_size: 28.0,
        section_title_size: 22.0,
        body_size: 16.0,
        caption_size: 12.0,
        toolbar_size: 14.0,
        line_height: 1.6,
    }
}

fn default_radius() -> ThemeRadius {
    ThemeRadius {
        button: 6.0,
        panel: 8.0,
        card: 4.0,
        input: 4.0,
    }
}

fn default_shadow() -> ThemeShadow {
    ThemeShadow {
        panel_blur: 8.0,
        panel_alpha: 0.12,
        floating_blur: 12.0,
        card_shadow_alpha: 40,
        badge_bg_alpha: 120,
    }
}

fn default_panel() -> ThemePanel {
    ThemePanel {
        top_bar_height: 48.0,
        status_bar_height: 32.0,
        sidebar_min_width: 200.0,
        sidebar_default_width: 280.0,
        sidebar_max_width: 400.0,
        content_max_width: 720.0,
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
