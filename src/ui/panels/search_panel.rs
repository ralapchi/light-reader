use eframe::egui;

use crate::app::Action;
use crate::domain::app_state::AppState;
use crate::domain::search_enums::SearchScope;
use crate::domain::search_query::SearchQuery;
use crate::ui::ThemeConfig;

pub fn search_panel(
    ctx: &egui::Context,
    state: &AppState,
    theme: &ThemeConfig,
) -> Vec<Action> {
    let s = &theme.spacing;
    let t = &theme.typography;
    let mut actions = Vec::new();
    let mut query_text = state
        .search_state
        .current_query
        .as_ref()
        .map(|q| q.keyword.clone())
        .unwrap_or_default();
    let current_scope = state
        .search_state
        .current_query
        .as_ref()
        .map(|q| q.scope.clone())
        .unwrap_or(SearchScope::CurrentChapter);
    let case_sensitive = state.ui_state.search_case_sensitive;

    egui::SidePanel::right("search_panel")
        .default_width(320.0)
        .min_width(260.0)
        .max_width(480.0)
        .show(ctx, |ui| {
            ui.add_space(s.sm);

            // Header with close button
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("搜索").size(t.section_title_size).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("关闭").clicked() {
                        actions.push(Action::ClearSearch);
                        actions.push(Action::ToggleSearchPanel);
                    }
                });
            });

            ui.add_space(s.xs);
            ui.separator();
            ui.add_space(s.sm);

            // Search input
            let text_edit = egui::TextEdit::singleline(&mut query_text)
                .hint_text("输入关键词...")
                .desired_width(f32::INFINITY);
            let response = ui.add(text_edit);

            // Submit on Enter
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !query_text.is_empty() {
                    actions.push(Action::SearchQueryChanged(SearchQuery {
                        keyword: query_text.clone(),
                        case_sensitive,
                        scope: current_scope.clone(),
                    }));
                    actions.push(Action::SearchSubmitted);
                    response.request_focus();
                }
            }

            // Update query text on change (without submitting)
            if response.changed() {
                actions.push(Action::SearchQueryChanged(SearchQuery {
                    keyword: query_text.clone(),
                    case_sensitive,
                    scope: current_scope.clone(),
                }));
            }

            ui.add_space(s.sm);

            // Scope segmented control
            ui.horizontal(|ui| {
                let scope_current = current_scope == SearchScope::CurrentChapter;
                let scope_all = current_scope == SearchScope::EntireBook;

                if ui
                    .selectable_label(scope_current, "当前章节")
                    .clicked()
                    && !scope_current
                {
                    actions.push(Action::SearchQueryChanged(SearchQuery {
                        keyword: query_text.clone(),
                        case_sensitive,
                        scope: SearchScope::CurrentChapter,
                    }));
                }
                if ui.selectable_label(scope_all, "全书").clicked() && !scope_all {
                    actions.push(Action::SearchQueryChanged(SearchQuery {
                        keyword: query_text.clone(),
                        case_sensitive,
                        scope: SearchScope::EntireBook,
                    }));
                }
            });

            // Case sensitivity toggle
            let mut cs = case_sensitive;
            if ui.checkbox(&mut cs, "区分大小写").changed() {
                actions.push(Action::ToggleSearchCaseSensitive);
                actions.push(Action::SearchQueryChanged(SearchQuery {
                    keyword: query_text.clone(),
                    case_sensitive: cs,
                    scope: current_scope.clone(),
                }));
            }

            ui.add_space(s.sm);
            ui.separator();
            ui.add_space(s.sm);

            // Results
            let results = &state.search_state.results;
            let selected = state.search_state.selected_result_index;

            if results.is_empty() {
                if state.search_state.current_query.is_some() && !query_text.is_empty() {
                    ui.add_space(s.xl);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("无匹配结果")
                                .size(t.body_size)
                                .color(theme.colors.text_muted.to_color32()),
                        );
                    });
                } else {
                    ui.add_space(s.xl);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("输入关键词开始搜索")
                                .size(t.body_size)
                                .color(theme.colors.text_muted.to_color32()),
                        );
                    });
                }
            } else {
                // Result count
                ui.label(
                    egui::RichText::new(format!("找到 {} 个结果", results.len()))
                        .size(t.caption_size)
                        .color(theme.colors.text_secondary.to_color32()),
                );
                ui.add_space(s.xs);

                egui::ScrollArea::vertical()
                    .id_salt("search_results_scroll")
                    .show(ui, |ui| {
                        for (index, result) in results.iter().enumerate() {
                            let is_selected = selected == Some(index);

                            let resp = ui
                                .group(|ui| {
                                    // Chapter title
                                    ui.label(
                                        egui::RichText::new(&result.chapter_title)
                                            .size(t.caption_size)
                                            .color(theme.colors.accent.to_color32()),
                                    );
                                    ui.add_space(s.xxs);

                                    // Snippet with highlight
                                    let snippet = &result.snippet;
                                    ui.label(
                                        egui::RichText::new(snippet)
                                            .size(t.body_size),
                                    );

                                    // Match position
                                    ui.add_space(s.xxs);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "段落 {} · 位置 {}",
                                            result.paragraph_index + 1,
                                            result.match_start
                                        ))
                                        .size(t.caption_size)
                                        .color(theme.colors.text_muted.to_color32()),
                                    );
                                })
                                .response;

                            // Highlight selected
                            if is_selected {
                                let rect = resp.rect;
                                ui.painter().rect_stroke(
                                    rect,
                                    egui::CornerRadius::same(4),
                                    egui::Stroke::new(2.0, theme.colors.accent.to_color32()),
                                    egui::StrokeKind::Outside,
                                );
                            }

                            if resp.interact(egui::Sense::click()).clicked() {
                                actions.push(Action::SearchResultSelected(index));
                            }

                            ui.add_space(s.xxs);
                        }
                    });
            }
        });

    actions
}
