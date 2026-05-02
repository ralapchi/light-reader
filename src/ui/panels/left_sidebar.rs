use eframe::egui;

use crate::app::Action;
use crate::domain::bookmark::Bookmark;
use crate::domain::enums::LeftPanelTab;
use crate::domain::recent_book_item::RecentBookItem;
use crate::domain::toc_item::TocItem;
use crate::ui::ThemeConfig;

pub fn left_sidebar(
    ctx: &egui::Context,
    active_tab: &LeftPanelTab,
    toc: &[TocItem],
    bookmarks: &[Bookmark],
    recent_books: &[RecentBookItem],
    theme: &ThemeConfig,
    current_toc_width: f32,
) -> Option<Action> {
    let p = &theme.panel;
    let s = &theme.spacing;
    let mut action = None;

    let panel_response = egui::SidePanel::left("left_sidebar")
        .default_width(current_toc_width)
        .width_range(p.sidebar_min_width..=p.sidebar_max_width)
        .resizable(true)
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
                        action = Some(Action::SwitchLeftPanelTab(variant.clone()));
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
                                render_toc_item(ui, item, theme, &mut action, 0);
                            }
                        });
                }
                LeftPanelTab::Bookmarks => {
                    if bookmarks.is_empty() {
                        ui.add_space(s.xl);
                        ui.vertical_centered(|ui| {
                            ui.label("暂无书签");
                            ui.add_space(s.xs);
                            ui.label(
                                egui::RichText::new("点击工具栏书签按钮添加")
                                    .size(theme.typography.caption_size),
                            );
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .id_salt("bookmarks_scroll")
                            .show(ui, |ui| {
                                for bookmark in bookmarks {
                                    let resp = ui
                                        .group(|ui| {
                                            ui.label(
                                                egui::RichText::new(&bookmark.title).strong(),
                                            );
                                            if !bookmark.snippet.is_empty() {
                                                ui.add_space(s.xxs);
                                                ui.label(
                                                    egui::RichText::new(&bookmark.snippet)
                                                        .size(theme.typography.caption_size),
                                                );
                                            }
                                        })
                                        .response;
                                    if resp.interact(egui::Sense::click()).clicked() {
                                        action =
                                            Some(Action::JumpToBookmark(bookmark.id.clone()));
                                    }
                                    ui.add_space(s.xxs);
                                }
                            });
                    }
                }
                LeftPanelTab::Recent => {
                    if recent_books.is_empty() {
                        ui.add_space(s.xl);
                        ui.vertical_centered(|ui| {
                            ui.label("暂无阅读记录");
                            ui.add_space(s.xs);
                            ui.label(
                                egui::RichText::new("打开书籍后将自动记录")
                                    .size(theme.typography.caption_size),
                            );
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .id_salt("recent_scroll")
                            .show(ui, |ui| {
                                for item in recent_books {
                                    let resp = ui
                                        .group(|ui| {
                                            ui.label(egui::RichText::new(&item.title).strong());
                                            if let Some(author) = &item.author {
                                                ui.add_space(s.xxs);
                                                ui.label(
                                                    egui::RichText::new(author)
                                                        .size(theme.typography.caption_size),
                                                );
                                            }
                                            ui.add_space(s.xxs);
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{} · {:.0}%",
                                                    item.format,
                                                    item.last_progress_percent * 100.0
                                                ))
                                                .size(theme.typography.caption_size),
                                            );
                                        })
                                        .response;
                                    if resp.interact(egui::Sense::click()).clicked() {
                                        action = Some(Action::RecentBookSelected(
                                            item.book_id.clone(),
                                        ));
                                    }
                                    ui.add_space(s.xxs);
                                }
                            });
                    }
                }
            }
        });

    // Check if panel was resized and emit width change action
    let new_width = panel_response.response.rect.width();
    if (new_width - current_toc_width).abs() > 1.0 {
        action = Some(Action::ReaderSettingChanged(
            "toc_width".to_string(),
            new_width.to_string(),
        ));
    }

    action
}

/// 递归渲染目录项及其子项
fn render_toc_item(
    ui: &mut egui::Ui,
    item: &TocItem,
    theme: &ThemeConfig,
    action: &mut Option<Action>,
    indent_level: u8,
) {
    let s = &theme.spacing;
    let indent = indent_level as f32 * s.lg;

    ui.horizontal(|ui| {
        ui.add_space(indent);

        let label = if let Some(idx) = item.chapter_index {
            format!("{}. {}", idx + 1, item.title)
        } else {
            item.title.clone()
        };

        if ui.selectable_label(false, label).clicked() {
            if let Some(idx) = item.chapter_index {
                *action = Some(Action::GoToChapter(idx));
            }
        }
    });
    ui.add_space(s.xxs);

    // 递归渲染子项
    for child in &item.children {
        render_toc_item(ui, child, theme, action, indent_level + 1);
    }
}
