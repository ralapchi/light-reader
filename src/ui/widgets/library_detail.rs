use eframe::egui;

use crate::app::Action;
use crate::domain::library_item::LibraryItem;
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
    let mut actions = Vec::new();

    egui::Window::new(format!("{} - 详情", item.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([380.0, 420.0])
        .show(ctx, |ui| {
            ui.add_space(s.sm);

            // Cover + basic info
            ui.horizontal(|ui| {
                // Mini cover
                let mini_cover_rect = egui::Rect::from_min_size(
                    ui.next_widget_position(),
                    egui::Vec2::new(100.0, 130.0),
                );
                let (_rect, _resp) = ui.allocate_exact_size(egui::Vec2::new(100.0, 130.0), egui::Sense::hover());
                if ui.is_rect_visible(mini_cover_rect) {
                    let painter = ui.painter_at(mini_cover_rect);
                    let cover_color = cover_base_color(item);
                    painter.rect_filled(mini_cover_rect, egui::CornerRadius::same(4), cover_color);
                    painter.rect_stroke(mini_cover_rect, egui::CornerRadius::same(4),
                        egui::Stroke::new(1.0, colors.border_subtle.to_color32()), egui::StrokeKind::Inside);
                    // Format label
                    painter.text(
                        mini_cover_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        book_card::format_tag(&item.format),
                        egui::FontId::new(typo.caption_size, egui::FontFamily::Proportional),
                        egui::Color32::WHITE,
                    );
                }

                ui.add_space(s.md);

                // Right side: key info
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&item.title).size(typo.body_size).strong());
                    ui.add_space(s.xxs);
                    if let Some(ref author) = item.author {
                        ui.label(egui::RichText::new(author).size(typo.caption_size).color(colors.text_secondary.to_color32()));
                    }
                    ui.add_space(s.sm);
                    ui.label(format!("格式: {}", book_card::format_tag(&item.format)));
                });
            });

            ui.add_space(s.md);
            ui.separator();
            ui.add_space(s.sm);

            // Detail fields
            detail_row(ui,"书籍 ID", &item.book_id, theme);
            detail_row(ui,"文件路径", &item.source_path, theme);
            detail_row(ui,"导入时间", &format_date(&item.imported_at), theme);
            if let Some(ref opened) = item.last_opened_at {
                detail_row(ui,"最近打开", &format_date(opened), theme);
            }
            detail_row(ui,"总章节数", &item.chapter_count.to_string(), theme);
            detail_row(ui,"阅读进度", &format!("{:.1}%", item.progress_percent * 100.0), theme);

            ui.add_space(s.sm);
            ui.separator();
            ui.add_space(s.sm);

            // Stats section
            ui.label(egui::RichText::new("阅读统计").size(typo.body_size).strong());
            ui.add_space(s.xs);
            detail_row(ui,"总阅读时长", &format_seconds(item.stats.total_read_seconds), theme);
            detail_row(ui,"书签数", &item.stats.bookmark_count.to_string(), theme);
            if let Some(ref last) = item.stats.last_read_at {
                detail_row(ui,"最后阅读", &format_date(last), theme);
            }
            if let Some(idx) = item.stats.last_chapter_index {
                detail_row(ui,"最后章节", &format!("第 {} 章", idx + 1), theme);
            }

            ui.add_space(s.md);

            // File health status
            ui.horizontal(|ui| {
                let (status_text, status_color) = match item.file_health {
                    crate::domain::library_item::FileHealth::Ok => ("文件正常", colors.accent.to_color32()),
                    crate::domain::library_item::FileHealth::Missing => ("文件缺失", colors.danger.to_color32()),
                    crate::domain::library_item::FileHealth::Moved => ("文件已移动", colors.warning.to_color32()),
                    crate::domain::library_item::FileHealth::ParseWarning => ("解析告警", colors.warning.to_color32()),
                };
                ui.colored_label(status_color, format!("状态: {}", status_text));
            });

            ui.add_space(s.lg);

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("打开阅读").clicked() {
                    actions.push(Action::LibraryBookSelected(item.book_id.clone()));
                }
                ui.add_space(s.sm);
                if ui.button("从书库移除").clicked() {
                    actions.push(Action::RemoveFromLibrary(item.book_id.clone()));
                }
            });
        });

    actions
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

fn cover_base_color(item: &LibraryItem) -> egui::Color32 {
    match item.format {
        crate::domain::book_format::BookFormat::Epub => egui::Color32::from_rgb(60, 100, 160),
        crate::domain::book_format::BookFormat::Txt => egui::Color32::from_rgb(80, 140, 80),
        crate::domain::book_format::BookFormat::ReservedPdf => egui::Color32::from_rgb(170, 70, 70),
        crate::domain::book_format::BookFormat::ReservedMobi => egui::Color32::from_rgb(100, 75, 150),
    }
}
