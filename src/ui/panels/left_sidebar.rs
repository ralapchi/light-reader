use eframe::egui;

use crate::app::actions::ReaderSettingUpdate;
use crate::app::Action;
use crate::domain::bookmark::Bookmark;
use crate::domain::enums::LeftPanelTab;
use crate::domain::recent_book_item::RecentBookItem;
use crate::domain::toc_item::TocItem;
use crate::ui::ThemeConfig;

#[allow(dead_code)]
pub fn left_sidebar(
    ctx: &egui::Context,
    active_tab: &LeftPanelTab,
    toc: &[TocItem],
    bookmarks: &[Bookmark],
    recent_books: &[RecentBookItem],
    theme: &ThemeConfig,
    current_toc_width: f32,
    current_chapter_index: usize,
) -> Vec<Action> {
    let p = &theme.panel;
    let s = &theme.spacing;
    let mut actions = Vec::new();

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
                        actions.push(Action::SwitchLeftPanelTab(variant.clone()));
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
                                render_toc_item(ui, item, theme, &mut actions, 0, current_chapter_index);
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
                                    let snippet = if bookmark.snippet.is_empty() {
                                        None
                                    } else {
                                        Some(bookmark.snippet.as_str())
                                    };
                                    render_sidebar_card(
                                        ui,
                                        theme,
                                        &bookmark.title,
                                        snippet,
                                        Some(Action::RemoveBookmark(bookmark.id.clone())),
                                        Some(Action::JumpToBookmark(bookmark.id.clone())),
                                        &mut actions,
                                    );
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
                                    let subtitle = {
                                        let mut parts = Vec::new();
                                        if let Some(author) = &item.author {
                                            parts.push(author.clone());
                                        }
                                        parts.push(format!(
                                            "{} · {:.0}%",
                                            item.format,
                                            item.last_progress_percent * 100.0
                                        ));
                                        parts.join(" · ")
                                    };
                                    render_sidebar_card(
                                        ui,
                                        theme,
                                        &item.title,
                                        Some(&subtitle),
                                        Some(Action::RemoveRecentBook(item.book_id.clone())),
                                        Some(Action::RecentBookSelected(item.book_id.clone())),
                                        &mut actions,
                                    );
                                }
                            });
                    }
                }
            }
        });

    // Check if panel was resized and emit width change action
    let new_width = panel_response.response.rect.width();
    if (new_width - current_toc_width).abs() > 1.0 {
        actions.push(Action::UpdateReaderSetting(
            ReaderSettingUpdate::SetTocWidth(new_width),
        ));
    }

    actions
}

/// 渲染侧栏卡片项（统一密度：标题 strong + 副标题 caption + 右侧删除按钮）
fn render_sidebar_card(
    ui: &mut egui::Ui,
    theme: &ThemeConfig,
    title: &str,
    subtitle: Option<&str>,
    delete_action: Option<Action>,
    click_action: Option<Action>,
    actions: &mut Vec<Action>,
) {
    let s = &theme.spacing;
    let resp = ui
        .group(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title).strong());
                    if let Some(sub) = subtitle {
                        if !sub.is_empty() {
                            ui.add_space(s.xxs);
                            ui.label(
                                egui::RichText::new(sub).size(theme.typography.caption_size),
                            );
                        }
                    }
                });
                if let Some(del_action) = delete_action {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("×")
                                        .size(theme.typography.caption_size)
                                        .color(theme.colors.danger.to_color32()),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE),
                            )
                            .clicked()
                        {
                            actions.push(del_action);
                        }
                    });
                }
            });
        })
        .response;
    if let Some(click_action) = click_action {
        if resp.interact(egui::Sense::click()).clicked() {
            actions.push(click_action);
        }
    }
    ui.add_space(s.xxs);
}

/// 递归渲染目录项及其子项
fn render_toc_item(
    ui: &mut egui::Ui,
    item: &TocItem,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
    indent_level: u8,
    current_chapter_index: usize,
) {
    let s = &theme.spacing;
    let indent = indent_level as f32 * s.md;

    let label = if let Some(idx) = item.chapter_index {
        format!("{}. {}", idx + 1, item.title)
    } else {
        item.title.clone()
    };

    let is_current = item.chapter_index == Some(current_chapter_index);

    let resp = ui
        .group(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(indent);
                let text = if is_current {
                    egui::RichText::new(label)
                        .strong()
                        .color(theme.colors.accent.to_color32())
                } else {
                    egui::RichText::new(label)
                };
                ui.label(text);
            });
        })
        .response;

    if is_current {
        ui.painter().rect_stroke(
            resp.rect,
            egui::CornerRadius::same(4),
            egui::Stroke::new(1.5, theme.colors.accent.to_color32()),
            egui::StrokeKind::Outside,
        );
    }

    if resp.interact(egui::Sense::click()).clicked() {
        if let Some(idx) = item.chapter_index {
            actions.push(Action::GoToChapter(idx));
        }
    }

    ui.add_space(s.xxs);

    // 递归渲染子项
    for child in &item.children {
        render_toc_item(ui, child, theme, actions, indent_level + 1, current_chapter_index);
    }
}
