use eframe::egui;

use crate::ui::ThemeConfig;

pub struct TableOfContents;

impl TableOfContents {
    pub fn show(
        ctx: &egui::Context,
        chapter_titles: &[String],
        current_page: &mut usize,
        theme: &ThemeConfig,
    ) {
        let p = &theme.panel;

        egui::SidePanel::left("toc")
            .default_width(p.sidebar_default_width)
            .min_width(p.sidebar_min_width)
            .max_width(p.sidebar_max_width)
            .show(ctx, |ui| {
                ui.add_space(theme.spacing.lg);

                ui.horizontal(|ui| {
                    ui.add_space(theme.spacing.lg);
                    ui.heading("目录");
                });

                ui.add_space(theme.spacing.sm);
                ui.separator();
                ui.add_space(theme.spacing.sm);

                if chapter_titles.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(theme.spacing.lg);
                        ui.label("请打开书籍文件");
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("toc_scroll_area")
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(theme.spacing.sm);

                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width() - theme.spacing.sm);

                                    for (idx, title) in chapter_titles.iter().enumerate() {
                                        let is_selected = idx == *current_page;

                                        let response = ui.add(
                                            egui::Button::new(format!("{}.{}", idx + 1, title))
                                                .selected(is_selected)
                                                .frame(true),
                                        );

                                        if response.clicked() {
                                            *current_page = idx;
                                        }

                                        if is_selected {
                                            ui.scroll_to_rect(
                                                response.rect,
                                                Some(egui::Align::Center),
                                            );
                                        }
                                    }
                                });
                            });
                        });
                }

                ui.add_space(theme.spacing.lg);
            });
    }
}
