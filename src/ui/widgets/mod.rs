use eframe::egui;

use crate::ui::ThemeConfig;

pub fn empty_state_with_button(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    button_text: &str,
    theme: &ThemeConfig,
) -> bool {
    let s = &theme.spacing;
    ui.vertical_centered(|ui| {
        ui.add_space(s.xl * 4.0);
        ui.heading(title);
        ui.add_space(s.sm);
        ui.label(description);
        ui.add_space(s.lg);
        ui.button(button_text).clicked()
    })
    .inner
}

pub fn loading_state(ui: &mut egui::Ui, title: &str, description: &str, theme: &ThemeConfig) {
    let s = &theme.spacing;
    ui.vertical_centered(|ui| {
        ui.add_space(s.xl * 4.0);
        ui.heading(title);
        ui.add_space(s.sm);
        ui.label(description);
        ui.add_space(s.sm);
        ui.add(egui::Spinner::new());
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
