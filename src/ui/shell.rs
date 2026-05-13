use chrono::Utc;
use eframe::egui;
use rfd::FileDialog;

use crate::app::compat::CompatAdapter;
use crate::app::Action;
use crate::domain::enums::ScreenKind;
use crate::ui::image_cache::IMG_CACHE;
use crate::ui::panels::library_page::library_page;
use crate::ui::panels::reader_view::reader_view;
use crate::ui::panels::search_panel::{search_panel, SearchPanelProps};
use crate::ui::panels::settings_panel::{settings_panel, SettingsPanelProps};
use crate::ui::panels::status_bar::status_bar;
use crate::ui::panels::top_bar::{TopBar, TopBarProps};
use crate::ui::panels::tts_player_bar::{tts_player_bar, TtsPlayerBarProps};
use crate::ui::widgets::error_state;
use crate::ui::{ThemeConfig, ThemeService};

pub struct AppShell;

impl AppShell {
    pub fn update(shell: &mut CompatAdapter, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme_kind = shell.state().reader_settings.theme.clone();
        let config = ThemeConfig::from(theme_kind);
        ThemeService::apply_theme(ctx, &config);

        // Auto-clear status messages after 3 seconds
        if let Some(ref set_at) = shell.state().status_message_set_at {
            if let Ok(set_time) = chrono::DateTime::parse_from_rfc3339(set_at) {
                let elapsed = Utc::now().signed_duration_since(set_time);
                if elapsed.num_seconds() >= 3 {
                    shell.dispatch(Action::StatusMessageTimedOut);
                }
            }
        }

        // Keyboard shortcuts
        let shortcuts = Self::collect_shortcuts(ctx);
        for action in shortcuts {
            shell.dispatch(action);
        }

        let screen = shell.state().ui_state.screen.clone();

        match screen {
            ScreenKind::EmptyLibrary => {
                let state = shell.state();
                let actions = library_page(ctx, state, &config);
                for action in actions {
                    shell.dispatch(action);
                }
            }
            ScreenKind::LoadingBook => {
                let state = shell.state();
                let title = state.ui_state.loading_book_title.as_deref().unwrap_or("正在加载");
                let author = state.ui_state.loading_book_author.as_deref();
                let book_id = state.ui_state.loading_book_id.as_deref();
                let cover_key = state.ui_state.loading_book_cover_key.as_deref();
                // Try to load cover for loading screen
                let mut cover_tex = None;
                if let Some(bid) = book_id {
                    cover_tex = IMG_CACHE.with(|c| c.borrow_mut().cover_texture(
                        ctx, bid, cover_key,
                    ));
                }
                let _ = state;
                egui::CentralPanel::default().show(ctx, |ui| {
                    book_loading_screen(ui, title, author, cover_tex.as_ref(), &config);
                });
            }
            ScreenKind::Reader => {
                Self::reader_layout(shell, ctx, &config);
            }
            ScreenKind::Error => {
                let err_msg = shell
                    .state()
                    .last_error
                    .as_ref()
                    .map(|e| format!("[{}] {}", e.code, e.message))
                    .unwrap_or_else(|| "未知错误".to_string());
                let last_path = shell.state().ui_state.last_attempted_path.clone();
                egui::CentralPanel::default().show(ctx, |ui| {
                    let (retry, reopen) =
                        error_state(ui, "出错了", &err_msg, "重试", "重新打开", &config);
                    if retry {
                        if let Some(path) = &last_path {
                            let path_str = path.to_string_lossy().to_string();
                            shell.dispatch(Action::OpenBookSelected(path_str));
                        }
                    }
                    if reopen {
                        if let Some(path) = FileDialog::new()
                            .add_filter("电子书", &["epub", "txt"])
                            .pick_file()
                        {
                            let path_str = path.to_string_lossy().to_string();
                            shell.dispatch(Action::OpenBookSelected(path_str));
                        }
                    }
                });
            }
        }
    }

    fn collect_shortcuts(ctx: &egui::Context) -> Vec<Action> {
        let mut actions = Vec::new();

        ctx.input(|i| {
            let ctrl_or_cmd = i.modifiers.command || i.modifiers.ctrl;

            // Ctrl/Cmd + O → 打开书籍 (file dialog is a side effect, handled separately)
            if ctrl_or_cmd && i.key_pressed(egui::Key::O) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("电子书", &["epub", "txt"])
                    .pick_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    actions.push(Action::OpenBookSelected(path_str));
                }
            }
        });

        actions.extend(Self::map_shortcut_actions(ctx));
        actions
    }

    /// Map keyboard shortcuts to actions (pure logic, excluding file dialog side effects).
    fn map_shortcut_actions(ctx: &egui::Context) -> Vec<Action> {
        use crate::domain::theme_kind::ThemeKind;

        let mut actions = Vec::new();

        ctx.input(|i| {
            let ctrl_or_cmd = i.modifiers.command || i.modifiers.ctrl;

            if ctrl_or_cmd && i.key_pressed(egui::Key::F) {
                actions.push(Action::ToggleSearchPanel);
            }
            if ctrl_or_cmd && i.key_pressed(egui::Key::Comma) {
                actions.push(Action::ToggleSettingsPanel);
            }
            if ctrl_or_cmd && i.key_pressed(egui::Key::B) {
                actions.push(Action::AddBookmarkRequested);
            }
            if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::PageUp) {
                actions.push(Action::PrevChapter);
            }
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::PageDown) {
                actions.push(Action::NextChapter);
            }
            if i.key_pressed(egui::Key::Escape) {
                actions.push(Action::CloseSearchOrSettings);
            }
            if ctrl_or_cmd && i.key_pressed(egui::Key::Num1) {
                actions.push(Action::ThemeChanged(ThemeKind::Light));
            }
            if ctrl_or_cmd && i.key_pressed(egui::Key::Num2) {
                actions.push(Action::ThemeChanged(ThemeKind::Dark));
            }
            if ctrl_or_cmd && i.key_pressed(egui::Key::Num3) {
                actions.push(Action::ThemeChanged(ThemeKind::Sepia));
            }
        });

        actions
    }

    fn reader_layout(shell: &mut CompatAdapter, ctx: &egui::Context, theme: &ThemeConfig) {
        // Collect all panel actions inside a borrow scope to avoid cloning chapters/toc
        let pending_actions = {
            let state = shell.state();

            // Borrow (not clone) chapters and toc
            let chapters = state
                .current_book
                .as_ref()
                .map(|b| b.chapters.as_slice())
                .unwrap_or(&[]);
            let toc = state
                .current_book
                .as_ref()
                .map(|b| b.toc.as_slice())
                .unwrap_or(&[]);

            let settings = &state.reader_settings;

            let current_chapter_index = state
                .reading_progress
                .as_ref()
                .map(|p| p.chapter_index)
                .unwrap_or(0);

            let mut actions: Vec<Action> = Vec::new();

            // Hover toolbar: check mouse proximity to top edge
            let mouse_y = ctx.input(|i| i.pointer.hover_pos().map(|p| p.y).unwrap_or(0.0));
            let toolbar_should_show = mouse_y < theme.panel.top_bar_height || state.ui_state.show_floating_toc
                || state.ui_state.show_search_panel || state.ui_state.show_settings_panel;
            if toolbar_should_show != state.ui_state.reader_toolbar_visible {
                actions.push(Action::SetReaderToolbarVisible(toolbar_should_show));
            }

            // Smooth fade animation for toolbar
            let toolbar_alpha = ctx.animate_bool(
                egui::Id::new("reader_toolbar_fade"),
                toolbar_should_show,
            );
            let toolbar_visible = toolbar_alpha > 0.01;

            // Top bar (hover reveal, floating overlay — doesn't push content)
            if toolbar_visible {
                let top_bar_props = TopBarProps {
                    chapter_index: current_chapter_index,
                    total_chapters: chapters.len(),
                    floating_toc_open: state.ui_state.show_floating_toc,
                    status_message: if settings.show_status_bar { "" } else { &state.status_message },
                };
                let available = ctx.content_rect();
                egui::Area::new("top_bar_overlay".into())
                    .fixed_pos(egui::pos2(0.0, 0.0))
                    .order(egui::Order::Middle)
                    .show(ctx, |ui| {
                        ui.set_opacity(toolbar_alpha);
                        ui.set_max_width(available.width());
                        let bar_actions = TopBar::show(ui, &top_bar_props, theme);
                        actions.extend(bar_actions);
                    });
            }

            // Floating TOC overlay
            if state.ui_state.show_floating_toc {
                let toc_actions = floating_toc_panel(ctx, toc, current_chapter_index, theme);
                actions.extend(toc_actions);
            }

            // Reader content
            let selected_search_result = state
                .search_state
                .selected_result_index
                .and_then(|idx| state.search_state.results.get(idx));
            let search_keyword = state
                .search_state
                .current_query
                .as_ref()
                .map(|q| q.keyword.as_str());
            let case_sensitive = state
                .search_state
                .current_query
                .as_ref()
                .map(|q| q.case_sensitive)
                .unwrap_or(false);
            let status_message: &str = &state.status_message;
            let highlighted_paragraph_indices =
                &state.playback_state.current_paragraph_indices;

            egui::CentralPanel::default().show(ctx, |ui| {
                let reader_actions = reader_view(
                    ui,
                    chapters,
                    current_chapter_index,
                    settings,
                    theme,
                    selected_search_result,
                    search_keyword,
                    case_sensitive,
                    highlighted_paragraph_indices,
                );
                actions.extend(reader_actions);
            });

            // Status bar
            if settings.show_status_bar {
                let progress = state
                    .reading_progress
                    .as_ref()
                    .map(|p| p.progress_percent)
                    .unwrap_or(0.0);
                let chapter_pos = if !chapters.is_empty() {
                    format!("{}/{}", current_chapter_index + 1, chapters.len())
                } else {
                    String::new()
                };
                let char_count = chapters
                    .get(current_chapter_index)
                    .map(|c| c.char_count)
                    .unwrap_or(0);

                egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                    status_bar(ui, progress, &chapter_pos, char_count, status_message, theme);
                });
            }

            // Search panel (overlay on right side)
            if state.ui_state.show_search_panel {
                let search_props = SearchPanelProps {
                    current_query: &state.search_state.current_query,
                    results: &state.search_state.results,
                    selected_result_index: state.search_state.selected_result_index,
                };
                let search_actions = search_panel(ctx, &search_props, theme);
                actions.extend(search_actions);
            }

            // Settings panel (overlay on right side)
            if state.ui_state.show_settings_panel {
                let settings_props = SettingsPanelProps {
                    reader_settings: settings,
                    tts_config: &state.tts_config,
                    tts_state: &state.tts_state,
                };
                let settings_actions = settings_panel(ctx, &settings_props, theme);
                actions.extend(settings_actions);
            }

            // TTS player bar (bottom panel)
            if state.tts_config.enabled {
                let tts_props = TtsPlayerBarProps {
                    tts_state: &state.tts_state,
                    playback_state: &state.playback_state,
                };
                egui::TopBottomPanel::bottom("tts_player_bar").show(ctx, |ui| {
                    let player_actions = tts_player_bar(ui, &tts_props, theme);
                    actions.extend(player_actions);
                });
            }

            actions
        }; // state borrow is released

        // Dispatch all collected actions
        for action in pending_actions {
            shell.dispatch(action);
        }
    }
}

/// Floating TOC overlay panel.
fn floating_toc_panel(
    ctx: &egui::Context,
    toc: &[crate::domain::toc_item::TocItem],
    current_chapter_index: usize,
    theme: &ThemeConfig,
) -> Vec<Action> {
    let mut actions = Vec::new();
    egui::Window::new("目录")
        .collapsible(false)
        .resizable(true)
        .default_width(260.0)
        .anchor(egui::Align2::LEFT_TOP, [16.0, 48.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for item in toc {
                    render_floating_toc_item(ui, item, theme, &mut actions, 0, current_chapter_index);
                }
            });
        });
    actions
}

fn render_floating_toc_item(
    ui: &mut egui::Ui,
    item: &crate::domain::toc_item::TocItem,
    theme: &ThemeConfig,
    actions: &mut Vec<Action>,
    depth: u8,
    current_chapter_index: usize,
) {
    let indent = depth as f32 * 12.0;
    let is_current = item.chapter_index == Some(current_chapter_index);
    let label = if let Some(idx) = item.chapter_index {
        format!("{}. {}", idx + 1, item.title)
    } else {
        item.title.clone()
    };
    let text = if is_current {
        egui::RichText::new(&label).strong().color(theme.colors.sidebar_selected_text.to_color32())
    } else {
        egui::RichText::new(&label)
    };

    ui.horizontal(|ui| {
        ui.add_space(indent);
        if is_current {
            // Capsule-selected state for current chapter
            let desired = egui::vec2(ui.available_width(), 28.0);
            let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter_at(rect);
                // Capsule background
                painter.rect_filled(
                    rect,
                    egui::CornerRadius::same(14),
                    theme.colors.sidebar_selected_bg.to_color32(),
                );
                // Text centered vertically, left-padded
                let text_pos = egui::pos2(rect.left() + 10.0, rect.center().y);
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_CENTER,
                    &label,
                    egui::FontId::new(theme.typography.body_size, egui::FontFamily::Proportional),
                    theme.colors.sidebar_selected_text.to_color32(),
                );
            }
            if resp.clicked() {
                if let Some(idx) = item.chapter_index {
                    actions.push(Action::GoToChapter(idx));
                    actions.push(Action::ToggleFloatingToc);
                }
            }
        } else {
            if ui.selectable_label(false, text).clicked() {
                if let Some(idx) = item.chapter_index {
                    actions.push(Action::GoToChapter(idx));
                    actions.push(Action::ToggleFloatingToc);
                }
            }
        }
    });
    for child in &item.children {
        render_floating_toc_item(ui, child, theme, actions, depth + 1, current_chapter_index);
    }
}

/// Upgraded loading screen with book cover, title, author and spinner.
fn book_loading_screen(
    ui: &mut egui::Ui,
    title: &str,
    author: Option<&str>,
    cover_texture: Option<&egui::TextureHandle>,
    theme: &ThemeConfig,
) {
    let s = &theme.spacing;
    ui.vertical_centered(|ui| {
        ui.add_space(s.chapter_end_spacer);

        // Cover area
        let cover_size = egui::Vec2::new(160.0, 210.0);
        let cover_rect = egui::Rect::from_min_size(ui.next_widget_position(), cover_size);
        let (_r, _resp) = ui.allocate_exact_size(cover_size, egui::Sense::hover());
        if ui.is_rect_visible(cover_rect) {
            let painter = ui.painter_at(cover_rect);
            if let Some(tex) = cover_texture {
                painter.image(tex.id(), cover_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE);
            } else {
                painter.rect_filled(cover_rect, egui::CornerRadius::same(6),
                    theme.colors.panel_bg.to_color32());
                painter.rect_stroke(cover_rect, egui::CornerRadius::same(6),
                    egui::Stroke::new(1.0, theme.colors.border_subtle.to_color32()),
                    egui::StrokeKind::Inside);
            }
        }

        ui.add_space(s.xl);

        // Title
        ui.label(egui::RichText::new(title).size(theme.typography.title_size).strong());
        ui.add_space(s.xs);

        // Author
        if let Some(a) = author {
            ui.label(egui::RichText::new(a).size(theme.typography.caption_size)
                .color(theme.colors.text_secondary.to_color32()));
        }

        ui.add_space(s.xl);

        // Spinner with pulse animation
        let spinner_size = 32.0;
        let time = ui.input(|i| i.time);
        let pulse_alpha = ((time * 2.5).sin() as f32 * 0.5 + 0.5) * 0.12;
        let spinner_pos = ui.next_widget_position();
        let pulse_rect = egui::Rect::from_center_size(
            egui::pos2(spinner_pos.x + spinner_size * 0.5, spinner_pos.y + spinner_size * 0.5),
            egui::Vec2::splat(spinner_size + 16.0),
        );
        let (_sr, _sresp) = ui.allocate_exact_size(egui::Vec2::splat(spinner_size), egui::Sense::hover());
        if ui.is_rect_visible(pulse_rect) {
            ui.painter_at(pulse_rect).rect_filled(
                pulse_rect,
                egui::CornerRadius::same(16),
                theme.colors.accent.to_color32().gamma_multiply(pulse_alpha),
            );
        }
        ui.add(egui::Spinner::new().size(spinner_size));
        ui.add_space(s.sm);
        ui.label(egui::RichText::new("正在打开...").size(theme.typography.caption_size)
            .color(theme.colors.text_muted.to_color32()));

        ui.add_space(s.loading_screen_spacer);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::theme_kind::ThemeKind;
    use eframe::egui;

    fn ctx_with_key(modifiers: egui::Modifiers, key: egui::Key) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.input_mut(|i| {
            i.modifiers = modifiers;
            i.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            });
        });
        ctx
    }

    fn ctrl_key(key: egui::Key) -> egui::Context {
        ctx_with_key(egui::Modifiers::CTRL, key)
    }

    fn no_mod_key(key: egui::Key) -> egui::Context {
        ctx_with_key(egui::Modifiers::NONE, key)
    }

    #[test]
    fn ctrl_f_toggles_search_panel() {
        let ctx = ctrl_key(egui::Key::F);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::ToggleSearchPanel]);
    }

    #[test]
    fn ctrl_comma_toggles_settings_panel() {
        let ctx = ctrl_key(egui::Key::Comma);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::ToggleSettingsPanel]);
    }

    #[test]
    fn ctrl_b_adds_bookmark() {
        let ctx = ctrl_key(egui::Key::B);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::AddBookmarkRequested]);
    }

    #[test]
    fn arrow_left_prev_chapter() {
        let ctx = no_mod_key(egui::Key::ArrowLeft);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::PrevChapter]);
    }

    #[test]
    fn page_up_prev_chapter() {
        let ctx = no_mod_key(egui::Key::PageUp);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::PrevChapter]);
    }

    #[test]
    fn arrow_right_next_chapter() {
        let ctx = no_mod_key(egui::Key::ArrowRight);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::NextChapter]);
    }

    #[test]
    fn page_down_next_chapter() {
        let ctx = no_mod_key(egui::Key::PageDown);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::NextChapter]);
    }

    #[test]
    fn escape_closes_search_or_settings() {
        let ctx = no_mod_key(egui::Key::Escape);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::CloseSearchOrSettings]);
    }

    #[test]
    fn ctrl_1_theme_light() {
        let ctx = ctrl_key(egui::Key::Num1);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::ThemeChanged(ThemeKind::Light)]);
    }

    #[test]
    fn ctrl_2_theme_dark() {
        let ctx = ctrl_key(egui::Key::Num2);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::ThemeChanged(ThemeKind::Dark)]);
    }

    #[test]
    fn ctrl_3_theme_sepia() {
        let ctx = ctrl_key(egui::Key::Num3);
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert_eq!(actions, vec![Action::ThemeChanged(ThemeKind::Sepia)]);
    }

    #[test]
    fn no_key_no_actions() {
        let ctx = egui::Context::default();
        let actions = AppShell::map_shortcut_actions(&ctx);
        assert!(actions.is_empty());
    }
}
