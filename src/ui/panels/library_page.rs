use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::app::Action;
use crate::domain::app_state::AppState;
use crate::domain::book_format::BookFormat;
use crate::domain::library_item::{FileHealth, LibraryItem};
use crate::domain::library_view_state::{LibraryFilterMode, LibrarySortMode, LibraryViewState};
use std::cell::Cell;

use crate::ui::widgets::book_card;
use crate::ui::widgets::library_detail::library_detail_panel;
use crate::ui::ThemeConfig;

thread_local! {
    static SELECTED_DETAIL: Cell<Option<String>> = const { Cell::new(None) };
}

/// The main library home page.
/// Replaces the old EmptyLibrary with a full-featured bookshelf.
pub fn library_page(ctx: &egui::Context, state: &AppState, theme: &ThemeConfig) -> Vec<Action> {
    let s = &theme.spacing;
    let colors = &theme.colors;
    let typo = &theme.typography;
    let mut actions = Vec::new();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(s.sm);

        // ── Top action bar ──────────────────────────────────
        ui.horizontal(|ui| {
            // Import button
            if ui.button(
                egui::RichText::new("\u{2795} 导入书籍")
                    .size(typo.body_size)
            ).clicked() {
                info!("点击了导入书籍按钮");
                if let Some(path) = FileDialog::new()
                    .add_filter("电子书", &["epub", "txt"])
                    .add_filter("EPUB", &["epub"])
                    .add_filter("文本文件", &["txt"])
                    .pick_file()
                {
                    let path_str = path.to_str().unwrap_or("").to_string();
                    // Also trigger the open flow (which will import + read)
                    actions.push(Action::OpenBookSelected(path_str));
                }
            }

            ui.add_space(s.md);

            // Open book button
            if ui.button(
                egui::RichText::new("\u{1F4D6} 打开书籍")
                    .size(typo.body_size)
            ).clicked() {
                info!("点击了打开书籍按钮");
                if let Some(path) = FileDialog::new()
                    .add_filter("电子书", &["epub", "txt"])
                    .add_filter("EPUB", &["epub"])
                    .add_filter("文本文件", &["txt"])
                    .pick_file()
                {
                    let path_str = path.to_str().unwrap_or("").to_string();
                    actions.push(Action::OpenBookSelected(path_str));
                }
            }

            ui.add_space(s.xl);

            // Search box (using local string to avoid &mut on immutable state ref)
            let mut search_query = state.library_view_state.search_query.clone();
            let search_response = ui.add(
                egui::TextEdit::singleline(&mut search_query)
                    .hint_text("\u{1F50D} 搜索书库...")
                    .desired_width(200.0),
            );
            if search_response.changed() {
                actions.push(Action::LibrarySearchChanged(search_query));
            }

            ui.add_space(s.md);

            // Sort dropdown
            let sort_label = sort_mode_label(&state.library_view_state.sort_mode);
            egui::ComboBox::new("library_sort", sort_label)
                .selected_text(sort_label)
                .width(120.0)
                .show_ui(ui, |ui| {
                    let modes = [
                        (LibrarySortMode::LastOpenedDesc, "最近打开"),
                        (LibrarySortMode::ImportedDesc, "导入时间"),
                        (LibrarySortMode::TitleAsc, "标题 A-Z"),
                        (LibrarySortMode::AuthorAsc, "作者 A-Z"),
                        (LibrarySortMode::ProgressDesc, "阅读进度"),
                    ];
                    for (mode, label) in &modes {
                        if ui.selectable_label(
                            state.library_view_state.sort_mode == *mode,
                            *label,
                        ).clicked() {
                            actions.push(Action::LibrarySortChanged(mode.clone()));
                        }
                    }
                });

            ui.add_space(s.sm);

            // Filter dropdown
            let filter_label = filter_mode_label(&state.library_view_state.filter_mode);
            egui::ComboBox::new("library_filter", filter_label)
                .selected_text(filter_label)
                .width(100.0)
                .show_ui(ui, |ui| {
                    let modes = [
                        (LibraryFilterMode::All, "全部"),
                        (LibraryFilterMode::EpubOnly, "EPUB"),
                        (LibraryFilterMode::TxtOnly, "TXT"),
                        (LibraryFilterMode::InProgress, "阅读中"),
                        (LibraryFilterMode::Finished, "已读完"),
                        (LibraryFilterMode::Missing, "缺失"),
                    ];
                    for (mode, label) in &modes {
                        if ui.selectable_label(
                            state.library_view_state.filter_mode == *mode,
                            *label,
                        ).clicked() {
                            actions.push(Action::LibraryFilterChanged(mode.clone()));
                        }
                    }
                });
        });

        ui.add_space(s.lg);
        ui.separator();
        ui.add_space(s.md);

        // ── Build filtered / sorted items list ──────────────
        let filtered_items = filter_and_sort_items(
            &state.library_index.items,
            &state.library_view_state,
        );

        let has_progress_items = state.library_index.items.iter().any(|i| i.progress_percent > 0.0);
        let missing_count = state.library_index.items.iter()
            .filter(|i| i.file_health != FileHealth::Ok)
            .count();

        // ── Continue Reading section ─────────────────────────
        if has_progress_items {
            ui.label(
                egui::RichText::new("\u{1F3F7} 继续阅读")
                    .size(typo.section_title_size)
                    .color(colors.text_primary.to_color32())
                    .strong(),
            );
            ui.add_space(s.sm);

            // Sort by last opened desc and take top 3
            let mut continue_items: Vec<&LibraryItem> = state.library_index.items.iter()
                .filter(|i| i.progress_percent > 0.0 && i.progress_percent < 1.0)
                .collect();
            continue_items.sort_by(|a, b| {
                b.last_opened_at.as_deref().unwrap_or("").cmp(
                    a.last_opened_at.as_deref().unwrap_or(""),
                )
            });
            continue_items.truncate(3);

            ui.horizontal(|ui| {
                for item in &continue_items {
                    let card_responses = book_card::book_card(ui, item, theme);
                    for response in &card_responses {
                        if response.double_clicked() {
                            actions.push(Action::LibraryBookSelected(item.book_id.clone()));
                        } else if response.clicked() {
                            SELECTED_DETAIL.with(|c| c.set(Some(item.book_id.clone())));
                        }
                    }
                    ui.add_space(s.sm);
                }
            });

            ui.add_space(s.lg);
            ui.separator();
            ui.add_space(s.md);
        }

        // ── All Books section ────────────────────────────────
        ui.label(
            egui::RichText::new(format!(
                "\u{1F4DA} 全部书籍 ({})",
                filtered_items.len()
            ))
            .size(typo.section_title_size)
            .color(colors.text_primary.to_color32())
            .strong(),
        );
        ui.add_space(s.sm);

        if filtered_items.is_empty() {
            ui.add_space(s.xl);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("还没有导入书籍，点击上方\"导入书籍\"开始")
                        .size(typo.body_size)
                        .color(colors.text_secondary.to_color32()),
                );
            });
        } else {
            // Display as a wrap grid
            let available_width = ui.available_width();
            let card_width = 160.0 + s.sm; // card + spacing
            let columns = ((available_width - s.sm) / card_width).max(1.0) as usize;

            egui::Grid::new("library_grid")
                .spacing(egui::vec2(s.sm, s.md))
                .min_col_width(160.0)
                .max_col_width(180.0)
                .show(ui, |ui| {
                    for (i, item) in filtered_items.iter().enumerate() {
                        let card_responses = book_card::book_card(ui, item, theme);
                        for response in &card_responses {
                            if response.double_clicked() {
                                actions.push(Action::LibraryBookSelected(item.book_id.clone()));
                            } else if response.clicked() {
                                SELECTED_DETAIL.with(|c| c.set(Some(item.book_id.clone())));
                            }
                        }

                        if (i + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        }

        // ── Missing files alert section ──────────────────────
        if missing_count > 0 {
            ui.add_space(s.lg);
            ui.separator();
            ui.add_space(s.md);

            ui.horizontal(|ui| {
                ui.colored_label(
                    colors.warning.to_color32(),
                    format!("\u{26A0} {} 本书的文件缺失或异常", missing_count),
                );
                ui.add_space(s.md);
                if ui.button("扫描缺失文件").clicked() {
                    actions.push(Action::RescanMissingBooks);
                }
            });

            // Show items with missing files
            let missing_items: Vec<&LibraryItem> = state.library_index.items.iter()
                .filter(|i| i.file_health != FileHealth::Ok)
                .collect();

            for item in &missing_items {
                ui.add_space(s.sm);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&item.title)
                            .color(colors.text_primary.to_color32()),
                    );
                    ui.add_space(s.sm);
                    if ui.button("修复路径").clicked() {
                        if let Some(path) = FileDialog::new()
                            .set_title(&format!("重新指定: {}", item.title))
                            .pick_file()
                        {
                            let new_path = path.to_string_lossy().to_string();
                            actions.push(Action::RepairLibraryPath {
                                book_id: item.book_id.clone(),
                                new_path,
                            });
                        }
                    }
                });
            }
        }

        // ── Bottom status tips ────────────────────────────────
        ui.add_space(s.xl);
        ui.separator();
        ui.add_space(s.sm);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("提示: 双击书籍卡片打开阅读，Ctrl/Cmd+O 快速打开文件")
                    .size(typo.caption_size)
                    .color(colors.text_muted.to_color32()),
            );
        });

        ui.add_space(s.sm);
    });

    // ── Detail panel overlay ──────────────────────────────
    SELECTED_DETAIL.with(|cell| {
        let detail_id = cell.take();
        if let Some(book_id) = detail_id {
            let item_found = state.library_index.items.iter().find(|i| i.book_id == book_id).cloned();
            if let Some(item) = item_found {
                let detail_actions = library_detail_panel(ctx, &item, theme);
                let mut should_open = false;
                for action in &detail_actions {
                    if matches!(action, Action::LibraryBookSelected(_)) {
                        should_open = true;
                    }
                }
                if !should_open {
                    // Re-set to keep the panel open for next frame
                    cell.set(Some(book_id));
                }
                actions.extend(detail_actions);
            }
        }
    });

    actions
}

/// Filter items based on the current search query and filter mode.
fn filter_items<'a>(items: &'a [LibraryItem], view_state: &LibraryViewState) -> Vec<&'a LibraryItem> {
    let query = view_state.search_query.to_lowercase();

    items.iter().filter(|item| {
        // Search filter
        if !query.is_empty() {
            let title_match = item.title.to_lowercase().contains(&query);
            let author_match = item.author.as_deref()
                .map(|a| a.to_lowercase().contains(&query))
                .unwrap_or(false);
            if !title_match && !author_match {
                return false;
            }
        }

        // Type / status filter
        match view_state.filter_mode {
            LibraryFilterMode::All => true,
            LibraryFilterMode::EpubOnly => matches!(item.format, BookFormat::Epub),
            LibraryFilterMode::TxtOnly => matches!(item.format, BookFormat::Txt),
            LibraryFilterMode::InProgress => {
                item.progress_percent > 0.0 && item.progress_percent < 1.0
            }
            LibraryFilterMode::Finished => item.progress_percent >= 1.0,
            LibraryFilterMode::Missing => item.file_health != FileHealth::Ok,
        }
    }).collect()
}

/// Sort items by the current sort mode.
fn sort_items(items: &mut Vec<&LibraryItem>, sort_mode: &LibrarySortMode) {
    match sort_mode {
        LibrarySortMode::LastOpenedDesc => {
            items.sort_by(|a, b| {
                b.last_opened_at.as_deref().unwrap_or("").cmp(
                    a.last_opened_at.as_deref().unwrap_or(""),
                )
            });
        }
        LibrarySortMode::ImportedDesc => {
            items.sort_by(|a, b| {
                b.imported_at.cmp(&a.imported_at)
            });
        }
        LibrarySortMode::TitleAsc => {
            items.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        }
        LibrarySortMode::AuthorAsc => {
            items.sort_by(|a, b| {
                let a_author = a.author.as_deref().unwrap_or("").to_lowercase();
                let b_author = b.author.as_deref().unwrap_or("").to_lowercase();
                a_author.cmp(&b_author)
            });
        }
        LibrarySortMode::ProgressDesc => {
            items.sort_by(|a, b| {
                b.progress_percent.partial_cmp(&a.progress_percent).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
    }
}

/// Combine filter and sort.
fn filter_and_sort_items<'a>(
    items: &'a [LibraryItem],
    view_state: &LibraryViewState,
) -> Vec<&'a LibraryItem> {
    let mut filtered = filter_items(items, view_state);
    sort_items(&mut filtered, &view_state.sort_mode);
    filtered
}

fn sort_mode_label(mode: &LibrarySortMode) -> &'static str {
    match mode {
        LibrarySortMode::LastOpenedDesc => "最近打开",
        LibrarySortMode::ImportedDesc => "导入时间",
        LibrarySortMode::TitleAsc => "标题 A-Z",
        LibrarySortMode::AuthorAsc => "作者 A-Z",
        LibrarySortMode::ProgressDesc => "阅读进度",
    }
}

fn filter_mode_label(mode: &LibraryFilterMode) -> &'static str {
    match mode {
        LibraryFilterMode::All => "全部",
        LibraryFilterMode::EpubOnly => "EPUB",
        LibraryFilterMode::TxtOnly => "TXT",
        LibraryFilterMode::InProgress => "阅读中",
        LibraryFilterMode::Finished => "已读完",
        LibraryFilterMode::Missing => "缺失",
    }
}
