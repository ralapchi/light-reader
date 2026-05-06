use eframe::egui;

use crate::ui::ThemeConfig;

pub fn status_bar(
    ui: &mut egui::Ui,
    progress: f32,
    chapter_pos: &str,
    char_count: usize,
    theme: &ThemeConfig,
) {
    let s = &theme.spacing;
    ui.horizontal(|ui| {
        ui.add_space(s.sm);
        ui.label(format!("进度: {:.0}%", progress * 100.0));
        ui.add_space(s.lg);
        ui.separator();
        ui.add_space(s.lg);
        ui.label(chapter_pos);
        if char_count > 0 {
            ui.add_space(s.lg);
            ui.separator();
            ui.add_space(s.lg);
            ui.label(format!("{} 字", char_count));
        }
    });
}
