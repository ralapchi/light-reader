pub mod book_card;
pub mod library_detail;
pub use book_card::book_card;

use eframe::egui;
use crate::ui::ThemeConfig;

/// 渲染带搜索关键词高亮的文本
///
/// 在 `ui.horizontal_wrapped` 中将 `text` 按 `keyword` 分段渲染，
/// 匹配部分使用 `background_color` 高亮。
///
/// - `font_id`: 指定字体（为 None 时使用 `font_size` 构建默认字体）
/// - `line_height`: 行高（为 None 时使用默认值）
/// - `case_sensitive`: 是否区分大小写
pub fn render_highlighted_text(
    ui: &mut egui::Ui,
    text: &str,
    keyword: &str,
    font_size: f32,
    font_id: Option<&egui::FontId>,
    line_height: Option<f32>,
    theme: &ThemeConfig,
    case_sensitive: bool,
) {
    let highlight_color = theme.colors.accent.to_color32().gamma_multiply(0.3);
    let fallback_font = egui::FontId::new(font_size, egui::FontFamily::Proportional);
    let fid = font_id.unwrap_or(&fallback_font);

    ui.horizontal_wrapped(|ui| {
        let (search_text, search_keyword) = if case_sensitive {
            (text.to_string(), keyword.to_string())
        } else {
            (text.to_lowercase(), keyword.to_lowercase())
        };

        let mut last_end = 0;

        for (start, _) in search_text.match_indices(&search_keyword) {
            if start > last_end {
                let mut rt = egui::RichText::new(&text[last_end..start]).font(fid.clone());
                if let Some(lh) = line_height {
                    rt = rt.line_height(Some(lh));
                }
                ui.label(rt);
            }

            let end = start + keyword.len();
            let mut rt = egui::RichText::new(&text[start..end])
                .font(fid.clone())
                .background_color(highlight_color);
            if let Some(lh) = line_height {
                rt = rt.line_height(Some(lh));
            }
            ui.label(rt);
            last_end = end;
        }

        if last_end < text.len() {
            let mut rt = egui::RichText::new(&text[last_end..]).font(fid.clone());
            if let Some(lh) = line_height {
                rt = rt.line_height(Some(lh));
            }
            ui.label(rt);
        }
    });
}

pub fn error_state(
    ui: &mut egui::Ui,
    title: &str,
    message: &str,
    retry_text: &str,
    reopen_text: &str,
    theme: &ThemeConfig,
) -> (bool, bool) {
    let s = &theme.spacing;
    let mut retry = false;
    let mut reopen = false;
    ui.vertical_centered(|ui| {
        ui.add_space(s.xl * 4.0);
        ui.colored_label(theme.colors.danger.to_color32(), title);
        ui.add_space(s.sm);
        ui.label(message);
        ui.add_space(s.lg);
        ui.horizontal(|ui| {
            if ui.button(retry_text).clicked() {
                retry = true;
            }
            if ui.button(reopen_text).clicked() {
                reopen = true;
            }
        });
    });
    (retry, reopen)
}
