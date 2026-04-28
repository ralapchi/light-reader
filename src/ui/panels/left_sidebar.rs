use eframe::egui;

use crate::domain::enums::LeftPanelTab;
use crate::domain::toc_item::TocItem;
use crate::ui::ThemeConfig;

pub fn left_sidebar(
    ctx: &egui::Context,
    active_tab: &mut LeftPanelTab,
    toc: &[TocItem],
    theme: &ThemeConfig,
) {
    let p = &theme.panel;
    let s = &theme.spacing;

    egui::SidePanel::left("left_sidebar")
        .default_width(p.sidebar_default_width)
        .min_width(p.sidebar_min_width)
        .max_width(p.sidebar_max_width)
        .show(ctx, |ui| {
            ui.add_space(s.sm);

            ui.horizontal(|ui| {
                let tabs = ["目录", "书签", "最近"];
                let variants = [
                    LeftPanelTab::TableOfContents,
                    LeftPanelTab::Bookmarks,
                    LeftPanelTab::Recent,
                ];
                for (label, variant) in tabs.iter().zip(variants.iter()) {
                    let selected = *active_tab == *variant;
                    if ui.selectable_label(selected, *label).clicked() {
                        *active_tab = variant.clone();
                    }
                }
            });

            ui.add_space(s.xs);
            ui.separator();
            ui.add_space(s.xs);

            match active_tab {
                LeftPanelTab::TableOfContents => {
                    egui::ScrollArea::vertical()
                        .id_salt("toc_scroll")
                        .show(ui, |ui| {
                            for item in toc {
                                let label = format!(
                                    "{}. {}",
                                    item.chapter_index.unwrap_or(0) + 1,
                                    item.title
                                );
                                ui.label(label);
                                ui.add_space(s.xxs);
                            }
                        });
                }
                LeftPanelTab::Bookmarks => {
                    ui.label("（书签功能将在 Phase 5 实现）");
                }
                LeftPanelTab::Recent => {
                    ui.label("（最近阅读将在 Phase 5 完善）");
                }
            }
        });
}
