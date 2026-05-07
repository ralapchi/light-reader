use chrono::Utc;
use eframe::egui;
use rfd::FileDialog;

use crate::app::compat::CompatAdapter;
use crate::app::Action;
use crate::domain::enums::ScreenKind;
use crate::ui::panels::left_sidebar::left_sidebar;
use crate::ui::panels::library_page::library_page;
use crate::ui::panels::reader_view::reader_view;
use crate::ui::panels::search_panel::{search_panel, SearchPanelProps};
use crate::ui::panels::settings_panel::{settings_panel, SettingsPanelProps};
use crate::ui::panels::status_bar::status_bar;
use crate::ui::panels::top_bar::{TopBar, TopBarProps};
use crate::ui::widgets::{error_state, loading_state};
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
                egui::CentralPanel::default().show(ctx, |ui| {
                    loading_state(ui, "正在加载", "请稍候...", &config);
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
            let active_tab = &state.ui_state.left_panel_tab;

            let current_chapter_index = state
                .reading_progress
                .as_ref()
                .map(|p| p.chapter_index)
                .unwrap_or(0);

            let mut actions: Vec<Action> = Vec::new();

            // Left sidebar
            if settings.show_toc && !state.ui_state.sidebar_collapsed {
                let sidebar_actions = left_sidebar(
                    ctx,
                    active_tab,
                    toc,
                    &state.bookmarks,
                    &state.recent_books,
                    theme,
                    settings.toc_width,
                    current_chapter_index,
                );
                actions.extend(sidebar_actions);
            }

            // Top bar
            let top_bar_props = TopBarProps {
                sidebar_collapsed: state.ui_state.sidebar_collapsed,
                chapter_index: current_chapter_index,
                total_chapters: chapters.len(),
                // Show status message in top bar only when status bar is hidden
                status_message: if settings.show_status_bar { "" } else { &state.status_message },
            };
            egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
                let bar_actions = TopBar::show(ui, &top_bar_props, theme);
                actions.extend(bar_actions);
            });

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
                };
                let settings_actions = settings_panel(ctx, &settings_props, theme);
                actions.extend(settings_actions);
            }

            actions
        }; // state borrow is released

        // Dispatch all collected actions
        for action in pending_actions {
            shell.dispatch(action);
        }
    }
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
