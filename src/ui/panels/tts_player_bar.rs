use eframe::egui;

use crate::app::Action;
use crate::domain::tts_state::PlaybackState;
use crate::domain::tts_state::TtsState;
use crate::tts::types::PlaybackStatus;
use crate::ui::ThemeConfig;

/// Props for the TTS player bar at the bottom of the reader view.
pub struct TtsPlayerBarProps<'a> {
    pub tts_state: &'a TtsState,
    pub playback_state: &'a PlaybackState,
    pub chapter_count: usize,
}

/// Render the TTS player bar with playback controls.
///
/// Returns a list of Actions triggered by user interaction.
/// The bar is only rendered when `tts_config.enabled` is true (checked by caller).
pub fn tts_player_bar(
    ui: &mut egui::Ui,
    props: &TtsPlayerBarProps<'_>,
    theme: &ThemeConfig,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let s = &theme.spacing;
    let t = &theme.typography;

    ui.horizontal(|ui| {
        // Status indicator
        let (status_text, status_color) = match &props.playback_state.status {
            PlaybackStatus::Idle => ("就绪", theme.colors.text_secondary.to_color32()),
            PlaybackStatus::Buffering => ("缓冲中...", theme.colors.warning.to_color32()),
            PlaybackStatus::Playing => ("播放中", theme.colors.success.to_color32()),
            PlaybackStatus::Paused => ("已暂停", theme.colors.warning.to_color32()),
            PlaybackStatus::Finished => ("播放完毕", theme.colors.text_secondary.to_color32()),
            PlaybackStatus::Error(_) => ("出错", theme.colors.danger.to_color32()),
        };
        ui.label(
            egui::RichText::new(status_text)
                .color(status_color)
                .size(t.caption_size),
        );
        ui.add_space(s.sm);

        // Previous segment button
        let is_first = props.playback_state.current_segment_index.unwrap_or(0) == 0;
        let prev_enabled = matches!(
            props.playback_state.status,
            PlaybackStatus::Playing | PlaybackStatus::Paused | PlaybackStatus::Finished
        ) && !is_first;
        if ui
            .add_enabled(prev_enabled, egui::Button::new("上一段"))
            .clicked()
        {
            actions.push(Action::PlayPrevSegment);
        }
        ui.add_space(s.xxs);

        // Play / Pause / Retry button (changes based on state)
        match &props.playback_state.status {
            PlaybackStatus::Idle | PlaybackStatus::Finished => {
                if ui.button("播放").clicked() {
                    actions.push(Action::StartTts);
                }
            }
            PlaybackStatus::Buffering => {
                ui.add_enabled(false, egui::Button::new("缓冲中"));
            }
            PlaybackStatus::Playing => {
                if ui.button("暂停").clicked() {
                    actions.push(Action::PauseTts);
                }
            }
            PlaybackStatus::Paused => {
                if ui.button("继续").clicked() {
                    actions.push(Action::ResumeTts);
                }
            }
            PlaybackStatus::Error(_) => {
                if ui.button("重试").clicked() {
                    actions.push(Action::StartTts);
                }
            }
        }
        ui.add_space(s.xxs);

        // Next segment button
        let total_segments = props.chapter_count;
        let is_last = props
            .playback_state
            .current_segment_index
            .map_or(true, |idx| idx >= total_segments.saturating_sub(1));
        let next_enabled = matches!(
            props.playback_state.status,
            PlaybackStatus::Playing | PlaybackStatus::Paused | PlaybackStatus::Finished
        ) && !is_last;
        if ui
            .add_enabled(next_enabled, egui::Button::new("下一段"))
            .clicked()
        {
            actions.push(Action::PlayNextSegment);
        }
        ui.add_space(s.sm);

        // Stop button
        let stop_enabled = matches!(
            props.playback_state.status,
            PlaybackStatus::Playing | PlaybackStatus::Paused | PlaybackStatus::Buffering
        );
        if ui
            .add_enabled(stop_enabled, egui::Button::new("停止"))
            .clicked()
        {
            actions.push(Action::StopTts);
        }
        ui.add_space(s.sm);

        // Segment position info
        if let Some(seg_idx) = props.playback_state.current_segment_index {
            let total = total_segments;
            ui.label(
                egui::RichText::new(format!("段落 {}/{}", seg_idx + 1, total))
                    .size(t.caption_size)
                    .color(theme.colors.text_secondary.to_color32()),
            );
        }

        // Push error message to the right side
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(ref err) = props.tts_state.last_error {
                ui.label(
                    egui::RichText::new(err)
                        .color(theme.colors.danger.to_color32())
                        .size(t.caption_size),
                );
            }
        });
    });

    actions
}
