use eframe::egui;
use log::info;
use rfd::FileDialog;

use crate::app::Action;
use crate::domain::app_state::AppState;
use crate::domain::book_format::BookFormat;
use crate::domain::enums::LibraryNavSection;
use crate::domain::library_item::{FileHealth, LibraryItem};
use crate::domain::library_view_state::{LibraryFilterMode, LibrarySortMode, LibraryViewState};
use std::cell::Cell;

use crate::ui::image_cache::IMG_CACHE;
use crate::ui::widgets::book_card;
use crate::ui::widgets::library_detail::library_detail_panel;
use crate::ui::ThemeConfig;

thread_local! {
    static SELECTED_DETAIL: Cell<Option<String>> = const { Cell::new(None) };
}

/// The main library home page with dual-panel layout.
pub fn library_page(ctx: &egui::Context, state: &AppState, theme: &ThemeConfig) -> Vec<Action> {
    let mut actions = Vec::new();

    // ── Left sidebar ──────────────────────────────────────
    let sidebar_actions = library_sidebar(ctx, state, theme);
    actions.extend(sidebar_actions);

    // ── Central content ───────────────────────────────────
    let content_actions = library_content(ctx, state, theme);
    actions.extend(content_actions);

    // ── Detail panel overlay ──────────────────────────────
    SELECTED_DETAIL.with(|cell| {
        let detail_id = cell.take();
        if let Some(book_id) = detail_id {
            let item_found = state.library_index.items.iter().find(|i| i.book_id == book_id).cloned();
            if let Some(item) = item_found {
                let detail_actions = library_detail_panel(ctx, &item, theme);
                let mut should_close = false;
                for action in &detail_actions {
                    if matches!(action, Action::LibraryBookSelected(_) | Action::LibraryDetailClosed | Action::RemoveFromLibrary(_)) {
                        should_close = true;
                    }
                }
                if !should_close {
                    cell.set(Some(book_id));
                }
                actions.extend(detail_actions);
            }
        }
    });

    actions
}

// ═══════════════════════════════════════════════════════════
// Sidebar
// ═══════════════════════════════════════════════════════════

fn library_sidebar(ctx: &egui::Context, state: &AppState, theme: &ThemeConfig) -> Vec<Action> {
    let s = &theme.spacing;
    let colors = &theme.colors;
    let typo = &theme.typography;
    let mut actions = Vec::new();
    let selected_nav = &state.library_view_state.selected_nav;

    egui::SidePanel::left("library_sidebar")
        .resizable(true)
        .default_width(theme.panel.sidebar_default_width)
        .min_width(theme.panel.sidebar_min_width)
        .max_width(theme.panel.sidebar_max_width)
        .frame(egui::Frame::new()
            .fill(colors.sidebar_bg.to_color32())
            .inner_margin(egui::vec2(s.md, s.md)))
        .show(ctx, |ui| {
            // ── App title ──────────────────────────────────
            ui.add_space(s.sm);
            ui.label(
                egui::RichText::new("蓬莱山下")
                    .size(typo.title_size)
                    .color(colors.accent.to_color32())
                    .strong(),
            );
            ui.add_space(s.xs);
            ui.label(
                egui::RichText::new("阅读器")
                    .size(typo.caption_size)
                    .color(colors.text_muted.to_color32()),
            );
            ui.add_space(s.lg);

            // ── Search box ─────────────────────────────────
            let mut search_query = state.library_view_state.sidebar_search_query.clone();
            let search_resp = ui.add(
                egui::TextEdit::singleline(&mut search_query)
                    .hint_text("搜索书籍...")
                    .desired_width(ui.available_width()),
            );
            if search_resp.changed() {
                actions.push(Action::LibrarySidebarSearchChanged(search_query));
            }
            ui.add_space(s.lg);

            // ── Nav: 主页 ──────────────────────────────────
            nav_label(ui, "主页", typo.caption_size, colors.text_muted.to_color32());
            ui.add_space(s.xxs);
            nav_item(ui, "首页", LibraryNavSection::Home, selected_nav, theme, &mut actions);
            ui.add_space(s.md);

            // ── Nav: 书库 ──────────────────────────────────
            nav_label(ui, "书库", typo.caption_size, colors.text_muted.to_color32());
            ui.add_space(s.xxs);
            nav_item(ui, "全部书籍", LibraryNavSection::AllBooks, selected_nav, theme, &mut actions);
            nav_item(ui, "阅读中", LibraryNavSection::InProgress, selected_nav, theme, &mut actions);
            nav_item(ui, "已读完", LibraryNavSection::Finished, selected_nav, theme, &mut actions);
            ui.add_space(s.md);

            // ── Nav: 功能 ──────────────────────────────────

            // Push import button to bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(s.md);
                // Import button
                let import_btn = egui::Button::new(
                    egui::RichText::new("+ 导入书籍")
                        .size(typo.body_size)
                        .color(egui::Color32::WHITE),
                )
                .fill(colors.accent.to_color32())
                .corner_radius(egui::CornerRadius::same(theme.radius.button as u8));
                if ui.add_sized(egui::vec2(ui.available_width(), 36.0), import_btn).clicked() {
                    info!("点击了导入书籍按钮");
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
            });
        });

    actions
}

/// Render a small group label in the sidebar.
fn nav_label(ui: &mut egui::Ui, text: &str, size: f32, color: egui::Color32) {
    ui.label(
        egui::RichText::new(text)
            .size(size)
            .color(color)
            .strong(),
    );
}

/// Render a sidebar nav item with capsule-selected state.
fn nav_item(
    ui: &mut egui::Ui,
    label: &str,
    section: LibraryNavSection,
    selected: &LibraryNavSection,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let colors = &theme.colors;
    let typo = &theme.typography;
    let is_selected = *selected == section;

    let text = if is_selected {
        egui::RichText::new(label)
            .size(typo.body_size)
            .color(colors.sidebar_selected_text.to_color32())
            .strong()
    } else {
        egui::RichText::new(label)
            .size(typo.body_size)
            .color(colors.text_primary.to_color32())
    };

    let btn = if is_selected {
        egui::Button::new(text)
            .fill(colors.sidebar_selected_bg.to_color32())
            .corner_radius(egui::CornerRadius::same(20))
            .stroke(egui::Stroke::NONE)
    } else {
        egui::Button::new(text)
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
    };

    let resp = ui.add_sized(egui::vec2(ui.available_width(), 32.0), btn);
    if resp.hovered() && !is_selected {
        // Draw hover background
        let painter = ui.painter_at(resp.rect);
        painter.rect_filled(
            resp.rect,
            egui::CornerRadius::same(8),
            colors.sidebar_hover_bg.to_color32(),
        );
    }
    if resp.clicked() {
        actions.push(Action::LibraryNavChanged(section));
    }
}

// ═══════════════════════════════════════════════════════════
// Content Router
// ═══════════════════════════════════════════════════════════

fn library_content(ctx: &egui::Context, state: &AppState, theme: &ThemeConfig) -> Vec<Action> {
    let selected_nav = &state.library_view_state.selected_nav;
    match selected_nav {
        LibraryNavSection::Home => home_view(ctx, state, theme),
        LibraryNavSection::AllBooks => all_books_view(ctx, state, theme),
        LibraryNavSection::InProgress => filtered_books_view(ctx, state, theme, "阅读中", LibraryFilterMode::InProgress),
        LibraryNavSection::Finished => filtered_books_view(ctx, state, theme, "已读完", LibraryFilterMode::Finished),
    }
}

// ── Home View ─────────────────────────────────────────────

fn home_view(ctx: &egui::Context, state: &AppState, theme: &ThemeConfig) -> Vec<Action> {
    let s = &theme.spacing;
    let colors = &theme.colors;
    let typo = &theme.typography;
    let mut actions = Vec::new();

    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(egui::vec2(s.lg, s.lg)))
        .show(ctx, |ui| {
            ui.add_space(s.sm);

            // ── Continue Reading section (1.3x cards) ─────
            let continue_items: Vec<&LibraryItem> = {
                let mut items: Vec<&LibraryItem> = state.library_index.items.iter()
                    .filter(|i| i.progress_percent > 0.0 && i.progress_percent < 1.0)
                    .collect();
                items.sort_by(|a, b| {
                    b.last_opened_at.as_deref().unwrap_or("").cmp(
                        a.last_opened_at.as_deref().unwrap_or(""),
                    )
                });
                items.truncate(3);
                items
            };

            if !continue_items.is_empty() {
                ui.label(
                    egui::RichText::new("继续阅读")
                        .size(typo.section_title_size)
                        .color(colors.text_primary.to_color32())
                        .strong(),
                );
                ui.add_space(s.md);

                ui.horizontal(|ui| {
                    for item in &continue_items {
                        let cover_tex = IMG_CACHE.with(|c| c.borrow_mut().cover_texture(ctx, &item.book_id, item.cover_cache_key.as_deref()));
                        let (card_responses, card_actions) = book_card::book_card_scaled(ui, item, theme, cover_tex.as_ref(), 1.3);
                        actions.extend(card_actions);
                        for response in &card_responses {
                            if response.double_clicked() {
                                actions.push(Action::LibraryBookSelected(item.book_id.clone()));
                            } else if response.clicked() {
                                SELECTED_DETAIL.with(|c| c.set(Some(item.book_id.clone())));
                            }
                        }
                        ui.add_space(s.md);
                    }
                });

                ui.add_space(s.xl);
            }

            // ── Recently imported section ─────────────────
            let recent_items: Vec<&LibraryItem> = {
                let mut items: Vec<&LibraryItem> = state.library_index.items.iter().collect();
                items.sort_by(|a, b| b.imported_at.cmp(&a.imported_at));
                items.truncate(8);
                items
            };

            if !recent_items.is_empty() {
                ui.label(
                    egui::RichText::new("最近导入")
                        .size(typo.section_title_size)
                        .color(colors.text_primary.to_color32())
                        .strong(),
                );
                ui.add_space(s.sm);

                render_book_grid(ui, &recent_items, ctx, theme, &mut actions);
            }

            // Empty state
            if state.library_index.items.is_empty() {
                ui.add_space(s.xl * 2.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("欢迎来到蓬莱山下阅读器")
                            .size(typo.title_size)
                            .color(colors.text_primary.to_color32())
                            .strong(),
                    );
                    ui.add_space(s.md);
                    ui.label(
                        egui::RichText::new("点击左侧\"导入书籍\"按钮开始阅读之旅")
                            .size(typo.body_size)
                            .color(colors.text_secondary.to_color32()),
                    );
                });
            }
        });

    actions
}

// ── All Books View ────────────────────────────────────────

fn all_books_view(ctx: &egui::Context, state: &AppState, theme: &ThemeConfig) -> Vec<Action> {
    let s = &theme.spacing;
    let colors = &theme.colors;
    let typo = &theme.typography;
    let mut actions = Vec::new();

    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(egui::vec2(s.lg, s.lg)))
        .show(ctx, |ui| {
            // Header with sort/filter controls
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("全部书籍 ({})", state.library_index.items.len()))
                        .size(typo.section_title_size)
                        .color(colors.text_primary.to_color32())
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                });
            });

            ui.add_space(s.md);
            ui.separator();
            ui.add_space(s.md);

            let filtered_items = filter_and_sort_items(
                &state.library_index.items,
                &state.library_view_state,
            );

            if filtered_items.is_empty() {
                ui.add_space(s.xl);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("还没有导入书籍")
                            .size(typo.body_size)
                            .color(colors.text_secondary.to_color32()),
                    );
                });
            } else {
                render_book_grid(ui, &filtered_items, ctx, theme, &mut actions);
            }
        });

    actions
}

// ── Filtered Books View (InProgress / Finished) ──────────

fn filtered_books_view(
    ctx: &egui::Context,
    state: &AppState,
    theme: &ThemeConfig,
    title: &str,
    filter: LibraryFilterMode,
) -> Vec<Action> {
    let s = &theme.spacing;
    let colors = &theme.colors;
    let typo = &theme.typography;
    let mut actions = Vec::new();

    // Build a temporary view state with the specific filter
    let filtered_items: Vec<&LibraryItem> = state.library_index.items.iter().filter(|item| {
        match filter {
            LibraryFilterMode::InProgress => item.progress_percent > 0.0 && item.progress_percent < 1.0,
            LibraryFilterMode::Finished => item.progress_percent >= 1.0,
            _ => true,
        }
    }).collect();

    egui::CentralPanel::default()
        .frame(egui::Frame::new().inner_margin(egui::vec2(s.lg, s.lg)))
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(format!("{} ({})", title, filtered_items.len()))
                    .size(typo.section_title_size)
                    .color(colors.text_primary.to_color32())
                    .strong(),
            );
            ui.add_space(s.md);
            ui.separator();
            ui.add_space(s.md);

            if filtered_items.is_empty() {
                ui.add_space(s.xl);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(match filter {
                            LibraryFilterMode::InProgress => "没有正在阅读的书籍",
                            LibraryFilterMode::Finished => "没有已完成的书籍",
                            _ => "暂无内容",
                        })
                        .size(typo.body_size)
                        .color(colors.text_secondary.to_color32()),
                    );
                });
            } else {
                render_book_grid(ui, &filtered_items, ctx, theme, &mut actions);
            }
        });

    actions
}

// ═══════════════════════════════════════════════════════════
// Shared Helpers
// ═══════════════════════════════════════════════════════════

/// Render a grid of book cards.
fn render_book_grid(
    ui: &mut egui::Ui,
    items: &[&LibraryItem],
    ctx: &egui::Context,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
) {
    let available_width = ui.available_width();
    let card_width = theme.panel.card_width + theme.panel.card_gap;
    let columns = ((available_width - theme.panel.card_gap) / card_width).max(1.0) as usize;

    egui::Grid::new("library_book_grid")
        .spacing(egui::vec2(theme.panel.card_gap, theme.spacing.md))
        .min_col_width(theme.panel.card_width)
        .max_col_width(theme.panel.card_width + theme.panel.card_gap)
        .show(ui, |ui| {
            for (i, item) in items.iter().enumerate() {
                let cover_tex = IMG_CACHE.with(|c| c.borrow_mut().cover_texture(ctx, &item.book_id, item.cover_cache_key.as_deref()));
                let (card_responses, card_actions) = book_card::book_card(ui, item, theme, cover_tex.as_ref());
                actions.extend(card_actions);
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

/// Filter items based on the current search query and filter mode.
fn filter_items<'a>(items: &'a [LibraryItem], view_state: &LibraryViewState) -> Vec<&'a LibraryItem> {
    let query = view_state.search_query.to_lowercase();

    items.iter().filter(|item| {
        if !query.is_empty() {
            let title_match = item.title.to_lowercase().contains(&query);
            let author_match = item.author.as_deref()
                .map(|a| a.to_lowercase().contains(&query))
                .unwrap_or(false);
            if !title_match && !author_match {
                return false;
            }
        }
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
