use eframe::egui;

use crate::domain::toc_item::TocItem;
use crate::ui::ThemeConfig;

pub struct TableOfContents;

impl TableOfContents {
    pub fn show(
        ctx: &egui::Context,
        toc: &[TocItem],
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

                if toc.is_empty() {
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

                                    for item in toc {
                                        let chapter_index =
                                            item.chapter_index.unwrap_or(*current_page);
                                        let is_selected = chapter_index == *current_page;
                                        let label = if let Some(index) = item.chapter_index {
                                            format!("{}. {}", index + 1, item.title)
                                        } else {
                                            item.title.clone()
                                        };

                                        let response = ui.add(
                                            egui::Button::new(label)
                                                .selected(is_selected)
                                                .frame(true),
                                        );

                                        if response.clicked() {
                                            *current_page = chapter_index;
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
