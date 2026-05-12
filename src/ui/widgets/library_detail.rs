use eframe::egui;

use crate::app::Action;
use crate::domain::library_item::LibraryItem;
use crate::ui::image_cache::IMG_CACHE;
use crate::ui::widgets::book_card;
use crate::ui::ThemeConfig;

/// Show a detail overlay for a selected library book.
pub fn library_detail_panel(
    ctx: &egui::Context,
    item: &LibraryItem,
    theme: &ThemeConfig,
) -> Vec<Action> {
    let s = &theme.spacing;
    let colors = &theme.colors;
    let typo = &theme.typography;
    let r = &theme.radius;
    let mut actions = Vec::new();

    let mut open = true;
    egui::Window::new(format!("{} - 详情", item.title))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(420.0)
        .frame(egui::Frame::window(&ctx.style())
            .corner_radius(egui::CornerRadius::same(r.panel as u8))
            .shadow(egui::epaint::Shadow {
                offset: [0, 8],
                blur: theme.shadow.floating_blur as u8,
                spread: 0,
                color: egui::Color32::from_black_alpha(40),
            })
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(s.md);

            // Header: cover + title + close button
            ui.horizontal(|ui| {
                // Larger cover (120x160)
                let mini_cover_size = egui::Vec2::new(120.0, 160.0);
                let mini_cover_rect = egui::Rect::from_min_size(ui.next_widget_position(), mini_cover_size);
                let (_rect, _resp) = ui.allocate_exact_size(mini_cover_size, egui::Sense::hover());
                if ui.is_rect_visible(mini_cover_rect) {
                    let painter = ui.painter_at(mini_cover_rect);
                    let mut drew_real = false;
                    if let Some(tex) = IMG_CACHE.with(|c| c.borrow_mut().cover_texture(
                        ui.ctx(), &item.book_id, item.cover_cache_key.as_deref(),
                    )) {
                        painter.image(
                            tex.id(), mini_cover_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                        drew_real = true;
                    }
                    if !drew_real {
                        let cover_color = book_card::format_cover_color(&item.format);
                        painter.rect_filled(mini_cover_rect, egui::CornerRadius::same(6), cover_color);
                        painter.rect_stroke(mini_cover_rect, egui::CornerRadius::same(6),
                            egui::Stroke::new(1.0, colors.border_subtle.to_color32()), egui::StrokeKind::Inside);
                        painter.text(
                            mini_cover_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            book_card::format_tag(&item.format),
                            egui::FontId::new(typo.body_size, egui::FontFamily::Proportional),
                            egui::Color32::WHITE,
                        );
                    }
                }

                ui.add_space(s.md);

                // Right side: title, author, format
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&item.title).size(typo.section_title_size).strong());
                    ui.add_space(s.xxs);
                    if let Some(ref author) = item.author {
                        ui.label(egui::RichText::new(author).size(typo.body_size).color(colors.text_secondary.to_color32()));
                    }
                    ui.add_space(s.xs);
                    ui.label(egui::RichText::new(format!("格式: {}", book_card::format_tag(&item.format)))
                        .size(typo.caption_size).color(colors.text_muted.to_color32()));

                    // Progress bar
                    ui.add_space(s.sm);
                    let progress = item.progress_percent;
                    let bar_width = 200.0;
                    let bar_height = 6.0;
                    let bar_pos = ui.next_widget_position();
                    let bar_rect = egui::Rect::from_min_size(bar_pos, egui::Vec2::new(bar_width, bar_height));
                    let (_bar_r, _bar_resp) = ui.allocate_exact_size(egui::Vec2::new(bar_width, bar_height), egui::Sense::hover());
                    if ui.is_rect_visible(bar_rect) {
                        let painter = ui.painter_at(bar_rect);
                        painter.rect_filled(bar_rect, egui::CornerRadius::same(3), colors.panel_bg_muted.to_color32());
                        if progress > 0.0 {
                            let fill_width = bar_width * progress.min(1.0);
                            let fill_rect = egui::Rect::from_min_size(bar_pos, egui::Vec2::new(fill_width, bar_height));
                            painter.rect_filled(fill_rect, egui::CornerRadius::same(3), colors.accent.to_color32());
                        }
                    }
                    ui.label(egui::RichText::new(format!("{:.1}%", progress * 100.0))
                        .size(typo.caption_size).color(colors.text_muted.to_color32()));
                });
            });

            ui.add_space(s.md);
            ui.separator();
            ui.add_space(s.sm);

            // Section: 基本信息
            section_header(ui, "基本信息", theme);
            ui.add_space(s.xxs);
            detail_row(ui, "书籍 ID", &item.book_id, theme);
            detail_row(ui, "文件路径", &item.source_path, theme);
            detail_row(ui, "导入时间", &format_date(&item.imported_at), theme);
            if let Some(ref opened) = item.last_opened_at {
                detail_row(ui, "最近打开", &format_date(opened), theme);
            }
            detail_row(ui, "总章节数", &item.chapter_count.to_string(), theme);

            ui.add_space(s.sm);
            ui.separator();
            ui.add_space(s.sm);

            // Section: 阅读统计
            section_header(ui, "阅读统计", theme);
            ui.add_space(s.xxs);
            detail_row(ui, "总阅读时长", &format_seconds(item.stats.total_read_seconds), theme);
            detail_row(ui, "书签数", &item.stats.bookmark_count.to_string(), theme);
            if let Some(ref last) = item.stats.last_read_at {
                detail_row(ui, "最后阅读", &format_date(last), theme);
            }
            if let Some(idx) = item.stats.last_chapter_index {
                detail_row(ui, "最后章节", &format!("第 {} 章", idx + 1), theme);
            }

            ui.add_space(s.sm);
            ui.separator();
            ui.add_space(s.sm);

            // Section: 文件状态
            section_header(ui, "文件状态", theme);
            ui.add_space(s.xxs);
            ui.horizontal(|ui| {
                let (status_text, status_color) = match item.file_health {
                    crate::domain::library_item::FileHealth::Ok => ("文件正常", colors.success.to_color32()),
                    crate::domain::library_item::FileHealth::Missing => ("文件缺失", colors.danger.to_color32()),
                    crate::domain::library_item::FileHealth::Moved => ("文件已移动", colors.warning.to_color32()),
                    crate::domain::library_item::FileHealth::ParseWarning => ("解析告警", colors.warning.to_color32()),
                };
                ui.colored_label(status_color, status_text);
            });

            ui.add_space(s.lg);

            // Action buttons
            ui.horizontal(|ui| {
                // Primary "继续阅读" button with accent background
                let btn_text = egui::RichText::new("继续阅读").color(egui::Color32::WHITE);
                let btn = egui::Button::new(btn_text)
                    .fill(colors.accent.to_color32())
                    .corner_radius(egui::CornerRadius::same(r.button as u8));
                if ui.add_sized([120.0, 32.0], btn).clicked() {
                    actions.push(Action::LibraryBookSelected(item.book_id.clone()));
                }
                ui.add_space(s.sm);
                if ui.button("从书库移除").clicked() {
                    actions.push(Action::RemoveFromLibrary(item.book_id.clone()));
                }
            });
            }); // ScrollArea
        });

    if !open {
        actions.push(Action::LibraryDetailClosed);
    }

    actions
}

fn section_header(ui: &mut egui::Ui, title: &str, theme: &ThemeConfig) {
    ui.label(egui::RichText::new(title)
        .size(theme.typography.body_size)
        .strong()
        .color(theme.colors.text_primary.to_color32()));
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str, theme: &ThemeConfig) {
    let typo = &theme.typography;
    let colors = &theme.colors;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{}: ", label))
            .size(typo.caption_size)
            .color(colors.text_secondary.to_color32()));
        ui.label(egui::RichText::new(value)
            .size(typo.caption_size)
            .color(colors.text_primary.to_color32()));
    });
}

fn format_date(rfc3339: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| rfc3339.to_string())
}

fn format_seconds(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    if hours > 0 {
        format!("{}小时{}分", hours, minutes)
    } else {
        format!("{}分钟", minutes)
    }
}
