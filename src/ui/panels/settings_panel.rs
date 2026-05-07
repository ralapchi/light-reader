use eframe::egui;

use crate::app::Action;
use crate::app::actions::ReaderSettingUpdate;
use crate::domain::reader_settings::ReaderSettings;
use crate::domain::theme_kind::ThemeKind;
use crate::ui::ThemeConfig;

/// Lightweight read-only props for SettingsPanel, derived from AppState.
pub struct SettingsPanelProps<'a> {
    pub reader_settings: &'a ReaderSettings,
}

pub fn settings_panel(
    ctx: &egui::Context,
    props: &SettingsPanelProps<'_>,
    theme: &ThemeConfig,
) -> Vec<Action> {
    let s = &theme.spacing;
    let t = &theme.typography;
    let settings = props.reader_settings;
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
                    // 外观：主题、字体、字号
                    appearance_section(ui, settings, theme, &mut actions);
                    ui.add_space(s.md);
                    ui.separator();
                    ui.add_space(s.md);

                    // 排版：行距、段距、正文宽度、侧边距
                    typography_section(ui, settings, theme, &mut actions);
                    ui.add_space(s.md);
                    ui.separator();
                    ui.add_space(s.md);

                    // 阅读行为：目录、状态栏、章节进度、启动恢复
                    reading_behavior_section(ui, settings, theme, &mut actions);
                    ui.add_space(s.md);
                    ui.separator();
                    ui.add_space(s.md);

                    // 高级：平滑滚动、窗口内边距、自动保存、恢复默认
                    advanced_section(ui, settings, theme, &mut actions);

                    ui.add_space(s.lg);
                });
        });

    actions
}

fn appearance_section(
    ui: &mut egui::Ui,
    settings: &crate::domain::reader_settings::ReaderSettings,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.label(
        egui::RichText::new("外观")
            .size(t.body_size)
            .strong(),
    );
    ui.add_space(s.xs);

    // Theme
    ui.horizontal(|ui| {
        ui.label("主题");
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

    // Font size
    let mut font_size = settings.font_size;
    ui.horizontal(|ui| {
        ui.label("字号");
        if ui
            .add(egui::Slider::new(&mut font_size, 10.0..=32.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetFontSize(font_size)));
        }
    });

    // Font family
    let font_options = [
        ("sans-serif", "无衬线"),
        ("serif", "衬线"),
        ("monospace", "等宽"),
    ];
    let current_font_label = font_options
        .iter()
        .find(|(key, _)| *key == settings.font_family)
        .map(|(_, label)| *label)
        .unwrap_or(&settings.font_family);

    ui.horizontal(|ui| {
        ui.label("字体");
        egui::ComboBox::from_id_salt("font_family_select")
            .selected_text(current_font_label)
            .show_ui(ui, |ui| {
                for (key, label) in font_options {
                    let is_selected = settings.font_family == key;
                    if ui.selectable_label(is_selected, label).clicked() && !is_selected {
                        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetFontFamily(key.to_string())));
                    }
                }
            });
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

    // Line height
    let mut line_height = settings.line_height;
    ui.horizontal(|ui| {
        ui.label("行距");
        if ui
            .add(egui::Slider::new(&mut line_height, 1.0..=3.0).step_by(0.1))
            .changed()
        {
            actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetLineHeight(line_height)));
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
            actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetParagraphSpacing(para_spacing)));
        }
    });

    // Content width
    let mut content_width = settings.content_width;
    ui.horizontal(|ui| {
        ui.label("正文宽度");
        if ui
            .add(egui::Slider::new(&mut content_width, 400.0..=1200.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetContentWidth(content_width)));
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
            actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetSideMargin(side_margin)));
        }
    });
}

fn reading_behavior_section(
    ui: &mut egui::Ui,
    settings: &crate::domain::reader_settings::ReaderSettings,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.label(
        egui::RichText::new("阅读行为")
            .size(t.body_size)
            .strong(),
    );
    ui.add_space(s.xs);

    // Show TOC sidebar
    let mut show_toc = settings.show_toc;
    if ui.checkbox(&mut show_toc, "显示目录侧栏").changed() {
        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetShowToc(show_toc)));
    }

    // Show status bar
    let mut show_status = settings.show_status_bar;
    if ui.checkbox(&mut show_status, "显示状态栏").changed() {
        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetShowStatusBar(show_status)));
    }

    // Show chapter progress
    let mut show_chapter_progress = settings.show_chapter_progress;
    if ui.checkbox(&mut show_chapter_progress, "显示章节进度").changed() {
        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetShowChapterProgress(show_chapter_progress)));
    }

    // Restore last position on startup
    let mut restore_last = settings.restore_last_position;
    if ui.checkbox(&mut restore_last, "启动时恢复上次阅读位置").changed() {
        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetRestoreLastPosition(restore_last)));
    }

    // Open last book on startup
    let mut open_last = settings.open_last_book_on_startup;
    if ui.checkbox(&mut open_last, "启动时恢复最近阅读").changed() {
        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetOpenLastBookOnStartup(open_last)));
    }

    // Auto page turn at chapter end
    let mut auto_turn = settings.auto_page_turn;
    if ui.checkbox(&mut auto_turn, "章节末尾自动翻页").changed() {
        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetAutoPageTurn(auto_turn)));
    }
}

fn advanced_section(
    ui: &mut egui::Ui,
    settings: &crate::domain::reader_settings::ReaderSettings,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.label(
        egui::RichText::new("系统")
            .size(t.body_size)
            .strong(),
    );
    ui.add_space(s.xs);

    // Auto save progress
    let mut auto_save = settings.auto_save_progress;
    if ui.checkbox(&mut auto_save, "自动保存进度").changed() {
        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetAutoSaveProgress(auto_save)));
    }

    // Smooth scroll
    let mut smooth_scroll = settings.smooth_scroll;
    if ui.checkbox(&mut smooth_scroll, "平滑滚动").changed() {
        actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetSmoothScroll(smooth_scroll)));
    }

    // Sidebar width
    let mut toc_width = settings.toc_width;
    ui.horizontal(|ui| {
        ui.label("侧栏宽度");
        if ui
            .add(egui::Slider::new(&mut toc_width, 160.0..=480.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetTocWidth(toc_width)));
        }
    });

    // Window padding
    let mut window_padding = settings.window_padding;
    ui.horizontal(|ui| {
        ui.label("主区域内边距");
        if ui
            .add(egui::Slider::new(&mut window_padding, 0.0..=32.0).suffix(" px"))
            .changed()
        {
            actions.push(Action::UpdateReaderSetting(ReaderSettingUpdate::SetWindowPadding(window_padding)));
        }
    });

    ui.add_space(s.md);

    // Restore defaults
    if ui.button("恢复默认设置").clicked() {
        actions.push(Action::RestoreDefaultSettings);
    }
}
