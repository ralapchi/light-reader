use eframe::egui;

use crate::app::Action;
use crate::domain::app_state::AppState;
use crate::domain::theme_kind::ThemeKind;
use crate::ui::ThemeConfig;

pub fn settings_panel(
    ctx: &egui::Context,
    state: &AppState,
    theme: &ThemeConfig,
) -> Vec<Action> {
    let s = &theme.spacing;
    let t = &theme.typography;
    let settings = &state.reader_settings;
    let mut actions = Vec::new();

    egui::SidePanel::right("settings_panel")
        .default_width(320.0)
        .min_width(260.0)
        .max_width(480.0)
        .show(ctx, |ui| {
            ui.add_space(s.sm);

            // Header with close button
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("设置")
                        .size(t.section_title_size)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("关闭").clicked() {
                        actions.push(Action::ToggleSettingsPanel);
                    }
                });
            });

            ui.add_space(s.xs);
            ui.separator();
            ui.add_space(s.sm);

            egui::ScrollArea::vertical()
                .id_salt("settings_scroll")
                .show(ui, |ui| {
                    // Theme section
                    theme_section(ui, settings, theme, &mut actions);
                    ui.add_space(s.md);
                    ui.separator();
                    ui.add_space(s.md);

                    // Typography section
                    typography_section(ui, settings, theme, &mut actions);
                    ui.add_space(s.md);
                    ui.separator();
                    ui.add_space(s.md);

                    // Layout section
                    layout_section(ui, settings, theme, &mut actions);
                    ui.add_space(s.md);
                    ui.separator();
                    ui.add_space(s.md);

                    // Behavior section
                    behavior_section(ui, settings, theme, &mut actions);
                    ui.add_space(s.md);
                    ui.separator();
                    ui.add_space(s.md);

                    // Restore defaults
                    if ui.button("恢复默认设置").clicked() {
                        actions.push(Action::RestoreDefaultSettings);
                    }

                    ui.add_space(s.lg);
                });
        });

    actions
}

fn theme_section(
    ui: &mut egui::Ui,
    settings: &crate::domain::reader_settings::ReaderSettings,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.label(
        egui::RichText::new("主题")
            .size(t.body_size)
            .strong(),
    );
    ui.add_space(s.xs);

    ui.horizontal(|ui| {
        let themes = [
            ("浅色", ThemeKind::Light),
            ("深色", ThemeKind::Dark),
            ("护眼", ThemeKind::Sepia),
            ("纸张", ThemeKind::Paper),
        ];

        for (label, kind) in themes {
            let is_selected = settings.theme == kind;
            let btn = ui.selectable_label(is_selected, label);
            if btn.clicked() && !is_selected {
                actions.push(Action::ThemeChanged(kind));
            }
        }
    });
}

fn typography_section(
    ui: &mut egui::Ui,
    settings: &crate::domain::reader_settings::ReaderSettings,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.label(
        egui::RichText::new("排版")
            .size(t.body_size)
            .strong(),
    );
    ui.add_space(s.xs);

    // Font size
    let mut font_size = settings.font_size;
    ui.horizontal(|ui| {
        ui.label("字号");
        if ui
            .add(egui::Slider::new(&mut font_size, 10.0..=32.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::ReaderSettingChanged(
                "font_size".to_string(),
                font_size.to_string(),
            ));
        }
    });

    // Line height
    let mut line_height = settings.line_height;
    ui.horizontal(|ui| {
        ui.label("行距");
        if ui
            .add(egui::Slider::new(&mut line_height, 1.0..=3.0).step_by(0.1))
            .changed()
        {
            actions.push(Action::ReaderSettingChanged(
                "line_height".to_string(),
                line_height.to_string(),
            ));
        }
    });

    // Paragraph spacing
    let mut para_spacing = settings.paragraph_spacing;
    ui.horizontal(|ui| {
        ui.label("段间距");
        if ui
            .add(egui::Slider::new(&mut para_spacing, 0.0..=32.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::ReaderSettingChanged(
                "paragraph_spacing".to_string(),
                para_spacing.to_string(),
            ));
        }
    });
}

fn layout_section(
    ui: &mut egui::Ui,
    settings: &crate::domain::reader_settings::ReaderSettings,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.label(
        egui::RichText::new("布局")
            .size(t.body_size)
            .strong(),
    );
    ui.add_space(s.xs);

    // Content width
    let mut content_width = settings.content_width;
    ui.horizontal(|ui| {
        ui.label("正文宽度");
        if ui
            .add(egui::Slider::new(&mut content_width, 400.0..=1200.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::ReaderSettingChanged(
                "content_width".to_string(),
                content_width.to_string(),
            ));
        }
    });

    // Side margin
    let mut side_margin = settings.side_margin;
    ui.horizontal(|ui| {
        ui.label("侧边距");
        if ui
            .add(egui::Slider::new(&mut side_margin, 0.0..=100.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::ReaderSettingChanged(
                "side_margin".to_string(),
                side_margin.to_string(),
            ));
        }
    });

    // Sidebar width
    let mut toc_width = settings.toc_width;
    ui.horizontal(|ui| {
        ui.label("侧栏宽度");
        if ui
            .add(egui::Slider::new(&mut toc_width, 160.0..=480.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::ReaderSettingChanged(
                "toc_width".to_string(),
                toc_width.to_string(),
            ));
        }
    });
}

fn behavior_section(
    ui: &mut egui::Ui,
    settings: &crate::domain::reader_settings::ReaderSettings,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.label(
        egui::RichText::new("行为")
            .size(t.body_size)
            .strong(),
    );
    ui.add_space(s.xs);

    // Auto save progress
    let mut auto_save = settings.auto_save_progress;
    if ui
        .checkbox(&mut auto_save, "自动保存进度")
        .changed()
    {
        actions.push(Action::ReaderSettingChanged(
            "auto_save_progress".to_string(),
            auto_save.to_string(),
        ));
    }

    // Restore last position on startup
    let mut restore_last = settings.restore_last_position;
    if ui
        .checkbox(&mut restore_last, "启动时恢复上次阅读位置")
        .changed()
    {
        actions.push(Action::ReaderSettingChanged(
            "restore_last_position".to_string(),
            restore_last.to_string(),
        ));
    }

    // Show status bar
    let mut show_status = settings.show_status_bar;
    if ui
        .checkbox(&mut show_status, "显示状态栏")
        .changed()
    {
        actions.push(Action::ReaderSettingChanged(
            "show_status_bar".to_string(),
            show_status.to_string(),
        ));
    }

    // Show TOC sidebar
    let mut show_toc = settings.show_toc;
    if ui
        .checkbox(&mut show_toc, "显示目录侧栏")
        .changed()
    {
        actions.push(Action::ReaderSettingChanged(
            "show_toc".to_string(),
            show_toc.to_string(),
        ));
    }
}
